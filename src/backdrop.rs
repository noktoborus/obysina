//! The picture under the pattern (`docs/backdrop.md`): kept in the store of
//! pictures of the cache, `obusina-backdrop`, by the identifier of the
//! pattern; decoded once, and laid on the field by its four corners.
//!
//! A change of the picture is no step of undo: the picture is not part of
//! the pattern.

use std::sync::{Arc, Mutex};

use egui::{ColorImage, TextureHandle, Vec2, vec2};
use obusina_backdrop::{Backdrop as Description, Kept, Kind, Store};
use obusina_pattern::{Bounds, Cell, Pattern};

use crate::error::{AppError, Result};
use crate::pick::Picked;

/// The longest side a picture is drawn at, in pixels: a larger one is drawn
/// smaller. The file in the store keeps all of it.
pub const PICTURE_SIDE: u32 = 4096;
/// How much of a new picture shows.
pub const DEFAULT_OPACITY: f32 = 0.6;
/// The name of the directory of pictures in the cache.
pub const DIRECTORY: &str = "backdrops";

/// The picture under the pattern, and the store it is kept in.
#[derive(Default)]
pub struct Backdrop {
    /// The store; `None` when the platform gives no cache.
    store: Option<Store>,
    /// The picture of the store, when the pattern has one.
    kept: Option<Kept>,
    /// The picture decoded, until it is a texture.
    image: Option<ColorImage>,
    texture: Option<TextureHandle>,
    /// The corners on the field, in bead units, as they are drawn now.
    pub(crate) corners: [Vec2; 4],
    /// The corners before the editing of the corners began, while it goes on.
    editing: Option<[Vec2; 4]>,
    /// How much of the picture shows; the slider of its menu changes it.
    pub(crate) opacity: Arc<Mutex<f32>>,
}

impl Backdrop {
    /// The picture of `pattern` in the store of `dir`, if it has one, and
    /// what went wrong reading it.
    pub fn open(
        dir: Option<std::path::PathBuf>,
        pattern: &Pattern,
    ) -> (Backdrop, Option<AppError>) {
        let mut backdrop = Backdrop {
            store: dir.map(Store::new),
            opacity: Arc::new(Mutex::new(DEFAULT_OPACITY)),
            ..Backdrop::default()
        };
        let error = backdrop.load(pattern).err();
        (backdrop, error)
    }

    /// Reads the picture of the pattern from the store.
    fn load(&mut self, pattern: &Pattern) -> Result<()> {
        let Some(store) = &self.store else {
            return Ok(());
        };
        let Some(kept) = store.find(&pattern.id().to_string())? else {
            return Ok(());
        };
        let bytes = std::fs::read(&kept.picture).map_err(|source| AppError::PictureRead {
            path: kept.picture.clone(),
            source,
        })?;
        self.image = Some(decode(&bytes)?);
        self.corners = kept.backdrop.corners.map(|[x, y]| vec2(x, y));
        self.set_opacity(kept.backdrop.opacity);
        self.kept = Some(kept);
        Ok(())
    }

    /// True when the pattern has a picture.
    pub fn has(&self) -> bool {
        self.kept.is_some()
    }

    /// True when the picture is kept but not drawn.
    pub fn hidden(&self) -> bool {
        self.kept.as_ref().is_some_and(|kept| kept.backdrop.hidden)
    }

    /// True while the corners are edited.
    pub fn editing(&self) -> bool {
        self.editing.is_some()
    }

    /// How much of the picture shows.
    pub fn opacity(&self) -> f32 {
        *self.opacity.lock().expect("the opacity is never poisoned")
    }

    fn set_opacity(&mut self, opacity: f32) {
        *self.opacity.lock().expect("the opacity is never poisoned") = opacity;
    }

    /// The picture for the canvas, while there is one to draw. Its texture is
    /// made the first time it is asked.
    pub fn picture(&mut self, context: &egui::Context) -> Option<obusina_canvas::Picture<'_>> {
        if !self.has() || (self.hidden() && !self.editing()) {
            return None;
        }
        if let Some(image) = self.image.take() {
            self.texture =
                Some(context.load_texture("backdrop", image, egui::TextureOptions::LINEAR));
        }
        let texture = self.texture.as_ref()?.id();
        let opacity = self.opacity();
        let editing = self.editing();
        Some(obusina_canvas::Picture {
            texture,
            corners: &mut self.corners,
            opacity,
            editing,
        })
    }

    /// Puts a picture chosen by the person under the pattern, in place of the
    /// one it had, whose corners it takes. A first picture lies over the
    /// sheet, as large as fits it, its sides as they are.
    pub fn put(&mut self, pattern: &Pattern, picked: Picked) -> Result<()> {
        if self.store.is_none() {
            return Err(AppError::NoCache);
        }
        Kind::of(&picked.bytes).ok_or(AppError::PictureKind)?;
        let image = decode(&picked.bytes)?;
        if !self.has() {
            self.corners = over_sheet(pattern, image.size);
            self.set_opacity(DEFAULT_OPACITY);
        }
        let description = Description {
            pattern: pattern.id().to_string(),
            corners: self.corners.map(|corner| [corner.x, corner.y]),
            opacity: self.opacity(),
            hidden: false,
            source: picked.name,
        };
        let store = self.store.as_ref().ok_or(AppError::NoCache)?;
        let kept = store.put(&picked.bytes, description)?;
        self.kept = Some(kept);
        self.image = Some(image);
        self.texture = None;
        Ok(())
    }

    /// Writes how the picture lies now: its corners, its opacity, whether it
    /// is hidden.
    pub fn save(&mut self) -> Result<()> {
        let corners = self.corners.map(|corner| [corner.x, corner.y]);
        let opacity = self.opacity();
        let (Some(store), Some(kept)) = (&self.store, &mut self.kept) else {
            return Ok(());
        };
        let before = kept.backdrop.clone();
        kept.backdrop.corners = corners;
        kept.backdrop.opacity = opacity;
        if kept.backdrop != before {
            store.save(kept)?;
        }
        Ok(())
    }

    /// Hides the picture or shows it again, and writes that.
    pub fn set_hidden(&mut self, hidden: bool) -> Result<()> {
        let (Some(store), Some(kept)) = (&self.store, &mut self.kept) else {
            return Ok(());
        };
        kept.backdrop.hidden = hidden;
        store.save(kept)?;
        Ok(())
    }

    /// Takes the picture away, and its copy in the store.
    pub fn remove(&mut self) -> Result<()> {
        self.editing = None;
        self.image = None;
        self.texture = None;
        let (Some(store), Some(kept)) = (&self.store, self.kept.take()) else {
            return Ok(());
        };
        store.remove(&kept)?;
        Ok(())
    }

    /// Begins the editing of the corners.
    pub fn begin_editing(&mut self) {
        if self.has() {
            self.editing = Some(self.corners);
        }
    }

    /// Ends the editing of the corners: keeps them and writes them, or puts
    /// back those it began with.
    pub fn end_editing(&mut self, keep: bool) -> Result<()> {
        let Some(before) = self.editing.take() else {
            return Ok(());
        };
        if keep {
            self.save()
        } else {
            self.corners = before;
            Ok(())
        }
    }
}

/// The picture of `bytes`, no side longer than `PICTURE_SIDE`.
fn decode(bytes: &[u8]) -> Result<ColorImage> {
    let picture = image::load_from_memory(bytes).map_err(|source| AppError::Picture { source })?;
    let picture = if picture.width().max(picture.height()) > PICTURE_SIDE {
        picture.resize(
            PICTURE_SIDE,
            PICTURE_SIDE,
            image::imageops::FilterType::Triangle,
        )
    } else {
        picture
    };
    let rgba = picture.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

/// The corners of a picture of `size` pixels laid over the sheet of
/// `pattern`: in its middle, as large as fits it, its sides as they are.
fn over_sheet(pattern: &Pattern, size: [usize; 2]) -> [Vec2; 4] {
    let sheet = pattern
        .work_area()
        .unwrap_or(Bounds::of(Cell { column: 0, row: 0 }));
    let ((left, top), (right, bottom)) = pattern.grid().span(sheet, pattern.ratio());
    let room = vec2(right - left, bottom - top);
    let aspect = size[0] as f32 / size[1].max(1) as f32;
    let fit = if room.x / room.y > aspect {
        vec2(room.y * aspect, room.y)
    } else {
        vec2(room.x, room.x / aspect)
    };
    let min = vec2(left, top) + (room - fit) / 2.0;
    let max = min + fit;
    [min, vec2(max.x, min.y), max, vec2(min.x, max.y)]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG of `width` by `height` pixels of one colour.
    pub fn png(width: u32, height: u32) -> Vec<u8> {
        let picture = image::RgbaImage::from_pixel(width, height, image::Rgba([200, 60, 60, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        picture
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("a PNG is written");
        bytes.into_inner()
    }

    #[test]
    fn a_large_picture_is_drawn_smaller() {
        let image = decode(&png(PICTURE_SIDE * 2, 10)).expect("it decodes");

        assert_eq!(image.size, [PICTURE_SIDE as usize, 5]);
    }

    #[test]
    fn a_first_picture_lies_over_the_sheet_with_its_sides() {
        let mut pattern = crate::app::new_pattern();
        pattern.set_sheet(Some(Bounds {
            min: Cell { column: 0, row: 0 },
            max: Cell { column: 9, row: 9 },
        }));

        let corners = over_sheet(&pattern, [200, 100]);

        assert_eq!(corners[0], vec2(0.0, 2.5));
        assert_eq!(corners[2], vec2(10.0, 7.5));
    }
}
