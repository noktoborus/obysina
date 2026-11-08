#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod edge;
mod hold;
mod placement;

pub use edge::{Edge, HYSTERESIS, Orientation, SNAP, SNAP_PART};
pub use placement::{MARGIN, Placement, room};

use egui::{Context, Id, Rect, Ui, Vec2, vec2};

use hold::{Hold, Step};

/// How much lighter the ground of a panel that is held is, from 0 to 1.
const LIFT_LIGHT: f32 = 0.12;
/// The shadow of a panel that is held: how far it falls and how soft it is.
const LIFT_SHADOW: egui::Shadow = egui::Shadow {
    offset: [0, 8],
    blur: 24,
    spread: 2,
    color: egui::Color32::from_black_alpha(120),
};

/// The room the content of a panel has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// How the content is laid out.
    pub orientation: Orientation,
    /// The length the content may take along its orientation before the
    /// window ends. A line longer than this goes on in the next line; the
    /// panel does not reflow its content.
    pub length: f32,
}

/// What a panel remembers from frame to frame.
#[derive(Debug, Clone, Copy)]
struct State {
    /// Where it stands.
    placement: Placement,
    /// The size as drawn in the last frame.
    size: Vec2,
    /// The press on the panel, while it lasts.
    hold: Option<Hold>,
    /// The content changed its shape: the next frame measures the panel
    /// again instead of laying the content out in the size it had.
    measure: bool,
    /// The key of the shape of the content the panel was measured for.
    content: u64,
    /// How the content was laid out in the last frame.
    orientation: Orientation,
}

/// What the panel did in this frame.
pub struct PanelResponse<R> {
    /// What the content gave back.
    pub inner: R,
    /// The whole panel.
    pub rect: Rect,
    /// Where the panel stands now.
    pub placement: Placement,
    /// The panel was let go after a move in this frame: `placement` is one
    /// the caller may keep, to give back at the next start.
    pub changed: bool,
    /// The panel was taken in this frame: a finger held still on it long
    /// enough, or the mouse moved with its button held. The caller may
    /// answer with a short vibration.
    pub taken: bool,
    /// The edge strip the pointer is in while the panel is taken: the edge
    /// it is pulled to. The caller may turn the panel to it.
    pub edge: Option<Edge>,
}

/// A floating panel with no title and no grip: any place of it moves it.
/// Its content stands in the middle across the panel.
///
/// ```text
/// mouse   press ──moved past the click──► taken, follows the pointer
/// finger  press ──held still a long touch──► taken, follows the finger
///         press ──moved before that──► nothing, the touch is the content's
/// taken   ──let go──► stands where it was let go; in an edge strip,
///                     against that edge
/// ```
///
/// A panel never turns by itself: the caller gives its orientation, and may
/// turn it to [`PanelResponse::edge`] while it is taken. Turned while it is
/// held, the point taken stays under the pointer, the same part of the panel
/// along and across it. While it is taken it stands whole in the window.
pub struct FloatingPanel {
    id: Id,
    name: String,
    placement: Placement,
    orientation: Orientation,
    content: u64,
}

impl FloatingPanel {
    /// A panel; `name` is what a screen reader reads for it.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, name: impl Into<String>) -> Self {
        FloatingPanel {
            id: Id::unique(("floating-panel", id_salt)),
            name: name.into(),
            placement: Placement::new(Vec2::ZERO),
            orientation: Orientation::Horizontal,
            content: 0,
        }
    }

    /// The key of the shape of the content: when it differs from the one of
    /// the last frame, the panel measures itself again. A panel otherwise keeps
    /// the size it had, and a content that got smaller would stand in it.
    pub fn content(mut self, key: impl std::hash::Hash) -> Self {
        use std::hash::Hasher as _;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        self.content = hasher.finish();
        self
    }

    /// Where the panel stands. The caller keeps the place: the panel stands
    /// where it is given, but while it is held, and gives back the place it
    /// was let go at ([`PanelResponse::changed`]).
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// How the content is laid out.
    pub fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Shows the panel; `add_contents` lays out the content in the room it is
    /// given.
    pub fn show<R>(
        self,
        context: &Context,
        add_contents: impl FnOnce(&mut Ui, Room) -> R,
    ) -> PanelResponse<R> {
        let window = context.content_rect();
        let content = {
            use std::hash::{Hash as _, Hasher as _};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            (self.content, self.orientation).hash(&mut hasher);
            hasher.finish()
        };
        let stored = context.data(|data| data.get_temp::<State>(self.id));
        let mut state = stored.unwrap_or(State {
            placement: self.placement,
            size: Vec2::ZERO,
            hold: None,
            measure: true,
            content,
            orientation: self.orientation,
        });
        if state.hold.is_none() {
            state.placement = self.placement;
        }
        if state.content != content {
            state.content = content;
            state.measure = true;
        }
        if state.orientation != self.orientation {
            state.orientation = self.orientation;
            state.size = vec2(state.size.y, state.size.x);
            if let Some(hold) = &mut state.hold {
                hold.turn();
            }
        }

        let layer = egui::LayerId::new(egui::Order::Foreground, self.id);
        let last = Rect::from_min_size(state.placement.position(window, state.size), state.size);
        let step = hold::step(context, &mut state.hold, layer, (last, state.size), window);
        let mut changed = false;
        if let Step::Let { position, edge } = step {
            let at = Placement::at(window, state.size, position);
            state.placement.at = edge.map_or(at, |edge| edge.pin(at));
            changed = true;
        }
        let measure = std::mem::take(&mut state.measure);
        let lifted = state.hold.is_some_and(|hold| hold.taken());
        let position = match state.hold.and_then(|hold| hold.position()) {
            Some(position) => inside(window, state.size, position),
            None => state.placement.position(window, state.size),
        };
        let orientation = self.orientation;
        let room = content_room(context, window, orientation);
        let frame = frame(context, lifted);

        let name = self.name;
        let shown = egui::Area::new(self.id)
            .order(egui::Order::Foreground)
            .movable(false)
            .constrain(false)
            .pivot(egui::Align2::LEFT_TOP)
            .sizing_pass(measure)
            .current_pos(position)
            .show(context, |ui| {
                let shown = frame.show(ui, |ui| {
                    ui.with_layout(layout(orientation), |ui| add_contents(ui, room))
                        .inner
                });
                shown.response.widget_info(move || {
                    egui::WidgetInfo::labeled(egui::accesskit::Role::Group, true, &name)
                });
                shown.inner
            });
        let inner = shown.inner;
        let rect = shown.response.rect;
        let resized = rect.size() != state.size;
        state.size = rect.size();

        if lifted {
            context.set_cursor_icon(egui::CursorIcon::Grabbing);
        }

        let measured = window.is_positive();
        if measured {
            context.data_mut(|data| data.insert_temp(self.id, state));
        }
        let waiting = state.hold.is_some_and(|hold| hold.waits());
        if !measured || measure || state.measure || resized || changed || waiting {
            context.request_repaint();
        }

        PanelResponse {
            inner,
            rect,
            placement: state.placement,
            changed,
            taken: matches!(step, Step::Taken),
            edge: state.hold.and_then(|hold| hold.edge()),
        }
    }
}

/// The top left corner of a panel of `size` at `position`, moved so the
/// panel stands whole in the room of `window`; one larger than the room
/// keeps its start in sight.
fn inside(window: Rect, size: Vec2, position: egui::Pos2) -> egui::Pos2 {
    let room = placement::room(window);
    let far = (room.max - size).max(room.min);
    position.clamp(room.min, far)
}

/// The frame of a panel: the frame of a window of egui, lighter and with a
/// deeper shadow while the panel is held.
fn frame(context: &Context, lifted: bool) -> egui::Frame {
    let frame = egui::Frame::window(&context.global_style());
    if !lifted {
        return frame;
    }
    frame
        .fill(frame.fill.lerp_to_gamma(egui::Color32::WHITE, LIFT_LIGHT))
        .shadow(LIFT_SHADOW)
}

/// The layout of the content of a panel laid out as `orientation`.
fn layout(orientation: Orientation) -> egui::Layout {
    match orientation {
        Orientation::Horizontal => egui::Layout::left_to_right(egui::Align::Center),
        Orientation::Vertical => egui::Layout::top_down(egui::Align::Center),
    }
}

/// The room of the content of a panel laid out as `orientation`: the length
/// of the window less the margins and the frame.
fn content_room(context: &Context, window: Rect, orientation: Orientation) -> Room {
    let space = placement::room(window).size();
    let margin = egui::Frame::window(&context.global_style())
        .total_margin()
        .sum();
    let length = match orientation {
        Orientation::Horizontal => space.x - margin.x,
        Orientation::Vertical => space.y - margin.y,
    };
    Room {
        orientation,
        length: length.max(0.0),
    }
}
