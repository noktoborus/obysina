//! Notes left on a mockup: a widget tapped or a box drawn over it, a line
//! of text, a picture of the page, posted to the receiver
//! (`src/bin/notes`).
//!
//! A tap takes the whole widget under it: the smallest one the mockup named
//! (`mock::named`), lit while the pointer is over it. A drag draws a box,
//! for a place between widgets or several of them.
//!
//! ```text
//! off ──switch──► pointing ──tap or drag──► writing ──send──► capturing
//!                    ▲  ▲                      │                  │ picture
//!                    │  └────────cancel────────┘                  ▼
//!                    └──────────────answer of the receiver──── sending
//! ```

use std::sync::{Arc, Mutex};

use base64::Engine;
use egui::{Align2, Color32, ColorImage, Context, Id, Order, Pos2, Rect, Sense, Stroke, vec2};

use crate::mock::{self, Mockup};
use crate::note::{Kept, Note, Posted, Shot};

/// Where the receiver is: the proxy of trunk passes it on.
#[cfg(target_arch = "wasm32")]
const URL: &str = "/notes";

/// The box being drawn.
const DRAWN: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(0xff, 0x40, 0x40),
};
/// The widget under the pointer, which a tap takes.
const LIT: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(0x40, 0xc0, 0xff),
};
/// The box of a note sent and not read yet.
const WAITING: Stroke = Stroke {
    width: 1.5,
    color: Color32::from_rgb(0xff, 0xa0, 0x20),
};
/// The width of the form a note is written in.
const FORM_WIDTH: f32 = 320.0;

/// What the notes are doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Phase {
    /// The switch is off; the mockup takes every press.
    #[default]
    Off,
    /// A drag draws a box, a tap marks a point.
    Pointing,
    /// The text is written under the box.
    Writing,
    /// The picture of the page is asked for.
    Capturing,
    /// The note is on its way.
    Sending,
}

/// What comes back from another thread or the browser.
#[derive(Default)]
struct Inbox {
    /// The picture of the page.
    shot: Option<Arc<ColorImage>>,
    /// The notes the receiver keeps; `None` until it answered.
    kept: Option<Vec<Kept>>,
    /// A line for the user: what was sent, what failed.
    message: Option<String>,
}

/// The notes of the gallery.
#[derive(Default)]
pub struct Notes {
    pub(crate) phase: Phase,
    start: Option<Pos2>,
    pub(crate) rect: Option<Rect>,
    text: String,
    inbox: Arc<Mutex<Inbox>>,
    asked: bool,
}

impl Notes {
    /// The switch of the strip, and how many notes on `mock` wait to be
    /// read.
    pub fn switch(&mut self, ui: &mut egui::Ui, mock: &str) {
        if !self.asked {
            self.asked = true;
            fetch(&self.inbox, ui.ctx());
        }
        let mut on = self.phase != Phase::Off;
        if ui.toggle_value(&mut on, "\u{270f} note").clicked() {
            self.phase = if on { Phase::Pointing } else { Phase::Off };
            self.rect = None;
        }
        let inbox = self.inbox.lock().expect("the inbox is never poisoned");
        if let Some(kept) = &inbox.kept {
            let waiting = kept.iter().filter(|note| note.note.mock == mock).count();
            ui.label(format!("waiting: {waiting}"));
        }
        if let Some(message) = &inbox.message {
            ui.label(message);
        }
    }

    /// Draws the notes over `area`, the space of the mockup, and takes the
    /// presses there while the switch is on.
    pub fn show(&mut self, context: &Context, area: Rect, mock: &dyn Mockup) {
        if self.phase == Phase::Off {
            return;
        }
        egui::Area::new(Id::unique("mockups-notes"))
            .order(Order::Debug)
            .fixed_pos(area.min)
            .show(context, |ui| {
                let (_, response) = ui.allocate_exact_size(area.size(), Sense::click_and_drag());
                let under = response
                    .hover_pos()
                    .and_then(|pointer| widget_at(context, pointer));
                self.point(&response, under);
                let painter = ui.painter();
                if let (Phase::Pointing, None, Some(widget)) = (self.phase, self.start, under) {
                    painter.rect_stroke(widget, 2.0, LIT, egui::StrokeKind::Outside);
                }
                if self.phase != Phase::Capturing {
                    self.paint_waiting(painter, mock.name());
                }
                if let Some(rect) = self.rect {
                    painter.rect_stroke(rect.expand(2.0), 2.0, DRAWN, egui::StrokeKind::Outside);
                }
            });
        match self.phase {
            Phase::Writing => self.form(context, area),
            Phase::Capturing => self.capture(context, mock),
            _ => {}
        }
    }

    /// A tap takes the widget `under` the pointer, or a point where there is
    /// none; a drag draws the box.
    fn point(&mut self, response: &egui::Response, under: Option<Rect>) {
        if self.phase != Phase::Pointing {
            return;
        }
        let pointer = response.interact_pointer_pos();
        if response.drag_started() {
            self.start = pointer;
        }
        if let (Some(start), Some(pointer)) = (self.start, pointer)
            && response.dragged()
        {
            self.rect = Some(Rect::from_two_pos(start, pointer));
        }
        if response.drag_stopped() {
            self.start = None;
            if self.rect.is_some() {
                self.phase = Phase::Writing;
            }
        }
        if response.clicked()
            && let Some(pointer) = pointer
        {
            self.rect = Some(under.unwrap_or(Rect::from_min_max(pointer, pointer)));
            self.phase = Phase::Writing;
        }
    }

    /// The boxes of the notes on `mock` the receiver still keeps.
    fn paint_waiting(&self, painter: &egui::Painter, mock: &str) {
        let inbox = self.inbox.lock().expect("the inbox is never poisoned");
        for kept in inbox
            .kept
            .iter()
            .flatten()
            .filter(|kept| kept.note.mock == mock)
        {
            let [left, top, right, bottom] = kept.note.rect;
            let rect = Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom));
            painter.rect_stroke(rect.expand(2.0), 2.0, WAITING, egui::StrokeKind::Outside);
            painter.text(
                rect.left_top() - vec2(0.0, 4.0),
                Align2::LEFT_BOTTOM,
                format!("#{}", kept.id),
                egui::FontId::proportional(12.0),
                WAITING.color,
            );
        }
    }

    /// The form under the box: the text, send and cancel.
    fn form(&mut self, context: &Context, area: Rect) {
        let Some(rect) = self.rect else {
            return;
        };
        let x = rect
            .left()
            .clamp(area.left(), (area.right() - FORM_WIDTH).max(area.left()));
        let at = Pos2::new(x, rect.bottom() + 8.0);
        egui::Area::new(Id::unique("mockups-note-form"))
            .order(Order::Debug)
            .fixed_pos(at)
            .constrain(true)
            .show(context, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(FORM_WIDTH);
                    let edit = ui.add(
                        egui::TextEdit::multiline(&mut self.text)
                            .hint_text("what is wrong here")
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );
                    if !edit.has_focus() && self.text.is_empty() {
                        edit.request_focus();
                    }
                    ui.horizontal(|ui| {
                        if ui.button("send").clicked() && !self.text.trim().is_empty() {
                            self.phase = Phase::Capturing;
                            self.ask_picture(context);
                        }
                        if ui.button("cancel").clicked() {
                            self.phase = Phase::Pointing;
                            self.rect = None;
                        }
                    });
                });
            });
    }

    /// Asks for a picture of the next frame, which has the box and no form.
    fn ask_picture(&self, context: &Context) {
        let inbox = Arc::clone(&self.inbox);
        let repaint = context.clone();
        context.send_viewport_cmd(egui::ViewportCommand::Screenshot(
            egui::ScreenshotCallback::new(move |image| {
                inbox.lock().expect("the inbox is never poisoned").shot = Some(image);
                repaint.request_repaint();
            }),
        ));
        context.request_repaint();
    }

    /// Waits for the picture, then posts the note with it.
    fn capture(&mut self, context: &Context, mock: &dyn Mockup) {
        let shot = self
            .inbox
            .lock()
            .expect("the inbox is never poisoned")
            .shot
            .take();
        let Some(image) = shot else {
            context.request_repaint();
            return;
        };
        let Some(rect) = self.rect else {
            return;
        };
        let widgets = mock::named_widgets(context)
            .into_iter()
            .filter(|widget| {
                let bounds = bounds(widget);
                bounds.intersects(rect) || bounds.contains(rect.min)
            })
            .collect();
        let window = context.content_rect().size();
        let posted = Posted {
            note: Note {
                mock: mock.name().to_string(),
                text: std::mem::take(&mut self.text),
                rect: [rect.min.x, rect.min.y, rect.max.x, rect.max.y],
                window: [window.x, window.y],
                pixels_per_point: context.pixels_per_point(),
                widgets,
                state: mock.state(),
            },
            shot: Some(Shot {
                size: image.size,
                rgba: base64::engine::general_purpose::STANDARD.encode(image.as_raw()),
            }),
        };
        self.phase = Phase::Sending;
        self.post(context, &posted);
    }

    /// Posts a note; the answer sets the message and asks for the notes
    /// again.
    fn post(&mut self, context: &Context, posted: &Posted) {
        let body = match serde_json::to_vec(posted) {
            Ok(body) => body,
            Err(error) => {
                self.say(&format!("the note does not encode: {error}"));
                self.phase = Phase::Writing;
                return;
            }
        };
        send(&self.inbox, context, body);
        self.rect = None;
        self.phase = Phase::Pointing;
    }

    /// Sets the line for the user.
    fn say(&self, message: &str) {
        self.inbox
            .lock()
            .expect("the inbox is never poisoned")
            .message = Some(message.to_string());
    }
}

/// Where a named widget stands.
fn bounds(widget: &crate::note::Widget) -> Rect {
    let [left, top, right, bottom] = widget.rect;
    Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom))
}

/// The smallest named widget under `pointer`: a button rather than the
/// window it stands in.
fn widget_at(context: &Context, pointer: Pos2) -> Option<Rect> {
    mock::named_widgets(context)
        .iter()
        .map(bounds)
        .filter(|rect| rect.contains(pointer))
        .min_by(|first, second| first.area().total_cmp(&second.area()))
}

/// Posts the body of a note; the answer sets the message and asks for the
/// notes again.
#[cfg(target_arch = "wasm32")]
fn send(inbox: &Arc<Mutex<Inbox>>, context: &Context, body: Vec<u8>) {
    let inbox = Arc::clone(inbox);
    let repaint = context.clone();
    ehttp::fetch(ehttp::Request::post(URL, body), move |result| {
        let message = match result {
            Ok(response) if response.ok => "sent".to_string(),
            Ok(response) => format!("the receiver said {}", response.status),
            Err(error) => format!("no receiver: {error}"),
        };
        inbox.lock().expect("the inbox is never poisoned").message = Some(message);
        fetch(&inbox, &repaint);
    });
}

/// Asks the receiver for the notes it keeps.
#[cfg(target_arch = "wasm32")]
fn fetch(inbox: &Arc<Mutex<Inbox>>, context: &Context) {
    let inbox = Arc::clone(inbox);
    let repaint = context.clone();
    ehttp::fetch(ehttp::Request::get(URL), move |result| {
        let kept = result
            .ok()
            .filter(|response| response.ok)
            .and_then(|response| serde_json::from_slice::<Vec<Kept>>(&response.bytes).ok());
        if let Some(kept) = kept {
            inbox.lock().expect("the inbox is never poisoned").kept = Some(kept);
            repaint.request_repaint();
        }
    });
}

/// A window of the desktop sends no note: the notes go from the page,
/// whose server passes them to the receiver.
#[cfg(not(target_arch = "wasm32"))]
fn send(inbox: &Arc<Mutex<Inbox>>, _context: &Context, _body: Vec<u8>) {
    inbox.lock().expect("the inbox is never poisoned").message =
        Some("notes are sent from the page only".to_string());
}

/// A window of the desktop asks for no note.
#[cfg(not(target_arch = "wasm32"))]
fn fetch(_inbox: &Arc<Mutex<Inbox>>, _context: &Context) {}

#[cfg(test)]
mod tests {
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;

    use crate::gallery::Gallery;

    /// A tap in a note takes the whole widget under it, not the point.
    #[test]
    fn a_tap_takes_the_whole_widget() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_eframe(|_| Gallery::new("#palette-editor"));
        harness.run();
        let swatch = harness.get_by_label("3").rect();
        harness.get_by_label("\u{270f} note").click();
        harness.run();

        let inside = swatch.left_top() + egui::vec2(3.0, 3.0);
        harness.event(egui::Event::PointerMoved(inside));
        harness.run();
        for pressed in [true, false] {
            harness.event(egui::Event::PointerButton {
                pos: inside,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            harness.step();
        }
        harness.run();

        let notes = &harness.state().notes;
        assert_eq!(notes.rect, Some(swatch));
        assert_eq!(notes.phase, super::Phase::Writing);
    }
}
