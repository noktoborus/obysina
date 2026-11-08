//! Where a panel stands, in parts of the window, so the place outlives a
//! change of the size of the window.

use egui::{Pos2, Rect, Vec2, vec2};

/// The space between a panel and the edge of the window, in points.
pub const MARGIN: f32 = 8.0;

/// Where a panel stands.
///
/// `at` is a part of the room the window leaves the panel on each axis: 0 is
/// against the left or the top, 1 against the right or the bottom. A panel
/// against an edge stays there when the window changes its size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// The place in the room of the window, from 0 to 1 on each axis.
    pub at: Vec2,
}

impl Placement {
    /// A panel at `at` of the room of the window.
    pub fn new(at: Vec2) -> Placement {
        Placement { at }
    }

    /// The top left corner of a panel of `size` in `window`.
    pub fn position(&self, window: Rect, size: Vec2) -> Pos2 {
        let room = room(window);
        let at = self.at.clamp(Vec2::ZERO, Vec2::splat(1.0));
        room.min + (room.size() - size).max(Vec2::ZERO) * at
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    const WINDOW: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1000.0, 600.0));
    const SIZE: Vec2 = vec2(100.0, 300.0);

    #[test]
    fn a_panel_at_one_stands_against_the_far_edges() {
        let placement = Placement::new(vec2(1.0, 1.0));
        assert_eq!(
            placement.position(WINDOW, SIZE),
            pos2(1000.0 - MARGIN - SIZE.x, 600.0 - MARGIN - SIZE.y)
        );
    }

    #[test]
    fn a_position_comes_back_from_its_part() {
        let position = pos2(300.0, 120.0);
        let at = Placement::at(WINDOW, SIZE, position);
        let back = Placement::new(at).position(WINDOW, SIZE);
        assert!((back - position).length() < 0.01, "{back:?}");
    }

    #[test]
    fn a_panel_keeps_its_part_of_a_smaller_window() {
        let placement = Placement::new(vec2(1.0, 0.5));
        let small = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 400.0));
        let position = placement.position(small, SIZE);
        let middle = MARGIN + (400.0 - 2.0 * MARGIN - SIZE.y) / 2.0;
        assert_eq!(position, pos2(400.0 - MARGIN - SIZE.x, middle));
    }

    #[test]
    fn a_panel_larger_than_the_window_keeps_its_start_in_sight() {
        let narrow = Rect::from_min_max(pos2(0.0, 0.0), pos2(50.0, 600.0));
        let placement = Placement::new(vec2(1.0, 0.0));
        assert_eq!(placement.position(narrow, SIZE).x, MARGIN);
    }
}
