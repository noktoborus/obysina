//! Every change to a pattern, so that it can be undone (`RULE ui.undo`).

use obusina_pattern::{Cell, Grid, Pattern};

/// One bead changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    /// Where.
    pub cell: Cell,
    /// The colour before.
    pub before: Option<usize>,
    /// The colour after.
    pub after: Option<usize>,
}

/// One step of the history.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Beads changed by one stroke or one fill.
    Beads(Vec<Change>),
    /// The kind of grid changed.
    Grid {
        /// The kind before.
        before: Grid,
        /// The kind after.
        after: Grid,
    },
}

impl Edit {
    /// Applies the edit forwards.
    fn apply(&self, pattern: &mut Pattern) {
        match self {
            Edit::Beads(changes) => {
                for change in changes {
                    pattern.set(change.cell, change.after);
                }
            }
            Edit::Grid { after, .. } => pattern.set_grid(*after),
        }
    }

    /// Applies the edit backwards.
    fn revert(&self, pattern: &mut Pattern) {
        match self {
            Edit::Beads(changes) => {
                for change in changes.iter().rev() {
                    pattern.set(change.cell, change.before);
                }
            }
            Edit::Grid { before, .. } => pattern.set_grid(*before),
        }
    }
}

/// What was done and what was undone.
#[derive(Debug, Default)]
pub struct History {
    done: Vec<Edit>,
    undone: Vec<Edit>,
    stroke: Vec<Change>,
}

impl History {
    /// Applies an edit and keeps it.
    pub fn apply(&mut self, pattern: &mut Pattern, edit: Edit) {
        self.finish();
        edit.apply(pattern);
        self.done.push(edit);
        self.undone.clear();
    }

    /// Paints one bead as a part of the stroke that goes on.
    pub fn paint(&mut self, pattern: &mut Pattern, cell: Cell, colour: Option<usize>) {
        let before = pattern.get(cell);
        if before == colour {
            return;
        }
        pattern.set(cell, colour);
        self.stroke.push(Change {
            cell,
            before,
            after: colour,
        });
    }

    /// Ends the stroke that goes on: all its beads are one step.
    pub fn finish(&mut self) {
        if self.stroke.is_empty() {
            return;
        }
        self.done
            .push(Edit::Beads(std::mem::take(&mut self.stroke)));
        self.undone.clear();
    }

    /// Takes back the stroke that goes on, as if it never was.
    pub fn cancel(&mut self, pattern: &mut Pattern) {
        for change in std::mem::take(&mut self.stroke).iter().rev() {
            pattern.set(change.cell, change.before);
        }
    }

    /// True when there is a step to undo.
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty() || !self.stroke.is_empty()
    }

    /// True when there is a step to redo.
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Undoes the last step.
    pub fn undo(&mut self, pattern: &mut Pattern) {
        self.finish();
        if let Some(edit) = self.done.pop() {
            edit.revert(pattern);
            self.undone.push(edit);
        }
    }

    /// Does again the last step undone.
    pub fn redo(&mut self, pattern: &mut Pattern) {
        self.finish();
        if let Some(edit) = self.undone.pop() {
            edit.apply(pattern);
            self.done.push(edit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obusina_pattern::{Bead, DEFAULT_BEAD_RATIO, Rgb};

    fn pattern() -> Pattern {
        let bead = Bead {
            key: 'A',
            rgb: Rgb([0, 0, 0]),
            catalog: None,
            code: None,
        };
        Pattern::new(Grid::Loom, 4, 4, DEFAULT_BEAD_RATIO, vec![bead]).expect("a pattern")
    }

    #[test]
    fn a_stroke_is_undone_as_one_step() {
        let mut pattern = pattern();
        let empty = pattern.clone();
        let mut history = History::default();
        for column in 0..3 {
            history.paint(&mut pattern, Cell { column, row: 0 }, Some(0));
        }
        history.finish();
        let painted = pattern.clone();

        history.undo(&mut pattern);
        assert_eq!(pattern, empty);

        history.redo(&mut pattern);
        assert_eq!(pattern, painted);
    }

    #[test]
    fn a_new_step_forgets_what_was_undone() {
        let mut pattern = pattern();
        let mut history = History::default();
        history.apply(
            &mut pattern,
            Edit::Grid {
                before: Grid::Loom,
                after: Grid::Peyote,
            },
        );
        history.undo(&mut pattern);

        history.paint(&mut pattern, Cell { column: 0, row: 0 }, Some(0));
        history.finish();

        assert!(!history.can_redo());
        assert_eq!(pattern.grid(), Grid::Loom);
    }

    #[test]
    fn a_cancelled_stroke_leaves_no_trace() {
        let mut pattern = pattern();
        let empty = pattern.clone();
        let mut history = History::default();
        history.paint(&mut pattern, Cell { column: 1, row: 1 }, Some(0));

        history.cancel(&mut pattern);
        history.finish();

        assert_eq!(pattern, empty);
        assert!(!history.can_undo());
    }

    #[test]
    fn a_bead_painted_its_own_colour_is_no_step() {
        let mut pattern = pattern();
        let mut history = History::default();

        history.paint(&mut pattern, Cell { column: 0, row: 0 }, None);
        history.finish();

        assert!(!history.can_undo());
    }
}
