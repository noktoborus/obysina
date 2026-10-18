//! A pattern: a grid, a palette and the beads put on a field with no end.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{PatternError, Result};
use crate::grid::{Bounds, Cell, Grid};
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
/// Only the cells that hold a bead are kept; every other cell of the field
/// is empty. Two patterns are equal when they hold the same beads; the
/// revision is not compared.
#[derive(Debug, Clone)]
pub struct Pattern {
    grid: Grid,
    ratio: f32,
    palette: Vec<Bead>,
    beads: BTreeMap<Cell, usize>,
    revision: u64,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
            && self.ratio == other.ratio
            && self.palette == other.palette
            && self.beads == other.beads
    }
}

impl Pattern {
    /// A pattern with no beads.
    pub fn new(grid: Grid, ratio: f32, palette: Vec<Bead>) -> Result<Pattern> {
        if !(ratio.is_finite() && ratio > 0.0) {
            return Err(PatternError::Ratio { ratio });
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
            ratio,
            palette,
            beads: BTreeMap::new(),
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

    /// The active area: the box around the beads. `None` when there are none.
    pub fn bounds(&self) -> Option<Bounds> {
        let mut cells = self.beads.keys().copied();
        let first = cells.next()?;
        Some(cells.fold(Bounds::of(first), Bounds::with))
    }

    /// Every bead and its colour.
    pub fn beads(&self) -> impl Iterator<Item = (Cell, usize)> + '_ {
        self.beads.iter().map(|(cell, colour)| (*cell, *colour))
    }

    /// The colour of a bead, by its place in the palette; `None` for an empty
    /// cell.
    pub fn get(&self, cell: Cell) -> Option<usize> {
        self.beads.get(&cell).copied()
    }

    /// Puts a colour, or nothing, in a cell. Gives back what was there.
    ///
    /// A colour outside the palette changes nothing.
    pub fn set(&mut self, cell: Cell, colour: Option<usize>) -> Option<usize> {
        if colour.is_some_and(|index| index >= self.palette.len()) {
            return self.get(cell);
        }
        let before = match colour {
            Some(index) => self.beads.insert(cell, index),
            None => self.beads.remove(&cell),
        };
        if before != colour {
            self.revision = next_revision();
        }
        before
    }

    /// How many beads of each colour, in the order of the palette.
    pub fn counts(&self) -> Vec<usize> {
        let mut counts = vec![0; self.palette.len()];
        for index in self.beads.values() {
            counts[*index] += 1;
        }
        counts
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

    fn pattern() -> Pattern {
        Pattern::new(Grid::Loom, DEFAULT_BEAD_RATIO, vec![bead('A'), bead('B')]).expect("a pattern")
    }

    #[test]
    fn a_bead_is_counted_by_its_colour() {
        let mut pattern = pattern();
        pattern.set(Cell { column: 0, row: 0 }, Some(1));
        pattern.set(Cell { column: 2, row: 1 }, Some(1));
        pattern.set(Cell { column: -1, row: 1 }, Some(0));

        assert_eq!(pattern.counts(), [1, 2]);
        assert_eq!(pattern.set(Cell { column: 0, row: 0 }, None), Some(1));
        assert_eq!(pattern.counts(), [1, 1]);
    }

    #[test]
    fn the_active_area_is_the_box_around_the_beads() {
        let mut pattern = pattern();
        assert_eq!(pattern.bounds(), None);

        pattern.set(Cell { column: 3, row: -2 }, Some(0));
        pattern.set(Cell { column: -1, row: 4 }, Some(1));
        let bounds = pattern.bounds().expect("there are beads");
        assert_eq!(
            bounds.min,
            Cell {
                column: -1,
                row: -2
            }
        );
        assert_eq!(bounds.max, Cell { column: 3, row: 4 });

        pattern.set(Cell { column: -1, row: 4 }, None);
        assert_eq!(
            pattern.bounds(),
            Some(Bounds::of(Cell { column: 3, row: -2 }))
        );
    }

    #[test]
    fn a_change_gives_a_new_revision_and_no_change_none() {
        let mut pattern = pattern();
        let other = self::pattern();
        let first = pattern.revision();
        assert_ne!(first, other.revision());
        assert_eq!(pattern, other, "the revision is not compared");

        pattern.set(Cell { column: 0, row: 0 }, None);
        pattern.set_grid(Grid::Loom);
        assert_eq!(pattern.revision(), first);

        pattern.set(Cell { column: 0, row: 0 }, Some(0));
        let second = pattern.revision();
        assert_ne!(second, first);

        pattern.set_grid(Grid::Peyote);
        assert_ne!(pattern.revision(), second);
    }

    #[test]
    fn a_key_used_twice_is_refused() {
        let error = Pattern::new(Grid::Loom, 1.0, vec![bead('A'), bead('A')]);

        assert!(matches!(
            error,
            Err(PatternError::DuplicateKey { key: 'A' })
        ));
    }

    #[test]
    fn the_empty_key_is_not_a_colour() {
        let error = Pattern::new(Grid::Loom, 1.0, vec![bead(EMPTY_KEY)]);

        assert!(matches!(error, Err(PatternError::DuplicateKey { .. })));
    }
}
