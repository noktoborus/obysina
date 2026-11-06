#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod edge;
mod placement;
mod shake;

pub use edge::{Edge, HYSTERESIS, Orientation, SNAP, SNAP_PART};
pub use placement::{MARGIN, Placement};

use egui::{Context, Id, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};

use shake::Shake;

/// The thin side of the grip: wide enough for a finger.
const GRIP_THIN: f32 = 18.0;
/// The long side of the grip.
const GRIP_LONG: f32 = 36.0;
/// The distance between two dots of the grip.
const DOT_STEP: f32 = 6.0;
/// The radius of a dot of the grip.
const DOT_RADIUS: f32 = 1.5;

/// The room the content of a panel has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// How the content is laid out.
    pub orientation: Orientation,
    /// The length the content may take along its orientation before the
    /// window ends. A content of a length of its own fits itself to it; the
    /// panel does not reflow it.
    pub length: f32,
}

/// What a panel remembers from frame to frame.
#[derive(Debug, Clone, Copy)]
struct State {
    /// Where it stands.
    placement: Placement,
    /// The size of the open panel in each orientation, as last drawn:
    /// horizontal, then vertical.
    open: [Option<Vec2>; 2],
    /// The size as drawn in the last frame.
    size: Vec2,
    /// While the grip is held.
    drag: Option<Drag>,
    /// The content changed its shape: the next frame measures the panel
    /// again instead of laying the content out in the size it had.
    measure: bool,
    /// The key of the shape of the content the panel was measured for.
    content: u64,
}

/// A drag of the grip.
#[derive(Debug, Clone, Copy)]
struct Drag {
    /// The point of the panel under the pointer, from its top left corner.
    grab: Vec2,
    /// The top left corner of the panel, which follows the pointer.
    position: Pos2,
    /// The edge strip the pointer is in.
    edge: Option<Edge>,
    /// The pointer went far enough for a drag, not a tap.
    moved: bool,
    /// The turns of the pointer, for a shake.
    shake: Shake,
    /// The grip was shaken in this drag: a drag is one shake at most.
    shook: bool,
}

/// What the panel did in this frame.
pub struct PanelResponse<R> {
    /// What the content gave back; `None` while the panel is folded.
    pub inner: Option<R>,
    /// The grip.
    pub grip: Response,
    /// The whole panel.
    pub rect: Rect,
    /// Where the panel stands now.
    pub placement: Placement,
    /// The panel was let go or folded in this frame: `placement` is one the
    /// caller may keep, to give back at the next start.
    pub changed: bool,
    /// The grip was shaken in this frame: held, and moved to and fro
    /// quickly.
    pub shaken: bool,
}

/// A floating panel with no title: a grip of dots moves it, a tap on the grip
/// folds it. Its content stands in the middle across the panel.
///
/// While it is dragged it follows the pointer and turns at once: to the
/// orientation of the edge strip under the pointer, or to the other one when
/// it does not fit. Let go in an edge strip, it stands against that edge.
pub struct FloatingPanel {
    id: Id,
    name: String,
    placement: Placement,
    content: u64,
}

impl FloatingPanel {
    /// A panel; `name` is what a screen reader reads for its grip.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, name: impl Into<String>) -> Self {
        FloatingPanel {
            id: Id::unique(("floating-panel", id_salt)),
            name: name.into(),
            placement: Placement::new(Vec2::ZERO, Orientation::Horizontal),
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

    /// Where the panel stands the first time it is shown. Later it keeps the
    /// place it was moved to.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Shows the panel; `add_contents` lays out the content in the room it is
    /// given.
    ///
    /// The content is laid out a second time in a frame where the panel has
    /// never been drawn in the other orientation: invisible, to measure it,
    /// so the panel knows whether it fits turned (`docs/panels.md`).
    pub fn show<R>(
        self,
        context: &Context,
        mut add_contents: impl FnMut(&mut Ui, Room) -> R,
    ) -> PanelResponse<R> {
        let window = context.content_rect();
        let stored = context.data(|data| data.get_temp::<State>(self.id));
        let mut state = stored.unwrap_or(State {
            placement: self.placement,
            open: [None; 2],
            size: Vec2::ZERO,
            drag: None,
            measure: true,
            content: self.content,
        });
        if state.content != self.content {
            state.content = self.content;
            state.open = [None; 2];
            state.measure = true;
        }
        let (pointer, held, time) = context.input(|input| {
            (
                input.pointer.latest_pos(),
                input.pointer.primary_down(),
                input.time,
            )
        });
        let mut shaken = false;
        if let (Some(drag), Some(pointer), true) = (&mut state.drag, pointer, held) {
            drag.position = pointer - drag.grab;
            if !drag.shook && drag.shake.feed(time, pointer) {
                drag.shook = true;
                shaken = true;
            }
            if drag.moved {
                drag.edge = Edge::under(window, pointer, drag.edge);
            }
        }
        let frame = egui::Frame::window(&context.global_style());
        let margin = frame.total_margin();
        let start = vec2(margin.left, margin.top);
        self.measure_other(context, window, &frame, &mut state, &mut add_contents);
        turn(window, &mut state, start);
        let measure = std::mem::take(&mut state.measure);
        let position = match state.drag {
            Some(drag) => drag.position,
            None => state.placement.position(window, state.size),
        };
        let orientation = state.placement.orientation;
        let room = room(context, window, orientation, &frame);

        let shown = egui::Area::new(self.id)
            .order(egui::Order::Foreground)
            .movable(false)
            .constrain(false)
            .pivot(egui::Align2::LEFT_TOP)
            .sizing_pass(measure)
            .current_pos(position)
            .show(context, |ui| {
                frame.show(ui, |ui| {
                    ui.with_layout(layout(orientation), |ui| {
                        self.body(ui, &state, room, &mut add_contents)
                    })
                    .inner
                })
            });
        let (grip, inner) = shown.inner.inner;
        let rect = shown.response.rect;
        let resized = rect.size() != state.size;
        state.size = rect.size();
        if !state.placement.folded {
            state.open[index(orientation)] = Some(rect.size());
        }

        if let Some(drag) = &mut state.drag {
            let on_grip = grip.rect.translate(-rect.min.to_vec2());
            drag.grab = drag
                .grab
                .clamp(on_grip.min.to_vec2(), on_grip.max.to_vec2());
        }
        if grip.is_pointer_button_down_on()
            && state.drag.is_none()
            && let Some(origin) = context.input(|input| input.pointer.press_origin())
        {
            state.drag = Some(Drag {
                grab: origin - rect.min,
                position: rect.min,
                edge: None,
                moved: false,
                shake: Shake::default(),
                shook: false,
            });
        }
        if let Some(drag) = &mut state.drag
            && grip.dragged()
        {
            drag.moved = true;
        }

        let mut changed = false;
        if !held
            && let Some(drag) = state.drag.take()
            && drag.moved
        {
            let at = Placement::at(window, rect.size(), drag.position);
            state.placement.at = drag.edge.map_or(at, |edge| edge.pin(at));
            changed = true;
        }
        if grip.clicked() {
            state.placement.folded = !state.placement.folded;
            state.measure = true;
            changed = true;
        }

        let measured = window.is_positive();
        if measured {
            context.data_mut(|data| data.insert_temp(self.id, state));
        }
        if !measured || measure || state.measure || resized || changed || state.drag.is_some() {
            context.request_repaint();
        }

        PanelResponse {
            inner,
            grip,
            rect,
            placement: state.placement,
            changed,
            shaken,
        }
    }

    /// The grip, then the content unless the panel is folded.
    /// Measures the open panel in the orientation it does not have, when it
    /// was never drawn so: the content laid out once more, invisible, in a
    /// sizing pass of an area of its own.
    fn measure_other<R>(
        &self,
        context: &Context,
        window: Rect,
        frame: &egui::Frame,
        state: &mut State,
        add_contents: &mut impl FnMut(&mut Ui, Room) -> R,
    ) {
        let other = state.placement.orientation.other();
        if state.placement.folded || state.open[index(other)].is_some() || !window.is_positive() {
            return;
        }
        let mut turned = *state;
        turned.placement.orientation = other;
        let room = room(context, window, other, frame);
        let measured = egui::Area::new(self.id.with("measure"))
            .order(egui::Order::Background)
            .movable(false)
            .interactable(false)
            .constrain(false)
            .sizing_pass(true)
            .current_pos(window.min)
            .show(context, |ui| {
                frame.show(ui, |ui| {
                    ui.with_layout(layout(other), |ui| {
                        self.body(ui, &turned, room, &mut *add_contents)
                    })
                })
            });
        state.open[index(other)] = Some(measured.response.rect.size());
    }

    fn body<R>(
        &self,
        ui: &mut Ui,
        state: &State,
        room: Room,
        add_contents: &mut impl FnMut(&mut Ui, Room) -> R,
    ) -> (Response, Option<R>) {
        let grip = self.grip(ui, state);
        let inner = (!state.placement.folded).then(|| add_contents(ui, room));
        (grip, inner)
    }

    /// The grip: a field of dots across the start of the panel.
    fn grip(&self, ui: &mut Ui, state: &State) -> Response {
        let size = grip_size(state.placement.orientation);
        let response = ui.allocate_response(size, Sense::click_and_drag());
        let name = self.name.clone();
        let folded = state.placement.folded;
        response.widget_info(move || {
            let mut info = egui::WidgetInfo::labeled(egui::accesskit::Role::Button, true, &name);
            info.selected = Some(folded);
            info
        });
        let visuals = ui.style().interact(&response);
        let colour = visuals.fg_stroke.color;
        let painter = ui.painter();
        let columns = (size.x / DOT_STEP).floor() as usize;
        let rows = (size.y / DOT_STEP).floor() as usize;
        let start =
            response.rect.center() - vec2(columns as f32 - 1.0, rows as f32 - 1.0) * DOT_STEP / 2.0;
        for row in 0..rows {
            for column in 0..columns {
                let centre = start + vec2(column as f32, row as f32) * DOT_STEP;
                painter.circle_filled(centre, DOT_RADIUS, colour);
            }
        }
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        response
    }
}

/// The layout of the content of a panel laid out as `orientation`.
fn layout(orientation: Orientation) -> egui::Layout {
    match orientation {
        Orientation::Horizontal => egui::Layout::left_to_right(egui::Align::Center),
        Orientation::Vertical => egui::Layout::top_down(egui::Align::Center),
    }
}

/// The room of the content of a panel laid out as `orientation`: the length
/// of the window less the margins, the frame and the grip.
fn room(context: &Context, window: Rect, orientation: Orientation, frame: &egui::Frame) -> Room {
    let space = placement::room(window).size();
    let margin = frame.total_margin().sum();
    let spacing = context.global_style().spacing.item_spacing;
    let length = match orientation {
        Orientation::Horizontal => space.x - margin.x - GRIP_THIN - spacing.x,
        Orientation::Vertical => space.y - margin.y - GRIP_THIN - spacing.y,
    };
    Room {
        orientation,
        length: length.max(0.0),
    }
}

/// Turns the panel by the rule of `docs/panels.md`: it wants the
/// orientation of the edge strip under the pointer, or the one it has; when
/// that does not fit and the other does, it takes the other. `start` is where
/// the grip begins in the panel.
fn turn(window: Rect, state: &mut State, start: Vec2) {
    let current = state.placement.orientation;
    let wanted = state
        .drag
        .and_then(|drag| drag.edge)
        .map_or(current, Edge::orientation);
    let other = wanted.other();
    let chosen = if !fits(window, state, start, wanted) && fits(window, state, start, other) {
        other
    } else {
        wanted
    };
    if chosen == current {
        return;
    }
    state.placement.orientation = chosen;
    state.measure = true;
    if let Some(drag) = &mut state.drag {
        let pointer = drag.position + drag.grab;
        drag.grab = grab_in(drag.grab, start, chosen);
        drag.position = pointer - drag.grab;
    }
}

/// The rect of the panel laid out as `orientation`: under the pointer while
/// it is dragged, at its place otherwise. The size is the one last drawn or
/// measured in that orientation; folded, its sides swap.
fn span(window: Rect, state: &State, start: Vec2, orientation: Orientation) -> Rect {
    let swapped = vec2(state.size.y, state.size.x);
    let size = if orientation == state.placement.orientation {
        state.size
    } else if state.placement.folded {
        swapped
    } else {
        state.open[index(orientation)].unwrap_or(swapped)
    };
    let position = match state.drag {
        Some(drag) => drag.position + drag.grab - grab_in(drag.grab, start, orientation),
        None => state.placement.position(window, size),
    };
    Rect::from_min_size(position, size)
}

/// `grab` moved onto the grip of a panel laid out as `orientation`, which
/// begins at `start`.
fn grab_in(grab: Vec2, start: Vec2, orientation: Orientation) -> Vec2 {
    grab.clamp(start, start + grip_size(orientation))
}

/// True when the panel laid out as `orientation` fits the room of `window`.
/// Dragged in the middle, its far end lies inside, the near end being the
/// grip under the pointer. Otherwise it is not larger than the room: at an
/// edge it slides along the edge when it is let go.
fn fits(window: Rect, state: &State, start: Vec2, orientation: Orientation) -> bool {
    let room = placement::room(window);
    let rect = span(window, state, start, orientation);
    match state.drag {
        Some(drag) if drag.edge.is_none() => rect.max.x <= room.max.x && rect.max.y <= room.max.y,
        _ => rect.width() <= room.width() && rect.height() <= room.height(),
    }
}

/// The size of the grip of a panel laid out as `orientation`.
fn grip_size(orientation: Orientation) -> Vec2 {
    match orientation {
        Orientation::Horizontal => vec2(GRIP_THIN, GRIP_LONG),
        Orientation::Vertical => vec2(GRIP_LONG, GRIP_THIN),
    }
}

/// The place of `orientation` in [`State::open`].
fn index(orientation: Orientation) -> usize {
    match orientation {
        Orientation::Horizontal => 0,
        Orientation::Vertical => 1,
    }
}
