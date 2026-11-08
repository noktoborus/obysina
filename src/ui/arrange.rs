//! Where the panels stand when the settings keep no place for them in the
//! posture of the window: the first way in which both fit and do not
//! overlap (decision 10 of `docs/panels.md`).
//!
//! ```text
//! 1. both in one line at the first short edge
//! 2. one at each short edge
//! 3. both in one line at the first long edge
//! 4. one at each long edge
//! 5. 1 to 4, the tools in two lines, the beads a block
//! none: the beads a column at the right, the tools a row at the top
//! ```
//!
//! The short edges of a window that stands are the top and the bottom, of
//! one that lies the left and the right. The first edge is the top or the
//! left. The tools stand at the start of an edge, the beads after them or at
//! the start of the other edge.

use egui::{Rect, Vec2, vec2};
use obusina_panel::{MARGIN, Placement};

use crate::settings::{PanelView, Posture};

/// A panel to stand: the size of each of its views.
pub struct Panel<'a> {
    /// The views of a line: along a top or bottom edge, then along a side.
    pub line: [PanelView; 2],
    /// The views that take less length: along a top or bottom edge, then
    /// along a side.
    pub compact: [PanelView; 2],
    /// The size of the panel in a view.
    pub size: &'a dyn Fn(PanelView) -> Vec2,
}

/// The place and the view of a panel.
pub type Stand = (Placement, PanelView);

/// Where `tools` and `beads` stand in a window of `window` in `posture`.
pub fn arrange(window: Rect, posture: Posture, tools: &Panel, beads: &Panel) -> [Stand; 2] {
    let room = obusina_panel::room(window).size();
    let (short, long) = match posture {
        Posture::Portrait => (Side::Across, Side::Down),
        Posture::Landscape => (Side::Down, Side::Across),
    };
    for compact in [false, true] {
        for side in [short, long] {
            if let Some(found) = together(room, side, compact, tools, beads) {
                return found;
            }
            if let Some(found) = apart(room, side, compact, tools, beads) {
                return found;
            }
        }
    }
    [
        (Placement::new(vec2(0.0, 0.0)), tools.line[0]),
        (Placement::new(vec2(1.0, 0.0)), beads.line[1]),
    ]
}

/// The edges a panel stands along: the top and the bottom, or the left and
/// the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    /// The top and the bottom: a panel lies across.
    Across,
    /// The left and the right: a panel stands down.
    Down,
}

impl Side {
    /// The index of the view of this side in [`Panel::line`].
    fn index(self) -> usize {
        match self {
            Side::Across => 0,
            Side::Down => 1,
        }
    }

    /// The length along the edge and across it of `size`.
    fn split(self, size: Vec2) -> (f32, f32) {
        match self {
            Side::Across => (size.x, size.y),
            Side::Down => (size.y, size.x),
        }
    }

    /// The place `along` an edge of this side, the first (`far` false) or
    /// the other one.
    fn at(self, along: f32, far: bool) -> Placement {
        let across = if far { 1.0 } else { 0.0 };
        Placement::new(match self {
            Side::Across => vec2(along, across),
            Side::Down => vec2(across, along),
        })
    }
}

/// The view of `panel` along `side`.
fn view(panel: &Panel, side: Side, compact: bool) -> PanelView {
    let views = if compact { panel.compact } else { panel.line };
    views[side.index()]
}

/// Both panels in one line at the first edge of `side`, if they fit.
fn together(
    room: Vec2,
    side: Side,
    compact: bool,
    tools: &Panel,
    beads: &Panel,
) -> Option<[Stand; 2]> {
    let (tools_view, beads_view) = (view(tools, side, compact), view(beads, side, compact));
    let (tools_along, _) = side.split((tools.size)(tools_view));
    let (beads_along, _) = side.split((beads.size)(beads_view));
    let (length, _) = side.split(room);
    (tools_along + MARGIN + beads_along <= length).then(|| {
        [
            (side.at(0.0, false), tools_view),
            (side.at(1.0, false), beads_view),
        ]
    })
}

/// One panel at each edge of `side`, if each fits and they do not meet.
fn apart(
    room: Vec2,
    side: Side,
    compact: bool,
    tools: &Panel,
    beads: &Panel,
) -> Option<[Stand; 2]> {
    let (tools_view, beads_view) = (view(tools, side, compact), view(beads, side, compact));
    let (tools_along, tools_across) = side.split((tools.size)(tools_view));
    let (beads_along, beads_across) = side.split((beads.size)(beads_view));
    let (length, width) = side.split(room);
    let fits = tools_along <= length
        && beads_along <= length
        && tools_across + MARGIN + beads_across <= width;
    fits.then(|| {
        [
            (side.at(0.0, false), tools_view),
            (side.at(0.0, true), beads_view),
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    /// A button and its space.
    const STEP: f32 = 40.0;

    /// The size of a panel of `count` buttons in `view`.
    fn size(view: PanelView, count: f32) -> Vec2 {
        match view {
            PanelView::Row => vec2(count * STEP, STEP),
            PanelView::Column => vec2(STEP, count * STEP),
            PanelView::TwoRows => vec2((count / 2.0).ceil() * STEP, 2.0 * STEP),
            PanelView::TwoColumns => vec2(2.0 * STEP, (count / 2.0).ceil() * STEP),
            PanelView::Block => vec2(3.0 * STEP, (count / 3.0).ceil() * STEP),
        }
    }

    fn stands(window: Vec2, tools: f32, beads: f32) -> [Stand; 2] {
        let tools_size = move |view| size(view, tools);
        let beads_size = move |view| size(view, beads);
        let tools = Panel {
            line: [PanelView::Row, PanelView::Column],
            compact: [PanelView::TwoRows, PanelView::TwoColumns],
            size: &tools_size,
        };
        let beads = Panel {
            line: [PanelView::Row, PanelView::Column],
            compact: [PanelView::Block, PanelView::Block],
            size: &beads_size,
        };
        let rect = Rect::from_min_size(pos2(0.0, 0.0), window);
        arrange(rect, Posture::of(window), &tools, &beads)
    }

    fn at(stand: Stand) -> Vec2 {
        stand.0.at
    }

    /// A wide window holds both in one column at the left, a short edge.
    #[test]
    fn a_window_that_lies_puts_both_in_one_column_at_the_left() {
        let [tools, beads] = stands(vec2(1200.0, 800.0), 6.0, 10.0);
        assert_eq!((at(tools), tools.1), (vec2(0.0, 0.0), PanelView::Column));
        assert_eq!((at(beads), beads.1), (vec2(0.0, 1.0), PanelView::Column));
    }

    /// A phone that stands has room for one row at the top, one at the
    /// bottom.
    #[test]
    fn a_phone_that_stands_puts_one_row_at_the_top_one_at_the_bottom() {
        let [tools, beads] = stands(vec2(420.0, 900.0), 6.0, 10.0);
        assert_eq!((at(tools), tools.1), (vec2(0.0, 0.0), PanelView::Row));
        assert_eq!((at(beads), beads.1), (vec2(0.0, 1.0), PanelView::Row));
    }

    /// A row too long for the short edges goes to a long one.
    #[test]
    fn a_line_too_long_for_a_short_edge_goes_to_a_long_one() {
        let [tools, beads] = stands(vec2(300.0, 900.0), 6.0, 12.0);
        assert_eq!((at(tools), tools.1), (vec2(0.0, 0.0), PanelView::Column));
        assert_eq!((at(beads), beads.1), (vec2(0.0, 1.0), PanelView::Column));
    }

    /// When no line fits, the tools go in two lines and the beads in a block.
    #[test]
    fn a_small_window_takes_the_compact_views() {
        let [tools, beads] = stands(vec2(300.0, 330.0), 6.0, 12.0);
        assert_eq!(tools.1, PanelView::TwoRows);
        assert_eq!(beads.1, PanelView::Block);
    }
}
