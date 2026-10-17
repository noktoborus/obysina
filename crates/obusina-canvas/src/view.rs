//! Where the field is moved to and how large a bead is drawn.

use egui::{Vec2, vec2};

/// The smallest height of a bead, in points.
pub const MIN_BEAD: f32 = 4.0;
/// The largest height of a bead, in points.
pub const MAX_BEAD: f32 = 120.0;

/// Quarter turns in a whole turn.
const QUARTERS: u8 = 4;

/// A way to turn the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    /// A quarter turn clockwise.
    Clockwise,
    /// A quarter turn counter clockwise.
    CounterClockwise,
}

/// How the field is moved, zoomed and turned.
///
/// It turns how the pattern is shown, never the pattern.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// The top left of the turned pattern, from the top left of the field.
    pub offset: Vec2,
    /// The height of a bead, in points.
    pub bead: f32,
    /// The pattern has been put in the middle of the field once.
    pub centred: bool,
    /// Quarter turns clockwise, 0 to 3.
    quarters: u8,
}

impl Default for View {
    fn default() -> Self {
        View {
            offset: Vec2::ZERO,
            bead: 20.0,
            centred: false,
            quarters: 0,
        }
    }
}

impl View {
    /// Puts a pattern of `extent` points, not turned, in the middle of a
    /// field of `field` points.
    pub fn centre(&mut self, field: Vec2, extent: Vec2) {
        self.offset = (field - self.turned(extent)) / 2.0;
        self.centred = true;
    }

    /// How far the field is turned clockwise: 0, 90, 180 or 270 degrees. Four
    /// quarter turns are a whole turn, 360, which is 0 again.
    pub fn degrees(&self) -> u16 {
        u16::from(self.quarters) * 90
    }

    /// Turns the field by a quarter, keeping the middle of a pattern of
    /// `extent` points, not turned, where it stands.
    pub fn turn(&mut self, turn: Turn, extent: Vec2) {
        let middle = self.offset + self.turned(extent) / 2.0;
        self.quarters = match turn {
            Turn::Clockwise => (self.quarters + 1) % QUARTERS,
            Turn::CounterClockwise => (self.quarters + QUARTERS - 1) % QUARTERS,
        };
        self.offset = middle - self.turned(extent) / 2.0;
    }

    /// The size of a pattern of `extent` as it is shown.
    pub fn turned(&self, extent: Vec2) -> Vec2 {
        match self.quarters % 2 {
            0 => extent,
            _ => vec2(extent.y, extent.x),
        }
    }

    /// Where a point of a pattern of `extent` is shown, both from the top left
    /// corner, in points.
    pub fn to_shown(&self, point: Vec2, extent: Vec2) -> Vec2 {
        let (width, height) = (extent.x, extent.y);
        match self.quarters {
            0 => point,
            1 => vec2(height - point.y, point.x),
            2 => vec2(width - point.x, height - point.y),
            _ => vec2(point.y, width - point.x),
        }
    }

    /// The point of a pattern of `extent` shown at `shown`; the reverse of
    /// [`View::to_shown`].
    pub fn to_pattern(&self, shown: Vec2, extent: Vec2) -> Vec2 {
        let (width, height) = (extent.x, extent.y);
        match self.quarters {
            0 => shown,
            1 => vec2(shown.y, height - shown.x),
            2 => vec2(width - shown.x, height - shown.y),
            _ => vec2(width - shown.y, shown.x),
        }
    }

    /// Zooms by `factor`, keeping the point `anchor` of the field still.
    pub fn zoom(&mut self, factor: f32, anchor: Vec2) {
        let bead = (self.bead * factor).clamp(MIN_BEAD, MAX_BEAD);
        let applied = bead / self.bead;
        self.offset = anchor + (self.offset - anchor) * applied;
        self.bead = bead;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zoom_keeps_its_anchor_still() {
        let mut view = View {
            offset: Vec2::new(10.0, 10.0),
            bead: 20.0,
            centred: true,
            quarters: 0,
        };
        let anchor = Vec2::new(50.0, 30.0);
        let before = (anchor - view.offset) / view.bead;

        view.zoom(2.0, anchor);

        assert_eq!(view.bead, 40.0);
        assert_eq!((anchor - view.offset) / view.bead, before);
    }

    #[test]
    fn a_pattern_is_centred_in_the_field() {
        let mut view = View::default();

        view.centre(Vec2::new(1000.0, 700.0), Vec2::new(400.0, 600.0));

        assert_eq!(view.offset, Vec2::new(300.0, 50.0));
        assert!(view.centred);
    }

    #[test]
    fn a_point_comes_back_from_every_turn() {
        let extent = Vec2::new(300.0, 120.0);
        let point = Vec2::new(40.0, 25.0);
        let mut view = View::default();
        for _ in 0..QUARTERS {
            let shown = view.to_shown(point, extent);
            assert_eq!(view.to_pattern(shown, extent), point, "{}", view.degrees());
            view.turn(Turn::Clockwise, extent);
        }
    }

    #[test]
    fn a_quarter_turn_clockwise_moves_the_top_left_to_the_top_right() {
        let extent = Vec2::new(300.0, 120.0);
        let mut view = View::default();

        view.turn(Turn::Clockwise, extent);

        assert_eq!(view.degrees(), 90);
        assert_eq!(view.to_shown(Vec2::ZERO, extent), Vec2::new(120.0, 0.0));
        assert_eq!(view.turned(extent), Vec2::new(120.0, 300.0));
    }

    #[test]
    fn four_turns_are_none_and_the_middle_stays() {
        let extent = Vec2::new(300.0, 120.0);
        let mut view = View::default();
        view.centre(Vec2::new(1000.0, 700.0), extent);
        let start = view;
        let middle = view.offset + view.turned(extent) / 2.0;

        for turn in [Turn::Clockwise, Turn::Clockwise, Turn::CounterClockwise] {
            view.turn(turn, extent);
            assert_eq!(view.offset + view.turned(extent) / 2.0, middle);
        }
        view.turn(Turn::Clockwise, extent);
        assert_eq!(view.degrees(), 180);
        view.turn(Turn::Clockwise, extent);
        view.turn(Turn::Clockwise, extent);

        assert_eq!(view, start);
    }

    #[test]
    fn a_zoom_stops_at_its_bounds() {
        let mut view = View::default();

        view.zoom(1000.0, Vec2::ZERO);

        assert_eq!(view.bead, MAX_BEAD);
    }
}
