#![cfg_attr(doc, doc = include_str!(concat!(env!("OUT_DIR"), "/pictures.md")))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod edge;
mod placement;

pub use edge::{Edge, HYSTERESIS, Orientation, SNAP, SNAP_PART};
pub use placement::{MARGIN, Placement};

use egui::{Context, Id, Pos2, Rect, Response, Sense, Ui, Vec2, vec2};

/// The thin side of the grip: wide enough for a finger.
const GRIP_THIN: f32 = 18.0;
/// The long side of the grip.
const GRIP_LONG: f32 = 36.0;
/// The distance between two dots of the grip.
const DOT_STEP: f32 = 6.0;
/// The radius of a dot of the grip.
const DOT_RADIUS: f32 = 1.5;
/// The width of the outline of a strip and of the ghost.
const ZONE_LINE: f32 = 1.5;
/// How strong the outline of a strip is.
const ZONE_LINE_ALPHA: f32 = 0.6;
/// How strong the fill of the strip under the pointer is.
const ZONE_FILL_ALPHA: f32 = 0.35;
/// How strong the fill of the ghost is.
const GHOST_FILL_ALPHA: f32 = 0.5;

/// The room the content of a panel has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Room {
    /// How the content is laid out.
    pub orientation: Orientation,
    /// The length the content may take along its orientation before the
    /// window ends. The content breaks itself into lines to fit it; the
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
}

/// A drag of the grip.
#[derive(Debug, Clone, Copy)]
struct Drag {
    /// The point of the panel under the pointer, from its top left corner.
    grab: Vec2,
    /// The top left corner of the panel, which follows the pointer.
    position: Pos2,
    /// The pointer.
    pointer: Pos2,
    /// The edge the panel docks to when it is let go.
    target: Option<Edge>,
    /// The pointer went far enough for a drag, not a tap.
    moved: bool,
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
}

/// A floating panel with no title: a grip of dots moves it, a tap on the grip
/// folds it. Its content stands in the middle across the panel.
///
/// While it is dragged it keeps its shape. The edge strip under the pointer
/// and a ghost of the panel at that edge are drawn; on release the panel
/// takes the place of the ghost. Let go in the middle, it stays free and
/// keeps its orientation.
pub struct FloatingPanel {
    id: Id,
    name: String,
    placement: Placement,
}

impl FloatingPanel {
    /// A panel; `name` is what a screen reader reads for its grip.
    pub fn new(id_salt: impl std::hash::Hash + std::fmt::Debug, name: impl Into<String>) -> Self {
        FloatingPanel {
            id: Id::unique(("floating-panel", id_salt)),
            name: name.into(),
            placement: Placement::docked(Edge::Top, 0.0),
        }
    }

    /// Where the panel stands the first time it is shown. Later it keeps the
    /// place it was moved to.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
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
        let stored = context.data(|data| data.get_temp::<State>(self.id));
        let mut state = stored.unwrap_or(State {
            placement: self.placement,
            open: [None; 2],
            size: Vec2::ZERO,
            drag: None,
            measure: true,
        });
        let (pointer, held) =
            context.input(|input| (input.pointer.latest_pos(), input.pointer.primary_down()));
        if let (Some(drag), Some(pointer), true) = (&mut state.drag, pointer, held) {
            drag.position = pointer - drag.grab;
            drag.pointer = pointer;
            if drag.moved {
                drag.target = Edge::under(window, pointer, drag.target);
            }
        }
        let measure = std::mem::take(&mut state.measure);
        let position = match state.drag {
            Some(drag) => drag.position,
            None => state.placement.position(window, state.size),
        };
        let orientation = state.placement.orientation();
        let frame = egui::Frame::window(&context.global_style());
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
                    let layout = match orientation {
                        Orientation::Horizontal => egui::Layout::left_to_right(egui::Align::Center),
                        Orientation::Vertical => egui::Layout::top_down(egui::Align::Center),
                    };
                    ui.with_layout(layout, |ui| self.body(ui, &state, room, add_contents))
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

        if grip.is_pointer_button_down_on()
            && state.drag.is_none()
            && let Some(origin) = context.input(|input| input.pointer.press_origin())
        {
            state.drag = Some(Drag {
                grab: origin - rect.min,
                position: rect.min,
                pointer: origin,
                target: None,
                moved: false,
            });
        }
        if let Some(drag) = &mut state.drag
            && grip.dragged()
            && !drag.moved
        {
            drag.moved = true;
            drag.target = Edge::under(window, drag.pointer, None);
        }
        let start = frame.total_margin();
        let ghost = state.drag.filter(|drag| drag.moved).and_then(|drag| {
            let edge = drag.target?;
            Some(ghost(
                window,
                &state,
                vec2(start.left, start.top),
                drag,
                edge,
            ))
        });
        if let Some(drag) = state.drag.filter(|drag| drag.moved) {
            draw_zones(context, window, drag.target, ghost);
        }

        let mut changed = false;
        if !held
            && let Some(drag) = state.drag.take()
            && drag.moved
        {
            let folded = state.placement.folded;
            state.placement = match (drag.target, ghost) {
                (Some(edge), Some(ghost)) => Placement::against(edge, window, ghost),
                _ => Placement::free(
                    Placement::at(window, rect.size(), drag.position),
                    orientation,
                ),
            };
            state.placement.folded = folded;
            state.measure = state.placement.orientation() != orientation;
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
        }
    }

    /// The grip, then the content unless the panel is folded.
    fn body<R>(
        &self,
        ui: &mut Ui,
        state: &State,
        room: Room,
        add_contents: impl FnOnce(&mut Ui, Room) -> R,
    ) -> (Response, Option<R>) {
        let grip = self.grip(ui, state);
        let inner = (!state.placement.folded).then(|| add_contents(ui, room));
        (grip, inner)
    }

    /// The grip: a field of dots across the start of the panel.
    fn grip(&self, ui: &mut Ui, state: &State) -> Response {
        let size = grip_size(state.placement.orientation());
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

/// Where the panel stands if it is let go at `edge`: against the edge, its
/// grip under the pointer along it. `start` is where the grip begins in the
/// panel. The size in the orientation of the edge is the one last drawn;
/// before the panel was ever drawn so, its sides swap.
fn ghost(window: Rect, state: &State, start: Vec2, drag: Drag, edge: Edge) -> Rect {
    let target = edge.orientation();
    let swapped = vec2(state.size.y, state.size.x);
    let size = if target == state.placement.orientation() {
        state.size
    } else if state.placement.folded {
        swapped
    } else {
        state.open[index(target)].unwrap_or(swapped)
    };
    let grab = drag.grab.clamp(start, start + grip_size(target));
    let under = Rect::from_min_size(drag.pointer - grab, size);
    let position = Placement::against(edge, window, under).position(window, size);
    Rect::from_min_size(position, size)
}

/// The edge strips and the ghost, drawn over the field and under the panels
/// while a panel is dragged; the strip under the pointer is filled.
fn draw_zones(context: &Context, window: Rect, target: Option<Edge>, ghost: Option<Rect>) {
    let layer = egui::LayerId::new(egui::Order::Middle, Id::unique("floating-panel-zones"));
    let painter = context.layer_painter(layer);
    let selection = context.global_style().visuals.selection;
    let line = egui::Stroke::new(
        ZONE_LINE,
        selection.stroke.color.gamma_multiply(ZONE_LINE_ALPHA),
    );
    for edge in Edge::ALL {
        let fill = if target == Some(edge) {
            selection.bg_fill.gamma_multiply(ZONE_FILL_ALPHA)
        } else {
            egui::Color32::TRANSPARENT
        };
        painter.rect(
            edge.strip(window),
            0.0,
            fill,
            line,
            egui::StrokeKind::Inside,
        );
    }
    if let Some(ghost) = ghost {
        painter.rect(
            ghost,
            context.global_style().visuals.window_corner_radius,
            selection.bg_fill.gamma_multiply(GHOST_FILL_ALPHA),
            egui::Stroke::new(ZONE_LINE, selection.stroke.color),
            egui::StrokeKind::Inside,
        );
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
