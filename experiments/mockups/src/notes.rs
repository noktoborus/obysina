//! Notes left on a mockup: a widget tapped or a box drawn over it, a line
//! of text, a picture of the page, posted to the receiver
//! (`src/bin/notes`).
//!
//! While the switch or the list is on, the mockup is frozen: the first
//! frame is taken as a picture and its widgets are kept. Until the
//! picture comes the mockup is still drawn, with no input and nothing of
//! the notes over it: a browser takes the picture a frame or two later.
//! From then on the mockup is not drawn and gets no event, the picture
//! stands in its place. A menu or a picker of the mockup stays as it was, and is there
//! again when the switch goes off.
//!
//! A tap takes the whole widget under it: the smallest one of the tree of
//! accesskit (`seen`) or named by the mockup (`mock::named`), lit while the
//! pointer is over it. A drag draws a box,
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
use egui::{
    Align2, Color32, ColorImage, Context, Id, Key, Modifiers, Order, Pos2, Rect, Sense, Stroke,
    vec2,
};

use crate::mock::{self, Mockup};
use crate::note::{Chat, Kept, Note, Posted, Shot, Widget};

/// Where the receiver is: the proxy of trunk passes it on.
const URL: &str = "/notes";
/// Where a comment for the chat goes.
const CHAT: &str = "/notes/chat";

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
/// The width of the form a note is written in, and of the list.
const FORM_WIDTH: f32 = 320.0;
/// The key that sends the note being written.
const SEND: egui::KeyboardShortcut = egui::KeyboardShortcut::new(Modifiers::COMMAND, Key::Enter);

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
    /// The picture of the page with a note on it.
    shot: Option<Arc<ColorImage>>,
    /// The picture of the mockup as it was frozen.
    frozen: Option<Arc<ColorImage>>,
    /// The notes the receiver keeps; `None` until it answered.
    kept: Option<Vec<Kept>>,
    /// A line for the user: what was sent, what failed.
    message: Option<String>,
}

/// The mockup as it was when the notes froze it.
struct Frozen {
    /// Its widgets: those it named, and those of the tree of accesskit of
    /// the frame its picture shows.
    widgets: Vec<Widget>,
    /// The parts it named itself (`mock::named`): the windows and menus a
    /// path of a widget names.
    named: Vec<Widget>,
    /// What it showed, from `Mockup::state`.
    state: serde_json::Value,
    /// Its picture; `None` until the picture comes.
    picture: Option<egui::TextureHandle>,
    /// Where its layers stood, back to front: a widget is seen only where
    /// no layer above its own covers it.
    layers: Vec<Rect>,
}

/// The notes of the gallery.
#[derive(Default)]
pub struct Notes {
    /// The widgets of the last frame, from the tree of accesskit.
    seen: crate::seen::Widgets,
    /// The mockup frozen while the notes are on.
    frozen: Option<Frozen>,
    /// The notes were switched on in this frame: the mockup is frozen when
    /// it has been drawn.
    freezing: bool,
    pub(crate) phase: Phase,
    start: Option<Pos2>,
    pub(crate) rect: Option<Rect>,
    /// The widget a tap took.
    pub(crate) picked: Option<Widget>,
    /// The widget under the pointer, with its path, while pointing.
    hovered: Option<Widget>,
    text: String,
    inbox: Arc<Mutex<Inbox>>,
    asked: bool,
    /// The list of the notes waiting is open.
    listing: bool,
    /// The form of a comment for the chat is open.
    chatting: bool,
    /// The comment for the chat being written.
    comment: String,
}

impl Notes {
    /// The switch of the strip, and how many notes on `mock` wait to be
    /// read.
    ///
    /// `tapped` tells whether a widget of the strip was tapped: the strip
    /// takes its presses before the mockup sees them.
    pub fn switch(
        &mut self,
        ui: &mut egui::Ui,
        mock: &str,
        tapped: impl Fn(&egui::Response) -> bool,
    ) {
        if !self.asked {
            self.asked = true;
            fetch(&self.inbox, ui.ctx());
        }
        self.take_picture(ui.ctx());
        let on = self.phase != Phase::Off;
        let label = format!(
            "\u{270f} {}",
            self.looked_at().unwrap_or_else(|| "note".to_string())
        );
        let switch = ui.selectable_label(on, label);
        if tapped(&switch) {
            self.phase = if on { Phase::Off } else { Phase::Pointing };
            self.rect = None;
            self.text.clear();
        }
        let (here, all) = {
            let inbox = self.inbox.lock().expect("the inbox is never poisoned");
            let kept = inbox.kept.as_deref().unwrap_or_default();
            let here = kept.iter().filter(|note| note.note.mock == mock).count();
            (here, kept.len())
        };
        let label = format!("\u{2630} waiting: {here} of {all}");
        if tapped(&ui.selectable_label(self.listing, label)) {
            self.listing = !self.listing;
        }
        if tapped(&ui.selectable_label(self.chatting, "\u{2709} to chat")) {
            self.chatting = !self.chatting;
        }
        let active = self.phase != Phase::Off || self.listing || self.chatting;
        match (active, self.frozen.is_some() || self.freezing) {
            (true, false) => self.freezing = true,
            (false, true) => {
                self.frozen = None;
                self.freezing = false;
            }
            _ => {}
        }
        let inbox = self.inbox.lock().expect("the inbox is never poisoned");
        if let Some(message) = &inbox.message {
            ui.label(message);
        }
    }

    /// The plugin that keeps the widgets of each frame for the notes; it is
    /// added to the context once.
    pub fn plugin(&self) -> crate::seen::Seen {
        crate::seen::Seen(Arc::clone(&self.seen))
    }

    /// True while the mockup is frozen: it is not drawn, its picture is.
    pub fn frozen(&self) -> bool {
        self.frozen
            .as_ref()
            .is_some_and(|frozen| frozen.picture.is_some())
    }

    /// True while the picture of the mockup is awaited: the mockup is drawn
    /// as it was, and no input may reach it.
    pub fn holding(&self) -> bool {
        self.freezing
            || self
                .frozen
                .as_ref()
                .is_some_and(|frozen| frozen.picture.is_none())
    }

    /// Turns the notes off, as when another mockup is shown.
    pub fn off(&mut self) {
        self.phase = Phase::Off;
        self.listing = false;
        self.chatting = false;
        self.rect = None;
        self.frozen = None;
        self.freezing = false;
    }

    /// Draws the picture of the frozen mockup over the whole window, as it
    /// was taken; `ui` clips it to the space of the mockup.
    pub fn paint_frozen(&self, ui: &egui::Ui) {
        if let Some(picture) = self
            .frozen
            .as_ref()
            .and_then(|frozen| frozen.picture.as_ref())
        {
            let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
            ui.painter()
                .image(picture.id(), ui.ctx().content_rect(), uv, Color32::WHITE);
        }
    }

    /// Draws the notes over `area`, the space of the mockup, and takes the
    /// presses there while the switch is on.
    pub fn show(&mut self, context: &Context, area: Rect, mock: &dyn Mockup) {
        if self.freezing {
            self.freeze(context, mock);
        }
        if self.holding() {
            context.request_repaint();
            return;
        }
        if self.listing {
            self.list(context, area, mock.name());
        }
        if self.chatting {
            self.chat_form(context, area);
        }
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
                    .and_then(|pointer| self.widget_at(pointer))
                    .cloned();
                self.point(&response, under.as_ref());
                self.hovered = under.as_ref().map(|widget| self.with_path(widget));
                let under = under.as_ref().map(bounds);
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
            Phase::Capturing => self.capture(context, mock.name()),
            _ => {}
        }
    }

    /// Takes the picture of the frozen mockup, when it has come.
    fn take_picture(&mut self, context: &Context) {
        let Some(frozen) = self
            .frozen
            .as_mut()
            .filter(|frozen| frozen.picture.is_none())
        else {
            return;
        };
        let image = self
            .inbox
            .lock()
            .expect("the inbox is never poisoned")
            .frozen
            .take();
        if let Some(image) = image {
            let seen = self.seen.lock().expect("the widgets are never poisoned");
            for widget in seen.iter() {
                if !frozen.widgets.contains(widget) {
                    frozen.widgets.push(widget.clone());
                }
            }
            frozen.layers = layers(context);
            frozen.picture = Some(context.load_texture(
                "mockups-frozen",
                Arc::unwrap_or_clone(image),
                egui::TextureOptions::NEAREST,
            ));
        }
    }

    /// Keeps the widgets and the state of the mockup drawn in this frame,
    /// and asks for the picture of the frame.
    fn freeze(&mut self, context: &Context, mock: &dyn Mockup) {
        self.freezing = false;
        self.frozen = Some(Frozen {
            widgets: mock::named_widgets(context),
            named: mock::named_widgets(context),
            state: mock.state(),
            picture: None,
            layers: Vec::new(),
        });
        let inbox = Arc::clone(&self.inbox);
        let repaint = context.clone();
        context.send_viewport_cmd(egui::ViewportCommand::Screenshot(
            egui::ScreenshotCallback::new(move |image| {
                inbox.lock().expect("the inbox is never poisoned").frozen = Some(image);
                repaint.request_repaint();
            }),
        ));
        context.request_repaint();
    }

    /// The widgets of the frozen mockup.
    fn widgets(&self) -> &[Widget] {
        self.frozen.as_ref().map_or(&[], |frozen| &frozen.widgets)
    }

    /// The smallest widget seen under `pointer`: a button rather than the
    /// window it stands in, and not one a layer above it covers.
    fn widget_at(&self, pointer: Pos2) -> Option<&Widget> {
        self.widgets()
            .iter()
            .filter(|widget| bounds(widget).contains(pointer) && self.seen_at(widget, pointer))
            .min_by(|first, second| bounds(first).area().total_cmp(&bounds(second).area()))
    }

    /// True when `widget` is not covered at `point`: it lies in the top
    /// layer there.
    fn seen_at(&self, widget: &Widget, point: Pos2) -> bool {
        let layers = self
            .frozen
            .as_ref()
            .map_or(&[][..], |frozen| &frozen.layers);
        layers
            .iter()
            .rev()
            .find(|layer| layer.contains(point))
            .is_none_or(|top| top.expand(1.0).contains_rect(bounds(widget)))
    }

    /// The widgets a note on `rect` names: the one tapped, or those seen
    /// whole inside the box drawn.
    fn widgets_of(&self, rect: Rect) -> Vec<Widget> {
        if rect.area() == 0.0
            || self
                .picked
                .as_ref()
                .is_some_and(|picked| bounds(picked) == rect)
        {
            return self.picked.iter().cloned().collect();
        }
        self.widgets()
            .iter()
            .filter(|widget| {
                let bounds = bounds(widget);
                rect.expand(1.0).contains_rect(bounds) && self.seen_at(widget, bounds.center())
            })
            .map(|widget| self.with_path(widget))
            .collect()
    }

    /// What the note is about, as the strip and the form name it: the
    /// widget under the pointer while pointing, the widget taken, the
    /// widgets of a box, or a point. `None` while there is nothing.
    fn looked_at(&self) -> Option<String> {
        let rect = self.rect;
        match (self.phase, rect) {
            (Phase::Off, _) => None,
            (Phase::Pointing, _) => self.hovered.as_ref().map(identity),
            (_, Some(rect)) if rect.area() == 0.0 => {
                Some(format!("({:.0}, {:.0})", rect.min.x, rect.min.y))
            }
            (_, Some(rect)) => match self.widgets_of(rect).as_slice() {
                [widget] => Some(identity(widget)),
                widgets => Some(format!("{} widgets in the box", widgets.len())),
            },
            (_, None) => None,
        }
        .map(|text| shorten(&text))
    }

    /// The widget with its whole path: the names of the tree of accesskit,
    /// then the parts the mockup named that hold it in the same layer,
    /// the largest first.
    fn with_path(&self, widget: &Widget) -> Widget {
        let inner = bounds(widget);
        let named = self.frozen.as_ref().map_or(&[][..], |frozen| &frozen.named);
        let mut holders: Vec<&Widget> = named
            .iter()
            .filter(|holder| {
                let outer = bounds(holder);
                *holder != widget
                    && outer.expand(1.0).contains_rect(inner)
                    && self.seen_at(holder, inner.center())
            })
            .collect();
        holders.sort_by(|first, second| bounds(second).area().total_cmp(&bounds(first).area()));
        let mut path = widget.path.clone();
        path.extend(holders.into_iter().map(|holder| holder.name.clone()));
        Widget {
            path,
            ..widget.clone()
        }
    }

    /// A tap takes the widget `under` the pointer, or a point where there is
    /// none; a drag draws the box.
    fn point(&mut self, response: &egui::Response, under: Option<&Widget>) {
        if self.phase != Phase::Pointing {
            return;
        }
        let pointer = response.interact_pointer_pos();
        if response.drag_started() {
            self.start = pointer;
            self.picked = None;
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
            self.rect = Some(under.map_or(Rect::from_min_max(pointer, pointer), bounds));
            self.picked = under.map(|widget| self.with_path(widget));
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

    /// The list of the notes waiting, at the right under the strip: the
    /// box of a note is lit while its line is under the pointer, and the
    /// cross deletes it.
    fn list(&mut self, context: &Context, area: Rect, mock: &str) {
        let kept = self
            .inbox
            .lock()
            .expect("the inbox is never poisoned")
            .kept
            .clone()
            .unwrap_or_default();
        let mut lit = None;
        egui::Area::new(Id::unique("mockups-note-list"))
            .order(Order::Tooltip)
            .fixed_pos(area.right_top() + vec2(-FORM_WIDTH - 8.0, 8.0))
            .constrain(true)
            .show(context, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_width(FORM_WIDTH);
                    if kept.is_empty() {
                        ui.label("no notes wait");
                    }
                    egui::ScrollArea::vertical()
                        .max_height(area.height() / 2.0)
                        .show(ui, |ui| {
                            for note in &kept {
                                let line = ui.horizontal(|ui| {
                                    if ui
                                        .small_button("\u{2716}")
                                        .on_hover_text("delete")
                                        .clicked()
                                    {
                                        remove(&self.inbox, context, note.id);
                                    }
                                    ui.add(
                                        egui::Label::new(format!(
                                            "#{} {}: {}",
                                            note.id,
                                            note.note.mock,
                                            note.note.text.trim()
                                        ))
                                        .wrap(),
                                    );
                                });
                                if line.response.contains_pointer() && note.note.mock == mock {
                                    lit = Some(note.note.rect);
                                }
                            }
                        });
                });
            });
        if let Some([left, top, right, bottom]) = lit {
            let rect = Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom));
            context
                .layer_painter(egui::LayerId::new(
                    Order::Tooltip,
                    Id::unique("mockups-note-lit"),
                ))
                .rect_stroke(rect.expand(2.0), 2.0, DRAWN, egui::StrokeKind::Outside);
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
        let title = self.looked_at();
        match text_form(
            context,
            "mockups-note-form",
            at,
            title.as_deref(),
            &mut self.text,
            "what is wrong here",
        ) {
            Some(Answer::Send) => {
                self.phase = Phase::Capturing;
                self.ask_picture(context);
            }
            Some(Answer::Cancel) => {
                self.phase = Phase::Pointing;
                self.rect = None;
            }
            None => {}
        }
    }

    /// The form of a comment for the chat, under the strip at the right.
    fn chat_form(&mut self, context: &Context, area: Rect) {
        let at = area.right_top() + vec2(-FORM_WIDTH - 8.0, 8.0);
        let hint = "a comment for the chat";
        match text_form(
            context,
            "mockups-chat-form",
            at,
            None,
            &mut self.comment,
            hint,
        ) {
            Some(Answer::Send) => {
                let chat = Chat {
                    comment: std::mem::take(&mut self.comment),
                };
                match serde_json::to_vec(&chat) {
                    Ok(body) => send(&self.inbox, context, CHAT, body, "sent to the chat"),
                    Err(error) => self.say(&format!("the comment does not encode: {error}")),
                }
                self.chatting = false;
            }
            Some(Answer::Cancel) => self.chatting = false,
            None => {}
        }
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
    fn capture(&mut self, context: &Context, mock: &str) {
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
        let widgets = self.widgets_of(rect);
        let state = self
            .frozen
            .as_ref()
            .map_or(serde_json::Value::Null, |frozen| frozen.state.clone());
        let window = context.content_rect().size();
        let posted = Posted {
            note: Note {
                mock: mock.to_string(),
                text: std::mem::take(&mut self.text),
                rect: [rect.min.x, rect.min.y, rect.max.x, rect.max.y],
                window: [window.x, window.y],
                pixels_per_point: context.pixels_per_point(),
                widgets,
                state,
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
        send(&self.inbox, context, URL, body, "sent");
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

/// What was done in a form of text.
enum Answer {
    /// Sent, by the button or ctrl+enter.
    Send,
    /// Dropped, by the button or Esc.
    Cancel,
}

/// A form of text at `at`, above everything: the `title`, the text, send and
/// cancel. Text of nothing but spaces is not sent.
fn text_form(
    context: &Context,
    id: &'static str,
    at: Pos2,
    title: Option<&str>,
    text: &mut String,
    hint: &str,
) -> Option<Answer> {
    let mut answer = None;
    egui::Area::new(Id::unique(id))
        .order(Order::Debug)
        .fixed_pos(at)
        .constrain(true)
        .show(context, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(FORM_WIDTH);
                if let Some(title) = title {
                    ui.add(egui::Label::new(egui::RichText::new(title).strong()).wrap());
                }
                let send = ui.input_mut(|input| input.consume_shortcut(&SEND));
                let cancel = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape));
                let edit = ui.add(
                    egui::TextEdit::multiline(text)
                        .hint_text(hint)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                if !edit.has_focus() && text.is_empty() {
                    edit.request_focus();
                }
                ui.horizontal(|ui| {
                    let pressed = ui.button("send").on_hover_text("ctrl+enter").clicked();
                    if (pressed || send) && !text.trim().is_empty() {
                        answer = Some(Answer::Send);
                    }
                    if ui.button("cancel").on_hover_text("esc").clicked() || cancel {
                        answer = Some(Answer::Cancel);
                    }
                });
            });
        });
    answer
}

/// The longest name of a widget the strip shows, in characters.
const LONGEST: usize = 60;

/// What tells a widget apart: its path and its name.
fn identity(widget: &Widget) -> String {
    let mut path = widget.path.clone();
    path.push(widget.name.clone());
    path.join(" \u{203a} ")
}

/// A text cut to [`LONGEST`] characters, its end kept: the end is the
/// widget itself.
fn shorten(text: &str) -> String {
    let count = text.chars().count();
    if count <= LONGEST {
        return text.to_string();
    }
    let tail: String = text.chars().skip(count - LONGEST + 1).collect();
    format!("\u{2026}{tail}")
}

/// The rects of the visible layers of the last frame, back to front, but
/// the strip of the gallery, which is no part of a mockup.
fn layers(context: &Context) -> Vec<Rect> {
    let strip = Id::unique("mockups-strip");
    context.memory(|memory| {
        let visible = memory.areas().visible_layer_ids();
        memory
            .layer_ids()
            .filter(|layer| visible.contains(layer) && layer.id != strip)
            .filter_map(|layer| memory.area_rect(layer.id))
            .collect()
    })
}

/// Where a named widget stands.
fn bounds(widget: &Widget) -> Rect {
    let [left, top, right, bottom] = widget.rect;
    Rect::from_min_max(Pos2::new(left, top), Pos2::new(right, bottom))
}

/// Posts the body of a note; the answer sets the message and asks for the
/// notes again.
#[cfg(target_arch = "wasm32")]
fn send(inbox: &Arc<Mutex<Inbox>>, context: &Context, url: &str, body: Vec<u8>, done: &str) {
    let inbox = Arc::clone(inbox);
    let repaint = context.clone();
    let done = done.to_string();
    ehttp::fetch(ehttp::Request::post(url, body), move |result| {
        let message = match result {
            Ok(response) if response.ok => done,
            Ok(response) => format!("the receiver said {}", response.status),
            Err(error) => format!("no receiver: {error}"),
        };
        inbox.lock().expect("the inbox is never poisoned").message = Some(message);
        fetch(&inbox, &repaint);
    });
}

/// Deletes a note of the receiver, then asks for the notes again.
#[cfg(target_arch = "wasm32")]
fn remove(inbox: &Arc<Mutex<Inbox>>, context: &Context, id: u32) {
    let inbox = Arc::clone(inbox);
    let repaint = context.clone();
    let request = ehttp::Request::new(ehttp::Method::DELETE, format!("{URL}/{id}"), &[]);
    ehttp::fetch(request, move |result| {
        if !result.is_ok_and(|response| response.ok) {
            inbox.lock().expect("the inbox is never poisoned").message =
                Some(format!("note {id} is not deleted"));
        }
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
fn send(inbox: &Arc<Mutex<Inbox>>, _context: &Context, _url: &str, _body: Vec<u8>, _done: &str) {
    inbox.lock().expect("the inbox is never poisoned").message =
        Some("notes are sent from the page only".to_string());
}

/// A window of the desktop deletes no note.
#[cfg(not(target_arch = "wasm32"))]
fn remove(_inbox: &Arc<Mutex<Inbox>>, _context: &Context, _id: u32) {}

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
            .build_eframe(|_| Gallery::new("#palette-move"));
        harness.run();
        let swatch = harness.get_by_label("3").rect();
        harness.get_by_label_contains("\u{270f}").click();
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

    /// A press of the pointer at `point`, let go in place, through the
    /// filter of `Gallery::raw_input_hook`: the harness calls no hook.
    fn press(harness: &mut Harness<'_, Gallery>, point: egui::Pos2) {
        harness.event(egui::Event::PointerMoved(point));
        for pressed in [true, false] {
            let mut events = vec![egui::Event::PointerButton {
                pos: point,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            }];
            harness.state_mut().take_strip_presses(&mut events);
            for event in events {
                harness.event(event);
            }
            harness.step();
        }
        harness.run();
    }

    /// The notes switched on and off by the pointer leave a menu of the
    /// mockup open: the press on the strip never reaches it, and the frozen
    /// mockup gets no event.
    #[test]
    fn the_notes_leave_a_menu_of_the_mockup_open() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_eframe(|_| Gallery::new("#palette-move"));
        harness.run();
        harness.get_by_label("3").click();
        harness.run();
        assert!(
            harness.query_by_label("Удалить").is_some(),
            "the menu is open"
        );
        let switch = harness.get_by_label_contains("\u{270f}").rect().center();

        press(&mut harness, switch);
        assert!(harness.state().notes.frozen());
        assert!(
            harness.query_by_label("Удалить").is_none(),
            "a frozen mockup is not drawn"
        );

        press(&mut harness, switch);
        assert!(!harness.state().notes.frozen());

        assert!(
            harness.query_by_label("Удалить").is_some(),
            "the menu is open again"
        );
    }

    /// A tap in a note takes an entry of a `PlateMenu` of the mockup: the
    /// widgets come from the tree of accesskit, not only from the mockup.
    #[test]
    fn a_tap_takes_an_entry_of_a_menu() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_eframe(|_| Gallery::new("#palette-move"));
        harness.run();
        harness.get_by_label("3").click();
        harness.run();
        let entry = harness.get_by_label("Удалить").rect();
        let switch = harness.get_by_label_contains("\u{270f}").rect().center();
        press(&mut harness, switch);

        press(&mut harness, entry.center());

        let notes = &harness.state().notes;
        assert_eq!(notes.rect, Some(entry));
    }

    /// A tap where an entry of the menu covers a colour takes the entry: a
    /// widget a layer above covers is not seen. The note names it alone,
    /// with the path of what it stands in.
    #[test]
    fn a_covered_widget_is_not_taken() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(800.0, 500.0))
            .build_eframe(|_| Gallery::new("#palette-move"));
        harness.run();
        harness.get_by_label("3").click();
        harness.run();
        let entry = harness.get_by_label("Выбрать цвет").rect();
        let covered = ["1", "2", "3", "4", "5"]
            .iter()
            .map(|key| harness.get_by_label(key).rect())
            .find_map(|swatch| {
                let both = swatch.intersect(entry);
                both.is_positive().then_some(both.center())
            })
            .expect("the menu covers a colour");
        let switch = harness.get_by_label_contains("\u{270f}").rect().center();
        press(&mut harness, switch);

        press(&mut harness, covered);

        let notes = &harness.state().notes;
        let picked = notes.picked.as_ref().expect("a widget is taken");
        assert_eq!(picked.name, "Выбрать цвет");
        assert_eq!(picked.path, ["menu of colour 3"]);
        let named = "menu of colour 3 \u{203a} Выбрать цвет";
        assert!(
            harness
                .query_by_label(&format!("\u{270f} {named}"))
                .is_some(),
            "the strip names it"
        );
        assert!(harness.query_by_label(named).is_some(), "the form names it");

        assert_eq!(notes.widgets_of(entry), std::slice::from_ref(picked));
    }
}
