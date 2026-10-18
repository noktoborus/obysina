//! `notes wait <directory>`: waits for a comment for the chat, prints it
//! with the notes waiting, deletes it and ends.
//!
//! The agent runs it in the background: its end wakes the agent in the
//! chat, its output is what the user asked.

use std::path::Path;
use std::time::Duration;

use mockups::note::Said;

use crate::error::{NotesError, Result};
use crate::receiver::{CHAT_FILE, list};

/// How often the file is looked for.
const STEP: Duration = Duration::from_millis(500);

/// Waits until the receiver has kept a comment for the chat.
pub fn run(directory: &Path) -> Result<()> {
    let path = directory.join(CHAT_FILE);
    while !path.exists() {
        std::thread::sleep(STEP);
    }
    let text = std::fs::read_to_string(&path).map_err(|source| NotesError::File {
        path: path.clone(),
        source,
    })?;
    std::fs::remove_file(&path).map_err(|source| NotesError::File {
        path: path.clone(),
        source,
    })?;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let said: Said =
            serde_json::from_str(line).map_err(|source| NotesError::Json { source })?;
        println!("comment: {}", said.chat.comment.trim());
    }
    let kept = list(directory)?;
    println!("notes waiting: {}", kept.len());
    for note in kept {
        let shot = note
            .shot
            .map(|shot| directory.join(shot).display().to_string())
            .unwrap_or_default();
        println!(
            "#{} {}: {} {}",
            note.id,
            note.note.mock,
            note.note.text.trim(),
            shot
        );
        for widget in &note.note.widgets {
            let mut path = widget.path.clone();
            path.push(widget.name.clone());
            println!("    {}", path.join(" \u{203a} "));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two comments kept by the receiver are read at once, and the file is
    /// gone after.
    #[test]
    fn a_comment_kept_ends_the_wait() {
        let directory = tempfile::tempdir().expect("a scratch directory");
        for comment in ["first", "second"] {
            crate::receiver::say(
                directory.path(),
                format!("{{\"comment\":\"{comment}\"}}").as_bytes(),
            )
            .expect("the comment is kept");
        }
        let kept = std::fs::read_to_string(directory.path().join(CHAT_FILE)).expect("the file");
        assert_eq!(kept.lines().count(), 2);

        run(directory.path()).expect("the wait ends");

        assert!(!directory.path().join(CHAT_FILE).exists());
    }
}
