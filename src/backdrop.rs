//! The picture under the pattern (`docs/backdrop.md`): kept in the store of
//! pictures of the cache, `obusina-backdrop`, by the identifier of the
//! pattern, and laid on the field by its four corners.
//!
//! The store and the decoding of a picture work away from the frame, one job
//! after another ([`crate::task`]); their answers come back in a later frame.
//! While a picture loads, a panel that shimmers stands in its place: over its
//! corners once they are read, a square in the middle of the work area until
//! then.
//!
//! ```text
//! first frame ──► find ──► none ──────────────────────► no picture
//!                      └─► the description: corners ──► read, decode ──► the picture
//! chosen file ──► a square ──► decode: corners over the work area ──► put ──► the picture
//! ```
//!
//! A change of the picture is no step of undo: the picture is not part of
//! the pattern.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use egui::{ColorImage, TextureHandle, Vec2, vec2};
use obusina_backdrop::{Backdrop as Description, Kept, Kind, Store};
use obusina_pattern::{Bounds, Cell, Pattern, PatternId};

use crate::error::{AppError, Result};
use crate::pick::Picked;
use crate::task::{MaybeSend, Tasks};

/// The longest side a picture is drawn at, in pixels: a larger one is drawn
/// smaller. The file in the store keeps all of it.
pub const PICTURE_SIDE: u32 = 4096;
/// How much of a new picture shows.
pub const DEFAULT_OPACITY: f32 = 0.6;
/// The name of the directory of pictures in the cache.
pub const DIRECTORY: &str = "backdrops";

/// What a job of the store answers.
enum Answer {
    /// The description of the picture of the pattern, or none.
    Found(Option<Kept>),
    /// The picture, decoded, and how it lies.
    Loaded(Kept, ColorImage),
    /// A job failed; a load that failed leaves no picture.
    Failed(AppError),
}

/// The span of the work area on the field, in bead units: its top left and
/// bottom right corners.
type Span = ((f32, f32), (f32, f32));

/// The picture under the pattern, and the store it is kept in.
pub struct Backdrop {
    /// The store; `None` when the platform gives no cache.
    store: Option<Store>,
    tasks: Tasks,
    answers: (Sender<Answer>, Receiver<Answer>),
    /// The picture of the pattern was asked of the store.
    asked: bool,
    /// A picture is read or decoded, or the store is asked for it.
    loading: bool,
    /// The place of the picture that loads is known: its corners, or the
    /// square that stands for it.
    placed: bool,
    /// The description of the picture, once it is known.
    kept: Option<Kept>,
    /// The picture decoded, until it is a texture.
    image: Option<ColorImage>,
    texture: Option<TextureHandle>,
    /// The corners on the field, in bead units, as they are drawn now; while
    /// a new picture is decoded, the square that stands for it.
    pub(crate) corners: [Vec2; 4],
    /// The corners before the editing of the corners began, while it goes on.
    editing: Option<[Vec2; 4]>,
    /// A picture chosen by the person loads: its corners are edited once it
    /// is decoded.
    edit_when_loaded: bool,
    /// How much of the picture shows; the slider of its menu changes it.
    pub(crate) opacity: Arc<Mutex<f32>>,
}

impl Backdrop {
    /// The picture of the store in the directory of `files`. Nothing is
    /// read until the first frame ([`Backdrop::update`]).
    pub fn new(files: Option<unifs::Files>) -> Backdrop {
        Backdrop {
            store: files.map(Store::new),
            tasks: Tasks::new(),
            answers: channel(),
            asked: false,
            loading: false,
            placed: false,
            kept: None,
            image: None,
            texture: None,
            corners: [Vec2::ZERO; 4],
            editing: None,
            edit_when_loaded: false,
            opacity: Arc::new(Mutex::new(DEFAULT_OPACITY)),
        }
    }

    /// Asks the store for the picture of `pattern` in the first frame, and
    /// takes the answers that came. What went wrong is given back.
    pub fn update(&mut self, context: &egui::Context, pattern: &Pattern) -> Option<AppError> {
        if !self.asked {
            self.asked = true;
            self.find(context, pattern);
        }
        let mut failed = None;
        while let Ok(answer) = self.answers.1.try_recv() {
            match answer {
                Answer::Found(Some(kept)) => {
                    self.corners = kept.backdrop.corners.map(|[x, y]| vec2(x, y));
                    self.placed = true;
                    self.set_opacity(kept.backdrop.opacity);
                    self.kept = Some(kept);
                }
                Answer::Found(None) => self.loading = false,
                Answer::Loaded(kept, image) => {
                    self.corners = kept.backdrop.corners.map(|[x, y]| vec2(x, y));
                    self.kept = Some(kept);
                    self.image = Some(image);
                    self.texture = None;
                    self.loading = false;
                    if std::mem::take(&mut self.edit_when_loaded) {
                        self.begin_editing();
                    }
                }
                Answer::Failed(error) => {
                    self.edit_when_loaded = false;
                    if self.loading {
                        self.kept = None;
                        self.loading = false;
                    }
                    failed = Some(error);
                }
            }
        }
        failed
    }

    /// Asks the store for the description of the picture of `pattern`, then
    /// for the picture.
    fn find(&mut self, context: &egui::Context, pattern: &Pattern) {
        let Some(store) = self.store.clone() else {
            return;
        };
        self.loading = true;
        self.placed = false;
        let id = pattern.id().to_string();
        let send = self.sender(context);
        self.tasks.run(async move {
            let kept = match store.find(&id).await {
                Ok(kept) => kept,
                Err(error) => return send(Answer::Failed(error.into())),
            };
            send(Answer::Found(kept.clone()));
            let Some(kept) = kept else {
                return;
            };
            let decoded = match store.read(&kept).await {
                Ok(bytes) => decode(&bytes),
                Err(error) => Err(error.into()),
            };
            send(match decoded {
                Ok(image) => Answer::Loaded(kept, image),
                Err(error) => Answer::Failed(error),
            });
        });
    }

    /// What a job sends its answer by: the answer goes to the window, which
    /// is asked to draw again.
    fn sender(&self, context: &egui::Context) -> impl Fn(Answer) + Send + 'static {
        let sender = self.answers.0.clone();
        let context = context.clone();
        move |answer| {
            if sender.send(answer).is_ok() {
                context.request_repaint();
            }
        }
    }

    /// True when the pattern has a picture, loaded.
    pub fn has(&self) -> bool {
        self.kept.is_some() && !self.loading
    }

    /// True while a picture is read or decoded.
    pub fn loading(&self) -> bool {
        self.loading
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

    /// The picture for the canvas, while there is one to draw: with no
    /// texture while it loads and its place is known. Its texture is made
    /// the first time it is asked once it is decoded.
    pub fn picture(&mut self, context: &egui::Context) -> Option<obusina_canvas::Picture<'_>> {
        let texture = if self.loading {
            if !self.placed || self.hidden() {
                return None;
            }
            None
        } else {
            if !self.has() || (self.hidden() && !self.editing()) {
                return None;
            }
            if let Some(image) = self.image.take() {
                self.texture =
                    Some(context.load_texture("backdrop", image, egui::TextureOptions::LINEAR));
            }
            Some(self.texture.as_ref()?.id())
        };
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
    /// one it had, whose corners it takes. Until it is decoded a square
    /// stands for a first picture in the middle of the work area; it then
    /// lies over the work area, as large as fits it, its sides as they are.
    /// Once it is decoded its corners are edited: "put back" puts them where
    /// it was laid.
    pub fn put(
        &mut self,
        context: &egui::Context,
        pattern: &Pattern,
        picked: Picked,
    ) -> Result<()> {
        let store = self.store.clone().ok_or(AppError::NoCache)?;
        Kind::of(&picked.bytes).ok_or(AppError::PictureKind)?;
        let span = span(pattern);
        let corners = self.has().then_some(self.corners);
        if corners.is_none() {
            self.corners = square(span);
            self.set_opacity(DEFAULT_OPACITY);
        }
        if let Some(kept) = &mut self.kept {
            kept.backdrop.hidden = false;
        }
        self.editing = None;
        self.edit_when_loaded = true;
        self.loading = true;
        self.placed = true;
        self.texture = None;
        let description = Description {
            pattern: pattern.id().to_string(),
            corners: [[0.0; 2]; 4],
            opacity: self.opacity(),
            hidden: false,
            source: picked.name,
        };
        let send = self.sender(context);
        self.tasks.run(async move {
            let image = match decode(&picked.bytes) {
                Ok(image) => image,
                Err(error) => return send(Answer::Failed(error)),
            };
            let corners = corners.unwrap_or_else(|| over(span, image.size));
            let description = Description {
                corners: corners.map(|corner| [corner.x, corner.y]),
                ..description
            };
            send(match store.put(&picked.bytes, description).await {
                Ok(kept) => Answer::Loaded(kept, image),
                Err(error) => Answer::Failed(error.into()),
            });
        });
        Ok(())
    }

    /// Writes how the picture lies now: its corners and its opacity.
    pub fn save(&mut self, context: &egui::Context) {
        let corners = self.corners.map(|corner| [corner.x, corner.y]);
        let opacity = self.opacity();
        if self.loading {
            return;
        }
        let Some(kept) = self.kept.as_mut() else {
            return;
        };
        let before = kept.backdrop.clone();
        kept.backdrop.corners = corners;
        kept.backdrop.opacity = opacity;
        if kept.backdrop != before {
            let kept = kept.clone();
            self.write(context, move |store| async move { store.save(&kept).await });
        }
    }

    /// Hides the picture or shows it again, and writes that.
    pub fn set_hidden(&mut self, context: &egui::Context, hidden: bool) {
        if self.loading {
            return;
        }
        let Some(kept) = self.kept.as_mut() else {
            return;
        };
        kept.backdrop.hidden = hidden;
        let kept = kept.clone();
        self.write(context, move |store| async move { store.save(&kept).await });
    }

    /// Takes the picture away, and its copy in the store.
    pub fn remove(&mut self, context: &egui::Context) {
        self.editing = None;
        self.image = None;
        self.texture = None;
        let Some(kept) = self.kept.take() else {
            return;
        };
        self.write(
            context,
            move |store| async move { store.remove(&kept).await },
        );
    }

    /// Gives the pattern `to` a copy of the picture of `from`, if it has
    /// one: the picture of a duplicate.
    pub fn copy(&self, context: &egui::Context, from: PatternId, to: PatternId) {
        self.write(context, move |store| async move {
            let Some(kept) = store.find(&from.to_string()).await? else {
                return Ok(());
            };
            let bytes = store.read(&kept).await?;
            let description = Description {
                pattern: to.to_string(),
                ..kept.backdrop
            };
            store.put(&bytes, description).await.map(|_| ())
        });
    }

    /// Takes away the pictures of patterns deleted for good.
    pub fn forget(&self, context: &egui::Context, patterns: Vec<PatternId>) {
        self.write(context, move |store| async move {
            for pattern in patterns {
                if let Some(kept) = store.find(&pattern.to_string()).await? {
                    store.remove(&kept).await?;
                }
            }
            Ok(())
        });
    }

    /// Runs a write of the store; a failure comes back as an answer.
    fn write<F, W>(&self, context: &egui::Context, work: W)
    where
        W: FnOnce(Store) -> F + MaybeSend + 'static,
        F: std::future::Future<Output = obusina_backdrop::Result<()>> + MaybeSend,
    {
        let Some(store) = self.store.clone() else {
            return;
        };
        let send = self.sender(context);
        self.tasks.run(async move {
            if let Err(error) = work(store).await {
                send(Answer::Failed(error.into()));
            }
        });
    }

    /// The queue of the jobs of the store: a test holds them back; the
    /// window waits for them when it closes.
    #[cfg(any(test, not(target_arch = "wasm32")))]
    pub fn tasks(&mut self) -> &mut Tasks {
        &mut self.tasks
    }

    /// Begins the editing of the corners.
    pub fn begin_editing(&mut self) {
        if self.has() {
            self.editing = Some(self.corners);
        }
    }

    /// Ends the editing of the corners: keeps them and writes them, or puts
    /// back those it began with.
    pub fn end_editing(&mut self, context: &egui::Context, keep: bool) {
        let Some(before) = self.editing.take() else {
            return;
        };
        if keep {
            self.save(context);
        } else {
            self.corners = before;
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

/// The span of the work area of `pattern` on the field.
fn span(pattern: &Pattern) -> Span {
    let area = pattern
        .work_area()
        .unwrap_or(Bounds::of(Cell { column: 0, row: 0 }));
    pattern.grid().span(area, pattern.ratio())
}

/// The corners of a box of the sides of `size` in the middle of `span`, as
/// large as fits it.
fn fit(((left, top), (right, bottom)): Span, size: Vec2) -> [Vec2; 4] {
    let room = vec2(right - left, bottom - top);
    let aspect = size.x / size.y.max(f32::EPSILON);
    let fit = if room.x / room.y > aspect {
        vec2(room.y * aspect, room.y)
    } else {
        vec2(room.x, room.x / aspect)
    };
    let min = vec2(left, top) + (room - fit) / 2.0;
    let max = min + fit;
    [min, vec2(max.x, min.y), max, vec2(min.x, max.y)]
}

/// The corners of a picture of `size` pixels laid over `span`: in its
/// middle, as large as fits it, its sides as they are.
fn over(span: Span, size: [usize; 2]) -> [Vec2; 4] {
    fit(span, vec2(size[0] as f32, size[1] as f32))
}

/// The square that stands for a picture whose sides are not known yet: in
/// the middle of `span`, as large as fits it.
fn square(span: Span) -> [Vec2; 4] {
    fit(span, Vec2::splat(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG of `width` by `height` pixels of one colour.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let picture = image::RgbaImage::from_pixel(width, height, image::Rgba([200, 60, 60, 255]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        picture
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("a PNG is written");
        bytes.into_inner()
    }

    const SPAN: Span = ((0.0, 0.0), (10.0, 10.0));

    #[test]
    fn a_large_picture_is_drawn_smaller() {
        let image = decode(&png(PICTURE_SIDE * 2, 10)).expect("it decodes");

        assert_eq!(image.size, [PICTURE_SIDE as usize, 5]);
    }

    #[test]
    fn a_first_picture_lies_over_the_work_area_with_its_sides() {
        let corners = over(SPAN, [200, 100]);

        assert_eq!(corners[0], vec2(0.0, 2.5));
        assert_eq!(corners[2], vec2(10.0, 7.5));
    }

    #[test]
    fn a_picture_of_unknown_sides_is_a_square_in_the_middle() {
        let corners = square(((0.0, 0.0), (10.0, 6.0)));

        assert_eq!(corners[0], vec2(2.0, 0.0));
        assert_eq!(corners[2], vec2(8.0, 6.0));
    }
}
