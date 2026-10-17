//! The five zones of a window.
//!
//! ```text
//! ┌──────────────────────────┐
//! │╲          top           ╱│
//! │  ╲ ┌────────────────┐ ╱  │
//! │left│     centre     │right
//! │  ╱ └────────────────┘ ╲  │
//! │╱         bottom         ╲│
//! └──────────────────────────┘
//! ```
//!
//! The centre is the window less [`EDGE`] of its size on every side. Outside
//! it, the diagonals from the corners split the rest into four.

use egui::{Pos2, Rect};

/// The part of the width and of the height that each edge zone reaches into
/// the window.
pub const EDGE: f32 = 0.2;

/// A zone of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// Along the top edge.
    Top,
    /// Along the right edge.
    Right,
    /// Along the bottom edge.
    Bottom,
    /// Along the left edge.
    Left,
    /// The large middle.
    Centre,
}

/// How the content of a panel is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// In a row.
    Horizontal,
    /// In a column.
    Vertical,
}

impl Zone {
    /// The zone of `window` that `point` is in.
    pub fn of(window: Rect, point: Pos2) -> Zone {
        let centre = window.shrink2(window.size() * EDGE);
        if centre.contains(point) {
            return Zone::Centre;
        }
        let half = window.size() / 2.0;
        let x = (point.x - window.center().x) / half.x;
        let y = (point.y - window.center().y) / half.y;
        match (x.abs() > y.abs(), x < 0.0, y < 0.0) {
            (true, true, _) => Zone::Left,
            (true, false, _) => Zone::Right,
            (false, _, true) => Zone::Top,
            (false, _, false) => Zone::Bottom,
        }
    }

    /// How a panel stands in this zone: a column along the sides, a row along
    /// the top and the bottom. In the centre a panel keeps what it had.
    pub fn orientation(self) -> Option<Orientation> {
        match self {
            Zone::Left | Zone::Right => Some(Orientation::Vertical),
            Zone::Top | Zone::Bottom => Some(Orientation::Horizontal),
            Zone::Centre => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    const WINDOW: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 600.0));

    #[test]
    fn the_middle_is_the_centre() {
        assert_eq!(Zone::of(WINDOW, pos2(500.0, 300.0)), Zone::Centre);
        assert_eq!(Zone::of(WINDOW, pos2(201.0, 121.0)), Zone::Centre);
    }

    #[test]
    fn each_edge_has_its_zone() {
        assert_eq!(Zone::of(WINDOW, pos2(10.0, 300.0)), Zone::Left);
        assert_eq!(Zone::of(WINDOW, pos2(990.0, 300.0)), Zone::Right);
        assert_eq!(Zone::of(WINDOW, pos2(500.0, 10.0)), Zone::Top);
        assert_eq!(Zone::of(WINDOW, pos2(500.0, 590.0)), Zone::Bottom);
    }

    #[test]
    fn a_corner_is_split_by_its_diagonal() {
        assert_eq!(Zone::of(WINDOW, pos2(50.0, 20.0)), Zone::Top);
        assert_eq!(Zone::of(WINDOW, pos2(20.0, 50.0)), Zone::Left);
        assert_eq!(Zone::of(WINDOW, pos2(980.0, 590.0)), Zone::Bottom);
    }

    #[test]
    fn the_sides_stand_a_panel_up() {
        assert_eq!(Zone::Left.orientation(), Some(Orientation::Vertical));
        assert_eq!(Zone::Top.orientation(), Some(Orientation::Horizontal));
        assert_eq!(Zone::Centre.orientation(), None);
    }
}
