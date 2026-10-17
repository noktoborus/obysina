//! A pattern: a grid, a palette and a bead or an empty place in every cell.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{PatternError, Result};
use crate::grid::{Cell, Grid};
use crate::palette::{Bead, EMPTY_KEY};

/// The width of a bead to its height, for a pattern that says nothing else.
pub const DEFAULT_BEAD_RATIO: f32 = 1.0;

/// The next revision any pattern takes: one counter for all of them, so that
/// two patterns that are not clones never share a revision.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

/// A revision no pattern had before.
fn next_revision() -> u64 {
    NEXT_REVISION.fetch_add(1, Ordering::Relaxed)
}

/// A bead pattern.
///
/// Two patterns are equal when they hold the same beads; the revision is not
/// compared.
#[derive(Debug, Clone)]
pub struct Pattern {
    grid: Grid,
    width: usize,
    height: usize,
    ratio: f32,
    palette: Vec<Bead>,
    cells: Vec<Option<usize>>,
    revision: u64,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
            && self.width == other.width
            && self.height == other.height
            && self.ratio == other.ratio
            && self.palette == other.palette
            && self.cells == other.cells
    }
}

impl Pattern {
    /// An empty pattern.
    pub fn new(
        grid: Grid,
        width: usize,
        height: usize,
        ratio: f32,
        palette: Vec<Bead>,
    ) -> Result<Pattern> {
        if width == 0 || height == 0 || !(ratio.is_finite() && ratio > 0.0) {
            return Err(PatternError::Size {
                width,
                height,
                ratio,
            });
        }
        let mut seen = vec![EMPTY_KEY];
        for bead in &palette {
            if seen.contains(&bead.key) {
                return Err(PatternError::DuplicateKey { key: bead.key });
            }
            seen.push(bead.key);
        }
        Ok(Pattern {
            grid,
            width,
            height,
            ratio,
            palette,
            cells: vec![None; width * height],
            revision: next_revision(),
        })
    }

    /// The kind of grid.
    pub fn grid(&self) -> Grid {
        self.grid
    }

    /// Changes the kind of grid; the beads keep their cells.
    pub fn set_grid(&mut self, grid: Grid) {
        if self.grid != grid {
            self.grid = grid;
            self.revision = next_revision();
        }
    }

    /// Columns.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Rows.
    pub fn height(&self) -> usize {
        self.height
    }

    /// The width of a bead to its height.
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    /// A number that changes with every change of the pattern, and that no
    /// other pattern has unless it is a clone of this one: what a drawing of
    /// the pattern can be kept by.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The colours.
    pub fn palette(&self) -> &[Bead] {
        &self.palette
    }

    /// Every cell of the grid, row by row.
    pub fn cells(&self) -> impl Iterator<Item = Cell> + use<> {
        let width = self.width;
        (0..self.height).flat_map(move |row| (0..width).map(move |column| Cell { column, row }))
    }

    /// The colour of a bead, by its place in the palette; `None` for an empty
    /// place or a cell outside the grid.
    pub fn get(&self, cell: Cell) -> Option<usize> {
        self.slot(cell).and_then(|slot| self.cells[slot])
    }

    /// Puts a colour, or nothing, in a cell. Gives back what was there.
    ///
    /// A cell outside the grid or a colour outside the palette changes nothing.
    pub fn set(&mut self, cell: Cell, colour: Option<usize>) -> Option<usize> {
        let slot = self.slot(cell)?;
        if colour.is_some_and(|index| index >= self.palette.len()) {
            return self.cells[slot];
        }
        let before = std::mem::replace(&mut self.cells[slot], colour);
        if before != colour {
            self.revision = next_revision();
        }
        before
    }

    /// How many beads of each colour, in the order of the palette.
    pub fn counts(&self) -> Vec<usize> {
        let mut counts = vec![0; self.palette.len()];
        for index in self.cells.iter().flatten() {
            counts[*index] += 1;
        }
        counts
    }

    /// The bead at `cell` and every bead of its colour joined to it.
    pub fn region(&self, cell: Cell) -> Vec<Cell> {
        if self.slot(cell).is_none() {
            return Vec::new();
        }
        let colour = self.get(cell);
        let mut seen = vec![false; self.cells.len()];
        let mut found = Vec::new();
        let mut waiting = vec![cell];
        while let Some(next) = waiting.pop() {
            let Some(slot) = self.slot(next) else {
                continue;
            };
            if seen[slot] || self.cells[slot] != colour {
                continue;
            }
            seen[slot] = true;
            found.push(next);
            waiting.extend(self.grid.neighbours(next, self.width, self.height));
        }
        found
    }

    /// The place of a cell in `cells`.
    fn slot(&self, cell: Cell) -> Option<usize> {
        (cell.column < self.width && cell.row < self.height)
            .then_some(cell.row * self.width + cell.column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::Rgb;

    fn bead(key: char) -> Bead {
        Bead {
            key,
            rgb: Rgb([0, 0, 0]),
            catalog: None,
            code: None,
        }
    }

    #[test]
    fn a_bead_is_counted_by_its_colour() {
        let mut pattern = Pattern::new(
            Grid::Loom,
            3,
            2,
            DEFAULT_BEAD_RATIO,
            vec![bead('A'), bead('B')],
        )
        .expect("a pattern");
        pattern.set(Cell { column: 0, row: 0 }, Some(1));
        pattern.set(Cell { column: 2, row: 1 }, Some(1));
        pattern.set(Cell { column: 1, row: 1 }, Some(0));

        assert_eq!(pattern.counts(), [1, 2]);
        assert_eq!(pattern.set(Cell { column: 0, row: 0 }, None), Some(1));
        assert_eq!(pattern.counts(), [1, 1]);
    }

    #[test]
    fn a_region_stops_at_another_colour() {
        let mut pattern =
            Pattern::new(Grid::Loom, 3, 3, DEFAULT_BEAD_RATIO, vec![bead('A')]).expect("a pattern");
        for row in 0..3 {
            pattern.set(Cell { column: 1, row }, Some(0));
        }

        let region = pattern.region(Cell { column: 0, row: 0 });

        assert_eq!(region.len(), 3);
        assert!(region.iter().all(|cell| cell.column == 0));
    }

    #[test]
    fn a_change_gives_a_new_revision_and_no_change_none() {
        let mut pattern =
            Pattern::new(Grid::Loom, 2, 2, DEFAULT_BEAD_RATIO, vec![bead('A')]).expect("a pattern");
        let other =
            Pattern::new(Grid::Loom, 2, 2, DEFAULT_BEAD_RATIO, vec![bead('A')]).expect("a pattern");
        let first = pattern.revision();
        assert_ne!(first, other.revision());
        assert_eq!(pattern, other, "the revision is not compared");

        pattern.set(Cell { column: 0, row: 0 }, None);
        pattern.set_grid(Grid::Loom);
        assert_eq!(pattern.revision(), first);

        pattern.set(Cell { column: 0, row: 0 }, Some(0));
        let second = pattern.revision();
        assert_ne!(second, first);

        pattern.set_grid(Grid::Brick);
        assert_ne!(pattern.revision(), second);
    }

    #[test]
    fn a_key_used_twice_is_refused() {
        let error = Pattern::new(Grid::Loom, 1, 1, 1.0, vec![bead('A'), bead('A')]);

        assert!(matches!(
            error,
            Err(PatternError::DuplicateKey { key: 'A' })
        ));
    }

    #[test]
    fn the_empty_key_is_not_a_colour() {
        let error = Pattern::new(Grid::Loom, 1, 1, 1.0, vec![bead(EMPTY_KEY)]);

        assert!(matches!(error, Err(PatternError::DuplicateKey { .. })));
    }
}
