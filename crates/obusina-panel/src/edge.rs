//! The edges of a window a panel stands against.
//!
//! ```text
//! ┌──────────────────────────┐
//! │           top            │   each strip reaches SNAP into the window,
//! │    ┌────────────────┐    │   or SNAP_PART of a small window
//! │left│      free      │right
//! │    └────────────────┘    │   in a corner the nearer edge wins; the
//! │          bottom          │   edge held wins by HYSTERESIS
//! └──────────────────────────┘
//! ```
//!
//! The pointer decides the edge, not the panel: a wide panel still reaches
//! an edge.

use egui::{Pos2, Rect, Vec2};

/// How far an edge reaches into the window, in points.
pub const SNAP: f32 = 64.0;
/// The part of a side of the window that an edge reaches at most, so a small
/// window keeps a free middle.
pub const SNAP_PART: f32 = 0.25;
/// How much nearer another edge has to be before it takes over from the edge
/// held, in points. A pointer on a border does not flip the choice.
pub const HYSTERESIS: f32 = 12.0;

/// An edge of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The right edge.
    Right,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
}

/// How the content of a panel is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// In a row.
    Horizontal,
    /// In a column.
    Vertical,
}

impl Edge {
    /// Every edge.
    pub const ALL: [Edge; 4] = [Edge::Top, Edge::Right, Edge::Bottom, Edge::Left];

    /// The edge a panel dragged to `point` docks to, or `None` in the free
    /// middle. `held` is the edge chosen in the frame before: it reaches
    /// [`HYSTERESIS`] further and wins a corner by as much.
    pub fn under(window: Rect, point: Pos2, held: Option<Edge>) -> Option<Edge> {
        Edge::ALL
            .into_iter()
            .filter_map(|edge| {
                let bonus = if held == Some(edge) { HYSTERESIS } else { 0.0 };
                let distance = edge.distance(window, point) - bonus;
                (distance <= edge.reach(window)).then_some((edge, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(edge, _)| edge)
    }

    /// `at` of a [`crate::Placement`] moved against this edge.
    pub fn pin(self, mut at: Vec2) -> Vec2 {
        match self {
            Edge::Top => at.y = 0.0,
            Edge::Right => at.x = 1.0,
            Edge::Bottom => at.y = 1.0,
            Edge::Left => at.x = 0.0,
        }
        at
    }

    /// How far this edge reaches into `window`.
    fn reach(self, window: Rect) -> f32 {
        let side = match self {
            Edge::Top | Edge::Bottom => window.height(),
            Edge::Left | Edge::Right => window.width(),
        };
        SNAP.min(side * SNAP_PART)
    }

    /// How far `point` is from this edge, inward.
    fn distance(self, window: Rect, point: Pos2) -> f32 {
        match self {
            Edge::Top => point.y - window.min.y,
            Edge::Right => window.max.x - point.x,
            Edge::Bottom => window.max.y - point.y,
            Edge::Left => point.x - window.min.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    const WINDOW: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 600.0));

    #[test]
    fn the_middle_docks_nowhere() {
        assert_eq!(Edge::under(WINDOW, pos2(500.0, 300.0), None), None);
        assert_eq!(Edge::under(WINDOW, pos2(SNAP + 1.0, 300.0), None), None);
    }

    #[test]
    fn each_edge_has_its_strip() {
        assert_eq!(
            Edge::under(WINDOW, pos2(10.0, 300.0), None),
            Some(Edge::Left)
        );
        assert_eq!(
            Edge::under(WINDOW, pos2(990.0, 300.0), None),
            Some(Edge::Right)
        );
        assert_eq!(
            Edge::under(WINDOW, pos2(500.0, 10.0), None),
            Some(Edge::Top)
        );
        assert_eq!(
            Edge::under(WINDOW, pos2(500.0, 590.0), None),
            Some(Edge::Bottom)
        );
    }

    /// A strip is as wide on a large window as on a middle one.
    #[test]
    fn a_strip_does_not_grow_with_the_window() {
        let large = Rect::from_min_max(pos2(0.0, 0.0), pos2(4000.0, 2000.0));
        assert_eq!(
            Edge::under(large, pos2(SNAP, 1000.0), None),
            Some(Edge::Left)
        );
        assert_eq!(Edge::under(large, pos2(SNAP + 1.0, 1000.0), None), None);
    }

    #[test]
    fn a_small_window_keeps_a_free_middle() {
        let small = Rect::from_min_max(pos2(0.0, 0.0), pos2(160.0, 600.0));
        let reach = 160.0 * SNAP_PART;
        assert_eq!(
            Edge::under(small, pos2(reach, 300.0), None),
            Some(Edge::Left)
        );
        assert_eq!(Edge::under(small, pos2(reach + 1.0, 300.0), None), None);
    }

    #[test]
    fn a_pin_moves_only_the_axis_of_the_edge() {
        let at = egui::vec2(0.3, 0.6);
        assert_eq!(Edge::Right.pin(at), egui::vec2(1.0, 0.6));
        assert_eq!(Edge::Top.pin(at), egui::vec2(0.3, 0.0));
    }

    #[test]
    fn a_corner_goes_to_the_nearer_edge() {
        assert_eq!(Edge::under(WINDOW, pos2(30.0, 10.0), None), Some(Edge::Top));
        assert_eq!(
            Edge::under(WINDOW, pos2(10.0, 30.0), None),
            Some(Edge::Left)
        );
    }

    #[test]
    fn the_edge_held_wins_a_corner_by_the_hysteresis() {
        let near_top = pos2(20.0, 10.0);
        assert_eq!(
            Edge::under(WINDOW, near_top, Some(Edge::Left)),
            Some(Edge::Left)
        );
        let far_from_left = pos2(10.0 + HYSTERESIS + 1.0, 10.0);
        assert_eq!(
            Edge::under(WINDOW, far_from_left, Some(Edge::Left)),
            Some(Edge::Top)
        );
    }

    #[test]
    fn the_edge_held_reaches_further() {
        let past = pos2(SNAP + HYSTERESIS / 2.0, 300.0);
        assert_eq!(Edge::under(WINDOW, past, None), None);
        assert_eq!(
            Edge::under(WINDOW, past, Some(Edge::Left)),
            Some(Edge::Left)
        );
    }
}
