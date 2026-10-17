//! Where the field is moved to and how large a bead is drawn.

use egui::Vec2;

/// The smallest height of a bead, in points.
pub const MIN_BEAD: f32 = 4.0;
/// The largest height of a bead, in points.
pub const MAX_BEAD: f32 = 120.0;

/// How the field is moved and zoomed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    /// The top left of the pattern, from the top left of the field.
    pub offset: Vec2,
    /// The height of a bead, in points.
    pub bead: f32,
}

impl Default for View {
    fn default() -> Self {
        View {
            offset: Vec2::splat(16.0),
            bead: 20.0,
        }
    }
}

impl View {
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
        };
        let anchor = Vec2::new(50.0, 30.0);
        let before = (anchor - view.offset) / view.bead;

        view.zoom(2.0, anchor);

        assert_eq!(view.bead, 40.0);
        assert_eq!((anchor - view.offset) / view.bead, before);
    }

    #[test]
    fn a_zoom_stops_at_its_bounds() {
        let mut view = View::default();

        view.zoom(1000.0, Vec2::ZERO);

        assert_eq!(view.bead, MAX_BEAD);
    }
}
