//! Groups of buttons broken into lines that fit the room of a panel
//! (`obusina_panel::Room`).
//!
//! ```text
//! a row, room for 7 buttons:     [U R] [L R] [P F K]
//!                                [G] [S]
//! ```
//!
//! A group is never split. A group longer than the room has a line of its
//! own.

use std::ops::Range;

use obusina_panel::{Orientation, Room};

use crate::ui::button;

/// The length of `count` buttons side by side along `orientation`.
pub fn length(orientation: Orientation, count: usize, spacing: egui::Vec2) -> f32 {
    let (size, space) = along(orientation, button::SIZE, spacing);
    count as f32 * (size + space) - space
}

/// The length between two groups: the spacing after the last button of the
/// first and the [`crate::ui::tools::gap`].
pub fn gap_length(orientation: Orientation, spacing: egui::Vec2) -> f32 {
    let (_, space) = along(orientation, button::SIZE, spacing);
    space * 3.0
}

/// The groups of `counts` buttons each, as ranges of `counts`, one range a
/// line.
pub fn split(room: Room, counts: &[usize], spacing: egui::Vec2) -> Vec<Range<usize>> {
    let gap = gap_length(room.orientation, spacing);
    let mut lines = Vec::new();
    let mut start = 0;
    let mut used = 0.0;
    for (index, count) in counts.iter().copied().enumerate() {
        let group = length(room.orientation, count, spacing);
        if index > start && used + gap + group > room.length {
            lines.push(start..index);
            start = index;
            used = 0.0;
        }
        used += if index > start { gap + group } else { group };
    }
    if start < counts.len() {
        lines.push(start..counts.len());
    }
    lines
}

/// The layout of the lines, stacked across the panel.
pub fn across(orientation: Orientation) -> egui::Layout {
    match orientation {
        Orientation::Horizontal => egui::Layout::top_down(egui::Align::Min),
        Orientation::Vertical => egui::Layout::left_to_right(egui::Align::Min),
    }
}

/// The layout of one line, along the panel.
pub fn along_layout(orientation: Orientation) -> egui::Layout {
    match orientation {
        Orientation::Horizontal => egui::Layout::left_to_right(egui::Align::Min),
        Orientation::Vertical => egui::Layout::top_down(egui::Align::Min),
    }
}

/// The parts of `size` and `spacing` that run along `orientation`.
fn along(orientation: Orientation, size: egui::Vec2, spacing: egui::Vec2) -> (f32, f32) {
    match orientation {
        Orientation::Horizontal => (size.x, spacing.x),
        Orientation::Vertical => (size.y, spacing.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACING: egui::Vec2 = egui::vec2(4.0, 4.0);
    const COUNTS: [usize; 5] = [2, 2, 3, 1, 1];

    fn room(length: f32) -> Room {
        Room {
            orientation: Orientation::Horizontal,
            length,
        }
    }

    #[test]
    fn a_long_room_holds_one_line() {
        assert_eq!(split(room(10_000.0), &COUNTS, SPACING), vec![0..5]);
    }

    #[test]
    fn a_group_is_never_split() {
        let two_groups = length(Orientation::Horizontal, 2, SPACING) * 2.0
            + gap_length(Orientation::Horizontal, SPACING);
        let lines = split(room(two_groups), &COUNTS, SPACING);
        assert_eq!(lines, vec![0..2, 2..4, 4..5]);
    }

    #[test]
    fn a_group_longer_than_the_room_has_a_line_of_its_own() {
        assert_eq!(
            split(room(1.0), &COUNTS, SPACING),
            vec![0..1, 1..2, 2..3, 3..4, 4..5]
        );
    }
}
