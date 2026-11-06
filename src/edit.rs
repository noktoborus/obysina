//! Every change to a pattern, so that it can be undone (`RULE ui.undo`).

use obusina_pattern::{Bounds, Cell, ColourId, Grid, Palette, Pattern, Ruling};

/// One bead changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Change {
    /// Where.
    pub cell: Cell,
    /// The colour before.
    pub before: Option<ColourId>,
    /// The colour after.
    pub after: Option<ColourId>,
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
    /// The colours changed: added, changed, moved, removed, hidden. The
    /// beads do not change, so all the palettes before and after are kept.
    Palettes {
        /// The palettes before.
        before: Vec<Palette>,
        /// The palettes after.
        after: Vec<Palette>,
    },
    /// The sheet changed its size.
    Sheet {
        /// The sheet before.
        before: Option<Bounds>,
        /// The sheet after.
        after: Option<Bounds>,
    },
    /// The lines to count the beads by changed.
    Ruling {
        /// The ruling before.
        before: Ruling,
        /// The ruling after.
        after: Ruling,
    },
    /// Edits done as one step: done in order, undone in reverse.
    Many(Vec<Edit>),
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
            Edit::Palettes { after, .. } => put_back(pattern, after),
            Edit::Sheet { after, .. } => pattern.set_sheet(*after),
            Edit::Ruling { after, .. } => pattern.set_ruling(*after),
            Edit::Many(edits) => edits.iter().for_each(|edit| edit.apply(pattern)),
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
            Edit::Palettes { before, .. } => put_back(pattern, before),
            Edit::Sheet { before, .. } => pattern.set_sheet(*before),
            Edit::Ruling { before, .. } => pattern.set_ruling(*before),
            Edit::Many(edits) => edits.iter().rev().for_each(|edit| edit.revert(pattern)),
        }
    }
}

/// Puts back palettes. The history undoes and redoes in order, so the
/// beads are those the palettes were kept with, and a refusal is a fault of
/// the history.
fn put_back(pattern: &mut Pattern, palettes: &[Palette]) {
    if let Err(error) = pattern.set_palettes(palettes.to_vec()) {
        log::error!("the palettes are not put back: {error}");
    }
}

/// What was done and what was undone.
#[derive(Debug, Default)]
pub struct History {
    done: Vec<Edit>,
    undone: Vec<Edit>,
    /// The edits of a batch while it is open; they become one step when it
    /// ends.
    batch: Option<Vec<Edit>>,
    /// The columns and rows a sheet shrinks to at least when a bead goes:
    /// the first screen. `None` until it is known; a sheet then does not
    /// shrink.
    pub(crate) least_sheet: Option<(usize, usize)>,
}

impl History {
    /// Applies an edit and keeps it.
    pub fn apply(&mut self, pattern: &mut Pattern, edit: Edit) {
        edit.apply(pattern);
        self.keep(edit);
    }

    /// Keeps an edit already applied: in the open batch, or as a step.
    fn keep(&mut self, edit: Edit) {
        match &mut self.batch {
            Some(batch) => batch.push(edit),
            None => {
                self.done.push(edit);
                self.undone.clear();
            }
        }
    }

    /// Opens a batch: the edits kept until it ends are one step. Nothing
    /// changes when a batch is open already.
    pub fn begin(&mut self) {
        self.batch.get_or_insert_with(Vec::new);
    }

    /// Ends the batch as one step; an empty batch is no step. Nothing
    /// changes when no batch is open.
    pub fn end(&mut self) {
        let Some(mut edits) = self.batch.take() else {
            return;
        };
        match edits.len() {
            0 => {}
            1 => self.keep(edits.remove(0)),
            _ => self.keep(Edit::Many(edits)),
        }
    }

    /// Puts colours in cells as one step; a cell that has its colour already
    /// is left out, and nothing is kept when no cell changes. A bead put past
    /// the sheet grows the sheet to hold it; a bead taken away shrinks it, in
    /// the same step ([`History::fit_sheet`]).
    pub fn paint(&mut self, pattern: &mut Pattern, cells: &[Cell], colour: Option<ColourId>) {
        let changes: Vec<Change> = cells
            .iter()
            .map(|cell| Change {
                cell: *cell,
                before: pattern.get(*cell),
                after: colour,
            })
            .filter(|change| change.before != change.after)
            .collect();
        if changes.is_empty() {
            return;
        }
        for change in &changes {
            pattern.set(change.cell, change.after);
        }
        let before = pattern.sheet();
        let after = before.map(|sheet| self.fit_sheet(sheet, pattern.bounds(), &changes));
        let beads = Edit::Beads(changes);
        if after == before {
            self.keep(beads);
            return;
        }
        let sheet = Edit::Sheet { before, after };
        sheet.apply(pattern);
        self.keep(Edit::Many(vec![beads, sheet]));
    }

    /// The sheet after `changes`, the beads now in the box `beads`. It grows
    /// to hold a bead put past it. When a bead is taken away it shrinks to
    /// the box of the beads, never below `least_sheet` and never above its
    /// size now, moving as little as it takes to hold them.
    fn fit_sheet(&self, sheet: Bounds, beads: Option<Bounds>, changes: &[Change]) -> Bounds {
        let grown = changes
            .iter()
            .filter(|change| change.after.is_some())
            .fold(sheet, |sheet, change| sheet.with(change.cell));
        let taken = changes.iter().any(|change| change.after.is_none());
        let Some((least_width, least_height)) = self.least_sheet.filter(|_| taken) else {
            return grown;
        };
        let (width, height) = beads.map_or((0, 0), |beads| (beads.width(), beads.height()));
        let width = width.max(least_width).min(grown.width());
        let height = height.max(least_height).min(grown.height());
        grown.resized_over(width, height, beads)
    }

    /// Changes the colours by `change` as one step. Nothing is kept when the
    /// change fails or changes nothing.
    pub fn change_colours<T>(
        &mut self,
        pattern: &mut Pattern,
        change: impl FnOnce(&mut Pattern) -> obusina_pattern::Result<T>,
    ) -> obusina_pattern::Result<T> {
        let before = pattern.palettes().to_vec();
        let done = change(pattern)?;
        self.keep_palettes(pattern, before);
        Ok(done)
    }

    /// Keeps as one step every change of the colours since `before`, which
    /// were made past the history while they were shown: the changes of a
    /// picker before "Accept".
    pub fn keep_palettes(&mut self, pattern: &Pattern, before: Vec<Palette>) {
        let after = pattern.palettes().to_vec();
        if after != before {
            self.keep(Edit::Palettes { before, after });
        }
    }

    /// Keeps as one step the change of the ruling since `before`, made past
    /// the history while it was shown: the numbers of the menu of the ruling
    /// before "Accept".
    pub fn keep_ruling(&mut self, pattern: &Pattern, before: Ruling) {
        let after = pattern.ruling();
        if after != before {
            self.keep(Edit::Ruling { before, after });
        }
    }

    /// Deletes the beads of a colour as one step; the colour stays.
    pub fn delete_beads(&mut self, pattern: &mut Pattern, id: ColourId) {
        let cells: Vec<Cell> = pattern
            .beads()
            .filter(|(_, bead)| *bead == id)
            .map(|(cell, _)| cell)
            .collect();
        self.paint(pattern, &cells, None);
    }

    /// Gives the sheet a new size as a step. Changes of the size one after
    /// another are one step: a number dragged changes it in every frame.
    pub fn resize_sheet(&mut self, pattern: &mut Pattern, after: Bounds) {
        let before = pattern.sheet();
        if before == Some(after) {
            return;
        }
        if self.undone.is_empty()
            && self.batch.is_none()
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

    /// True when there is a step to undo. While a batch is open there is
    /// none: its edits are no step yet.
    pub fn can_undo(&self) -> bool {
        self.batch.is_none() && !self.done.is_empty()
    }

    /// True when there is a step to redo, and no batch is open.
    pub fn can_redo(&self) -> bool {
        self.batch.is_none() && !self.undone.is_empty()
    }

    /// Undoes the last step.
    pub fn undo(&mut self, pattern: &mut Pattern) {
        if self.batch.is_some() {
            return;
        }
        if let Some(edit) = self.done.pop() {
            edit.revert(pattern);
            self.undone.push(edit);
        }
    }

    /// Does again the last step undone.
    pub fn redo(&mut self, pattern: &mut Pattern) {
        if self.batch.is_some() {
            return;
        }
        if let Some(edit) = self.undone.pop() {
            edit.apply(pattern);
            self.done.push(edit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use obusina_pattern::{Colour, DEFAULT_BEAD_RATIO, Rgb};

    const COLOUR: ColourId = ColourId(1);

    fn pattern() -> Pattern {
        let colour = Colour::new(COLOUR, Rgb([0, 0, 0]));
        Pattern::new(Grid::Loom, DEFAULT_BEAD_RATIO, vec![colour]).expect("a pattern")
    }

    #[test]
    fn a_painting_is_undone_as_one_step() {
        let mut pattern = pattern();
        let empty = pattern.clone();
        let mut history = History::default();
        let cells: Vec<Cell> = (0..3).map(|column| Cell { column, row: 0 }).collect();
        history.paint(&mut pattern, &cells, Some(COLOUR));
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

        history.paint(&mut pattern, &[Cell { column: 0, row: 0 }], Some(COLOUR));

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

    fn two_colours() -> Pattern {
        let colours = vec![
            Colour::new(COLOUR, Rgb([0, 0, 0])),
            Colour::new(ColourId(2), Rgb([255, 255, 255])),
        ];
        Pattern::new(Grid::Loom, DEFAULT_BEAD_RATIO, colours).expect("a pattern")
    }

    /// A move to a new palette is one step, undone and redone.
    #[test]
    fn a_change_of_the_colours_is_undone() {
        let mut pattern = two_colours();
        let start = pattern.clone();
        let mut history = History::default();

        history
            .change_colours(&mut pattern, |pattern| pattern.move_colour(0, 1, None))
            .expect("the colour moves");
        let moved = pattern.clone();
        assert_eq!(moved.palettes().len(), 2);

        history.undo(&mut pattern);
        assert_eq!(pattern, start);
        history.redo(&mut pattern);
        assert_eq!(pattern, moved);
    }

    /// A removed colour comes back by undo, and its beads take it again.
    #[test]
    fn a_removed_colour_comes_back_with_its_beads() {
        let mut pattern = two_colours();
        let mut history = History::default();
        let cell = Cell { column: 0, row: 0 };
        history.paint(&mut pattern, &[cell], Some(COLOUR));

        history
            .change_colours(&mut pattern, |pattern| pattern.remove_colour(0, 0))
            .expect("the colour goes");
        assert!(pattern.colour(COLOUR).and_then(Colour::seen).is_none());

        history.undo(&mut pattern);
        assert!(pattern.colour(COLOUR).and_then(Colour::seen).is_some());
        assert_eq!(pattern.get(cell), Some(COLOUR));
    }

    /// The changes of a picker, made past the history, are one step.
    #[test]
    fn the_changes_of_a_picker_are_one_step() {
        let mut pattern = two_colours();
        let start = pattern.clone();
        let mut history = History::default();
        let before = pattern.palettes().to_vec();
        for grey in [10, 20, 30] {
            pattern
                .put_colour(0, 0, Rgb([grey; 3]))
                .expect("the place is there");
        }

        history.keep_palettes(&pattern, before);
        history.undo(&mut pattern);

        assert_eq!(pattern, start);
        assert!(!history.can_undo());
    }

    /// A change that fails or changes nothing is no step.
    #[test]
    fn a_change_of_nothing_is_no_step() {
        let mut pattern = two_colours();
        let mut history = History::default();

        let failed = history.change_colours(&mut pattern, |pattern| pattern.remove_colour(0, 5));
        history
            .change_colours(&mut pattern, |pattern| {
                pattern.set_hidden(ColourId(9), true);
                Ok(())
            })
            .expect("nothing to hide");

        assert!(failed.is_err());
        assert!(!history.can_undo());
    }

    /// Deleting the beads of a colour is one step; the colour stays.
    #[test]
    fn deleting_the_beads_is_undone() {
        let mut pattern = two_colours();
        let mut history = History::default();
        let cells: Vec<Cell> = (0..3).map(|column| Cell { column, row: 0 }).collect();
        history.paint(&mut pattern, &cells, Some(COLOUR));
        history.paint(
            &mut pattern,
            &[Cell { column: 5, row: 0 }],
            Some(ColourId(2)),
        );

        history.delete_beads(&mut pattern, COLOUR);
        assert_eq!(pattern.count(COLOUR), 0);
        assert_eq!(pattern.count(ColourId(2)), 1);
        assert!(pattern.colour(COLOUR).is_some());

        history.undo(&mut pattern);
        assert_eq!(pattern.count(COLOUR), 3);
    }

    /// A box from `(column, row)` to `(column, row)`, both in.
    fn bounds(min: (i32, i32), max: (i32, i32)) -> Bounds {
        Bounds {
            min: Cell {
                column: min.0,
                row: min.1,
            },
            max: Cell {
                column: max.0,
                row: max.1,
            },
        }
    }

    /// A bead taken away shrinks the sheet to the beads, not below the
    /// least size; a sheet smaller than that does not grow.
    #[test]
    fn a_sheet_shrinks_to_the_beads_and_never_grows_when_a_bead_goes() {
        let mut history = History {
            least_sheet: Some((4, 4)),
            ..History::default()
        };
        let mut pattern = pattern();
        pattern.set_sheet(Some(bounds((0, 0), (19, 9))));
        let row: Vec<Cell> = (0..10).map(|column| Cell { column, row: 0 }).collect();
        history.paint(&mut pattern, &row, Some(COLOUR));

        history.paint(&mut pattern, &row[5..], None);
        assert_eq!(pattern.sheet(), Some(bounds((0, 0), (4, 3))));

        history.paint(&mut pattern, &row[..5], None);
        assert_eq!(pattern.sheet(), Some(bounds((0, 0), (3, 3))));

        pattern.set_sheet(Some(bounds((0, 0), (1, 1))));
        history.paint(&mut pattern, &row[..1], Some(COLOUR));
        history.paint(&mut pattern, &row[..1], None);
        assert_eq!(pattern.sheet(), Some(bounds((0, 0), (1, 1))));
    }
}
