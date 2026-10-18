//! Where a panel stands, in parts of the window, so the place outlives a
//! change of the size of the window.

use egui::{Pos2, Rect, Vec2, vec2};

use crate::edge::{Edge, Orientation};

/// The space between a panel and the edge of the window, in points.
pub const MARGIN: f32 = 8.0;

/// Where a panel stands, how it is laid out and whether it is folded.
///
/// `at` is a part of the room the window leaves the panel on each axis: 0 is
/// against the left or the top, 1 against the right or the bottom. A docked
/// panel stands against its edge whatever `at` says across it, and is laid
/// out by the edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The edge the panel stands against; `None` for a free panel.
    pub dock: Option<Edge>,
    /// The place in the room of the window, from 0 to 1 on each axis.
    pub at: Vec2,
    /// How the content is laid out. A docked panel takes the one of its edge.
    pub orientation: Orientation,
    /// Only the grip is shown.
    pub folded: bool,
}

impl Placement {
    /// A panel against `edge`, at `along` of its length.
    pub fn docked(edge: Edge, along: f32) -> Placement {
        let across = match edge {
            Edge::Top | Edge::Left => 0.0,
            Edge::Right | Edge::Bottom => 1.0,
        };
        let at = match edge.orientation() {
            Orientation::Horizontal => vec2(along, across),
            Orientation::Vertical => vec2(across, along),
        };
        Placement {
            dock: Some(edge),
            at,
            orientation: edge.orientation(),
            folded: false,
        }
    }

    /// A panel against `edge` of `window`, along it where `rect` stands.
    pub fn against(edge: Edge, window: Rect, rect: Rect) -> Placement {
        let at = Placement::at(window, rect.size(), rect.min);
        Placement::docked(edge, along(edge, at))
    }

    /// A free panel at `at` of the room of the window.
    pub fn free(at: Vec2, orientation: Orientation) -> Placement {
        Placement {
            dock: None,
            at,
            orientation,
            folded: false,
        }
    }

    /// How the content is laid out: by the edge for a docked panel.
    pub fn orientation(&self) -> Orientation {
        self.dock
            .map_or(self.orientation, |edge| edge.orientation())
    }

    /// The top left corner of a panel of `size` in `window`.
    pub fn position(&self, window: Rect, size: Vec2) -> Pos2 {
        let at = match self.dock {
            Some(edge) => Placement::docked(edge, along(edge, self.at)).at,
            None => self.at,
        };
        let room = room(window);
        room.min + (room.size() - size).max(Vec2::ZERO) * at.clamp(Vec2::ZERO, Vec2::splat(1.0))
    }

    /// The `at` that puts a panel of `size` at `position` in `window`.
    pub fn at(window: Rect, size: Vec2, position: Pos2) -> Vec2 {
        let room = room(window);
        let free = room.size() - size;
        let part = |offset: f32, free: f32| {
            if free > 0.0 {
                (offset / free).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let offset = position - room.min;
        vec2(part(offset.x, free.x), part(offset.y, free.y))
    }
}

/// The part of `window` a panel stands in: all of it less [`MARGIN`].
pub fn room(window: Rect) -> Rect {
    window.shrink(MARGIN)
}

/// The coordinate of `at` along `edge`.
fn along(edge: Edge, at: Vec2) -> f32 {
    match edge.orientation() {
        Orientation::Horizontal => at.x,
        Orientation::Vertical => at.y,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    const WINDOW: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 600.0));
    const SIZE: Vec2 = vec2(100.0, 300.0);

    #[test]
    fn a_docked_panel_stands_against_its_edge() {
        let right = Placement::docked(Edge::Right, 0.0).position(WINDOW, SIZE);
        assert_eq!(right, pos2(1000.0 - MARGIN - SIZE.x, MARGIN));
        let bottom = Placement::docked(Edge::Bottom, 1.0).position(WINDOW, SIZE);
        assert_eq!(
            bottom,
            pos2(1000.0 - MARGIN - SIZE.x, 600.0 - MARGIN - SIZE.y)
        );
    }

    #[test]
    fn a_docked_panel_keeps_to_its_edge_whatever_at_says() {
        let mut placement = Placement::docked(Edge::Left, 0.5);
        placement.at.x = 0.7;
        assert_eq!(placement.position(WINDOW, SIZE).x, MARGIN);
    }

    #[test]
    fn a_position_comes_back_from_its_part() {
        let position = pos2(300.0, 120.0);
        let at = Placement::at(WINDOW, SIZE, position);
        let back = Placement::free(at, Orientation::Vertical).position(WINDOW, SIZE);
        assert!((back - position).length() < 0.01, "{back:?}");
    }

    #[test]
    fn a_free_panel_keeps_its_part_of_a_smaller_window() {
        let placement = Placement::free(vec2(1.0, 0.5), Orientation::Horizontal);
        let small = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 400.0));
        let position = placement.position(small, SIZE);
        assert_eq!(
            position,
            pos2(
                400.0 - MARGIN - SIZE.x,
                MARGIN + (400.0 - 2.0 * MARGIN - SIZE.y) / 2.0
            )
        );
    }

    #[test]
    fn a_panel_larger_than_the_window_keeps_its_start_in_sight() {
        let narrow = Rect::from_min_max(pos2(0.0, 0.0), pos2(50.0, 600.0));
        let position =
            Placement::free(vec2(1.0, 0.0), Orientation::Vertical).position(narrow, SIZE);
        assert_eq!(position.x, MARGIN);
    }
}
