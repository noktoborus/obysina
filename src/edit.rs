//! Every change to a pattern, so that it can be undone (`RULE ui.undo`).

use obusina_pattern::{Bounds, Cell, Grid, Pattern};

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
    /// Beads changed by one stroke.
    Beads(Vec<Change>),
    /// The kind of grid changed.
    Grid {
        /// The kind before.
        before: Grid,
        /// The kind after.
        after: Grid,
    },
    /// The sheet changed its size.
    Sheet {
        /// The sheet before.
        before: Option<Bounds>,
        /// The sheet after.
        after: Option<Bounds>,
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
            Edit::Sheet { after, .. } => pattern.set_sheet(*after),
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
            Edit::Sheet { before, .. } => pattern.set_sheet(*before),
        }
    }
}

/// What was done and what was undone.
#[derive(Debug, Default)]
pub struct History {
    done: Vec<Edit>,
    undone: Vec<Edit>,
}

impl History {
    /// Applies an edit and keeps it.
    pub fn apply(&mut self, pattern: &mut Pattern, edit: Edit) {
        edit.apply(pattern);
        self.done.push(edit);
        self.undone.clear();
    }

    /// Puts colours in cells as one step; a cell that has its colour already
    /// is left out, and nothing is kept when no cell changes.
    pub fn paint(&mut self, pattern: &mut Pattern, cells: &[Cell], colour: Option<usize>) {
        let changes: Vec<Change> = cells
            .iter()
            .map(|cell| Change {
                cell: *cell,
                before: pattern.get(*cell),
                after: colour,
            })
            .filter(|change| change.before != change.after)
            .collect();
        if !changes.is_empty() {
            self.apply(pattern, Edit::Beads(changes));
        }
    }

    /// Gives the sheet a new size as a step. Changes of the size one after
    /// another are one step: a number dragged changes it in every frame.
    pub fn resize_sheet(&mut self, pattern: &mut Pattern, after: Bounds) {
        let before = pattern.sheet();
        if before == Some(after) {
            return;
        }
        if self.undone.is_empty()
            && let Some(Edit::Sheet { after: last, .. }) = self.done.last_mut()
        {
            *last = Some(after);
            pattern.set_sheet(Some(after));
            return;
        }
        self.apply(
            pattern,
            Edit::Sheet {
                before,
                after: Some(after),
            },
        );
    }

    /// True when there is a step to undo.
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    /// True when there is a step to redo.
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Undoes the last step.
    pub fn undo(&mut self, pattern: &mut Pattern) {
        if let Some(edit) = self.done.pop() {
            edit.revert(pattern);
            self.undone.push(edit);
        }
    }

    /// Does again the last step undone.
    pub fn redo(&mut self, pattern: &mut Pattern) {
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
        Pattern::new(Grid::Loom, DEFAULT_BEAD_RATIO, vec![bead]).expect("a pattern")
    }

    #[test]
    fn a_painting_is_undone_as_one_step() {
        let mut pattern = pattern();
        let empty = pattern.clone();
        let mut history = History::default();
        let cells: Vec<Cell> = (0..3).map(|column| Cell { column, row: 0 }).collect();
        history.paint(&mut pattern, &cells, Some(0));
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

        history.paint(&mut pattern, &[Cell { column: 0, row: 0 }], Some(0));

        assert!(!history.can_redo());
        assert_eq!(pattern.grid(), Grid::Loom);
    }

    #[test]
    fn a_bead_painted_its_own_colour_is_no_step() {
        let mut pattern = pattern();
        let mut history = History::default();

        history.paint(&mut pattern, &[Cell { column: 0, row: 0 }], None);

        assert!(!history.can_undo());
    }
}
