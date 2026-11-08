//! Where the field is moved to and how large a bead is drawn.

use egui::{Vec2, vec2};
use obusina_pattern::{Bounds, Grid};

/// The smallest height of a bead, in points.
pub const MIN_BEAD: f32 = 4.0;
/// The largest height of a bead, in points.
pub const MAX_BEAD: f32 = 120.0;
/// The height of a bead at zoom 1, in points: a new view starts at it.
pub const DEFAULT_BEAD: f32 = 20.0;

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
/// It turns how the pattern is shown, never the pattern. A point of the
/// pattern is in points from the top left corner of cell (0, 0), before the
/// turn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// Where the top left corner of cell (0, 0) is shown, from the top left
    /// of the field.
    pub offset: Vec2,
    /// The height of a bead, in points.
    pub bead: f32,
    /// The pattern has been put in the middle of the field once.
    pub centred: bool,
    /// Quarter turns clockwise, 0 to 3.
    quarters: u8,
    /// The size of the field, as last drawn: a turn keeps its middle.
    pub(crate) field: Vec2,
}

impl Default for View {
    fn default() -> Self {
        View {
            offset: Vec2::ZERO,
            bead: DEFAULT_BEAD,
            centred: false,
            quarters: 0,
            field: Vec2::ZERO,
        }
    }
}

impl View {
    /// Shows the point `middle` of the pattern in the middle of a field of
    /// `field` points.
    pub fn centre(&mut self, field: Vec2, middle: Vec2) {
        self.offset = field / 2.0 - self.to_shown(middle);
        self.centred = true;
    }

    /// How far the field is turned clockwise: 0, 90, 180 or 270 degrees. Four
    /// quarter turns are a whole turn, 360, which is 0 again.
    pub fn degrees(&self) -> u16 {
        u16::from(self.quarters) * 90
    }

    /// Shows the field turned `degrees` clockwise, a whole number of quarters;
    /// another number is taken down to the quarter below it.
    pub fn set_degrees(&mut self, degrees: u16) {
        self.quarters = ((degrees / 90) % u16::from(QUARTERS)) as u8;
    }

    /// Turns the field by a quarter, keeping what is in the middle of the
    /// field where it stands.
    pub fn turn(&mut self, turn: Turn) {
        let anchor = self.field / 2.0;
        let middle = self.to_pattern(anchor - self.offset);
        self.quarters = match turn {
            Turn::Clockwise => (self.quarters + 1) % QUARTERS,
            Turn::CounterClockwise => (self.quarters + QUARTERS - 1) % QUARTERS,
        };
        self.offset = anchor - self.to_shown(middle);
    }

    /// Where a point of the pattern is shown, from where cell (0, 0) is
    /// shown, in points.
    pub fn to_shown(&self, point: Vec2) -> Vec2 {
        match self.quarters {
            0 => point,
            1 => vec2(-point.y, point.x),
            2 => -point,
            _ => vec2(point.y, -point.x),
        }
    }

    /// The point of the pattern shown at `shown`; the reverse of
    /// [`View::to_shown`].
    pub fn to_pattern(&self, shown: Vec2) -> Vec2 {
        match self.quarters {
            0 => shown,
            1 => vec2(shown.y, -shown.x),
            2 => -shown,
            _ => vec2(-shown.y, shown.x),
        }
    }

    /// The cells a screen of the size last drawn holds at zoom 1
    /// (`DEFAULT_BEAD`), about the cell in the middle of the screen: the
    /// size a field set by hand starts at.
    pub fn first_screen(&self, grid: Grid, ratio: f32) -> Bounds {
        let middle = self.to_pattern(self.field / 2.0 - self.offset) / self.bead;
        let centre = grid.cell_at(middle.x, middle.y, ratio);
        let shown = self.field / DEFAULT_BEAD;
        let (across, down) = match self.quarters % 2 {
            0 => (shown.x, shown.y),
            _ => (shown.y, shown.x),
        };
        let columns = (across / ratio).floor().max(1.0) as usize;
        let rows = down.floor().max(1.0) as usize;
        Bounds::of(centre).resized(columns, rows)
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

    const FIELD: Vec2 = Vec2::new(1000.0, 700.0);

    #[test]
    fn a_zoom_keeps_its_anchor_still() {
        let mut view = View {
            offset: Vec2::new(10.0, 10.0),
            ..View::default()
        };
        let anchor = Vec2::new(50.0, 30.0);
        let before = (anchor - view.offset) / view.bead;

        view.zoom(2.0, anchor);

        assert_eq!(view.bead, 40.0);
        assert_eq!((anchor - view.offset) / view.bead, before);
    }

    #[test]
    fn a_point_is_centred_in_the_field() {
        let mut view = View::default();

        view.centre(FIELD, Vec2::new(200.0, 300.0));

        assert_eq!(view.offset, Vec2::new(300.0, 50.0));
        assert!(view.centred);
    }

    #[test]
    fn a_point_comes_back_from_every_turn() {
        let point = Vec2::new(40.0, -25.0);
        let mut view = View {
            field: FIELD,
            ..View::default()
        };
        for _ in 0..QUARTERS {
            let shown = view.to_shown(point);
            assert_eq!(view.to_pattern(shown), point, "{}", view.degrees());
            view.turn(Turn::Clockwise);
        }
    }

    /// On a screen whose y grows down, a point to the right of cell (0, 0)
    /// comes below it.
    #[test]
    fn a_quarter_turn_clockwise_moves_the_right_down() {
        let mut view = View::default();

        view.turn(Turn::Clockwise);

        assert_eq!(view.degrees(), 90);
        assert_eq!(view.to_shown(Vec2::new(10.0, 0.0)), Vec2::new(0.0, 10.0));
    }

    #[test]
    fn four_turns_are_none_and_the_middle_stays() {
        let mut view = View {
            field: FIELD,
            ..View::default()
        };
        view.centre(FIELD, Vec2::new(120.0, 80.0));
        let start = view;
        let middle = view.to_pattern(FIELD / 2.0 - view.offset);

        for turn in [Turn::Clockwise, Turn::Clockwise, Turn::CounterClockwise] {
            view.turn(turn);
            assert_eq!(view.to_pattern(FIELD / 2.0 - view.offset), middle);
        }
        view.turn(Turn::Clockwise);
        assert_eq!(view.degrees(), 180);
        view.turn(Turn::Clockwise);
        view.turn(Turn::Clockwise);

        assert_eq!(view, start);
    }

    /// The first screen is the cells of the screen at zoom 1, about its
    /// middle, whatever the zoom now; a quarter turn swaps its sides.
    #[test]
    fn the_first_screen_is_the_screen_at_zoom_one() {
        let mut view = View {
            field: FIELD,
            bead: 40.0,
            ..View::default()
        };
        view.centre(FIELD, Vec2::new(5.5, 3.5) * 40.0);
        let first = view.first_screen(Grid::Loom, 1.0);
        let wide = (
            (FIELD.x / DEFAULT_BEAD) as usize,
            (FIELD.y / DEFAULT_BEAD) as usize,
        );
        assert_eq!((first.width(), first.height()), wide);
        assert!(first.contains(obusina_pattern::Cell { column: 5, row: 3 }));

        view.turn(Turn::Clockwise);
        let turned = view.first_screen(Grid::Loom, 1.0);
        assert_eq!((turned.width(), turned.height()), (wide.1, wide.0));
    }

    #[test]
    fn a_zoom_stops_at_its_bounds() {
        let mut view = View::default();

        view.zoom(1000.0, Vec2::ZERO);

        assert_eq!(view.bead, MAX_BEAD);
    }
}
