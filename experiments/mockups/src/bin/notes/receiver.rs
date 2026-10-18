//! The server: one request at a time, each note a file of the directory.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use mockups::note::{Kept, Posted};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::error::{NotesError, Result};

/// The port when none is given.
const PORT: u16 = 6081;
/// The path a request names.
const PATH: &str = "/notes";

/// Listens until the process is stopped. The arguments are the directory
/// and, if given, the port.
pub fn run(mut arguments: impl Iterator<Item = String>) -> Result<()> {
    let directory = PathBuf::from(arguments.next().unwrap_or_else(|| "notes".to_string()));
    let port = match arguments.next() {
        Some(text) => text.parse().map_err(|_| NotesError::Port { text })?,
        None => PORT,
    };
    std::fs::create_dir_all(&directory).map_err(|source| NotesError::File {
        path: directory.clone(),
        source,
    })?;
    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address).map_err(|error| NotesError::Listen {
        address: address.clone(),
        message: error.to_string(),
    })?;
    log::info!(
        "notes on http://{address}{PATH}, kept in {}",
        directory.display()
    );
    for request in server.incoming_requests() {
        answer(&directory, request);
    }
    Ok(())
}

/// Answers one request.
fn answer(directory: &Path, mut request: Request) {
    let path = request
        .url()
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    let result = match (request.method(), path.as_str()) {
        (Method::Get, PATH) => list(directory).and_then(|kept| json(&kept)),
        (Method::Post, PATH) => {
            let mut body = Vec::new();
            match std::io::Read::read_to_end(request.as_reader(), &mut body) {
                Ok(_) => keep(directory, &body).and_then(|kept| json(&kept)),
                Err(source) => Err(NotesError::File {
                    path: PathBuf::from(PATH),
                    source,
                }),
            }
        }
        (Method::Delete, _) if path.starts_with(&format!("{PATH}/")) => {
            let id = path[PATH.len() + 1..].to_string();
            delete(directory, &id)
                .and_then(|()| list(directory))
                .and_then(|kept| json(&kept))
        }
        _ => {
            let _ = request.respond(Response::empty(404));
            return;
        }
    };
    let response = match result {
        Ok(body) => Response::from_data(body).with_header(
            Header::from_bytes("Content-Type", "application/json").expect("a valid header"),
        ),
        Err(error) => {
            log::warn!("{error}");
            Response::from_string(error.to_string()).with_status_code(400)
        }
    };
    if let Err(error) = request.respond(response) {
        log::warn!("the answer did not go out: {error}");
    }
}

/// The text of a value.
fn json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty(value).map_err(|source| NotesError::Json { source })
}

/// Every note of the directory, by its number.
fn list(directory: &Path) -> Result<Vec<Kept>> {
    let file = |source| NotesError::File {
        path: directory.to_path_buf(),
        source,
    };
    let mut kept = Vec::new();
    for entry in std::fs::read_dir(directory).map_err(file)? {
        let path = entry.map_err(file)?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let text = std::fs::read(&path).map_err(|source| NotesError::File {
                path: path.clone(),
                source,
            })?;
            kept.push(serde_json::from_slice(&text).map_err(|source| NotesError::Json { source })?);
        }
    }
    kept.sort_by_key(|note: &Kept| note.id);
    Ok(kept)
}

/// Keeps a posted note and its picture, under the next number.
fn keep(directory: &Path, body: &[u8]) -> Result<Kept> {
    let posted: Posted =
        serde_json::from_slice(body).map_err(|source| NotesError::Json { source })?;
    let id = list(directory)?.last().map_or(1, |last| last.id + 1);
    let shot = match &posted.shot {
        Some(shot) => {
            let name = format!("{id:04}.png");
            picture(&directory.join(&name), shot)?;
            Some(name)
        }
        None => None,
    };
    let at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let kept = Kept {
        id,
        at,
        note: posted.note,
        shot,
    };
    let path = directory.join(format!("{id:04}.json"));
    std::fs::write(&path, json(&kept)?).map_err(|source| NotesError::File { path, source })?;
    log::info!("note {id} on {}: {}", kept.note.mock, kept.note.text);
    Ok(kept)
}

/// Deletes a note and its picture; a note that is not there is gone
/// already.
fn delete(directory: &Path, id: &str) -> Result<()> {
    let id: u32 = id.parse().map_err(|_| NotesError::Id {
        text: id.to_string(),
    })?;
    for extension in ["json", "png"] {
        let path = directory.join(format!("{id:04}.{extension}"));
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(source) => return Err(NotesError::File { path, source }),
        }
    }
    log::info!("note {id} deleted");
    Ok(())
}

/// Writes the pixels of a picture as PNG.
fn picture(path: &Path, shot: &mockups::note::Shot) -> Result<()> {
    let pixels = base64::engine::general_purpose::STANDARD
        .decode(&shot.rgba)
        .map_err(|source| NotesError::Base64 { source })?;
    let [width, height] = shot.size;
    if pixels.len() != width * height * 4 {
        return Err(NotesError::PictureSize {
            width,
            height,
            bytes: pixels.len(),
        });
    }
    let file = std::fs::File::create(path).map_err(|source| NotesError::File {
        path: path.to_path_buf(),
        source,
    })?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width as u32, height as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|source| NotesError::Png { source })?;
    writer
        .write_image_data(&pixels)
        .map_err(|source| NotesError::Png { source })
}
