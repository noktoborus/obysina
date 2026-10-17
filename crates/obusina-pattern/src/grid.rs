//! The kinds of grid: where each bead stands.
//!
//! A position is in bead units: one bead is `ratio` wide and one high. The
//! canvas multiplies by the size it draws a bead at.

use serde::{Deserialize, Serialize};

/// The place of one bead: column and row, from zero, from the top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Column.
    pub column: usize,
    /// Row.
    pub row: usize,
}

/// How the beads of a pattern stand to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grid {
    /// Loom and square stitch: rows and columns line up.
    Loom,
    /// Peyote: every odd column is half a bead lower.
    Peyote,
    /// Brick stitch: every odd row is half a bead to the right.
    Brick,
}

impl Grid {
    /// Every kind, in the order they are offered.
    pub const ALL: [Grid; 3] = [Grid::Loom, Grid::Peyote, Grid::Brick];

    /// The top left corner of a bead, in bead units.
    pub fn origin(self, cell: Cell, ratio: f32) -> (f32, f32) {
        let x = cell.column as f32 * ratio;
        let y = cell.row as f32;
        match self {
            Grid::Loom => (x, y),
            Grid::Peyote => (x, y + half(cell.column)),
            Grid::Brick => (x + half(cell.row) * ratio, y),
        }
    }

    /// The size of the whole grid, in bead units.
    pub fn extent(self, width: usize, height: usize, ratio: f32) -> (f32, f32) {
        let x = width as f32 * ratio;
        let y = height as f32;
        let shifted = |count: usize| if count > 1 { 0.5 } else { 0.0 };
        match self {
            Grid::Loom => (x, y),
            Grid::Peyote => (x, y + shifted(width)),
            Grid::Brick => (x + shifted(height) * ratio, y),
        }
    }

    /// The bead under a point given in bead units, if there is one.
    pub fn cell_at(self, x: f32, y: f32, width: usize, height: usize, ratio: f32) -> Option<Cell> {
        let column = match self {
            Grid::Brick => {
                let row = index(y)?;
                index(x / ratio - half(row))?
            }
            Grid::Loom | Grid::Peyote => index(x / ratio)?,
        };
        let row = match self {
            Grid::Peyote => index(y - half(column))?,
            Grid::Loom | Grid::Brick => index(y)?,
        };
        (column < width && row < height).then_some(Cell { column, row })
    }

    /// The beads that touch a bead, inside a grid of `width` by `height`.
    ///
    /// Beside a shifted column or row, a bead touches two beads: the one at
    /// its own index and the one half a bead the other way.
    pub fn neighbours(self, cell: Cell, width: usize, height: usize) -> Vec<Cell> {
        let Cell { column, row } = cell;
        let mut found = Vec::with_capacity(6);
        let mut add = |column: Option<usize>, row: Option<usize>| {
            if let (Some(column), Some(row)) = (column, row)
                && column < width
                && row < height
            {
                found.push(Cell { column, row });
            }
        };
        let before = |index: usize| index.checked_sub(1);
        let after = |index: usize| Some(index + 1);
        let same = Some;
        match self {
            Grid::Loom => {
                add(before(column), same(row));
                add(after(column), same(row));
                add(same(column), before(row));
                add(same(column), after(row));
            }
            Grid::Peyote => {
                add(same(column), before(row));
                add(same(column), after(row));
                let other = if column % 2 == 0 {
                    before(row)
                } else {
                    after(row)
                };
                for side in [before(column), after(column)] {
                    add(side, same(row));
                    add(side, other);
                }
            }
            Grid::Brick => {
                add(before(column), same(row));
                add(after(column), same(row));
                let other = if row % 2 == 0 {
                    before(column)
                } else {
                    after(column)
                };
                for side in [before(row), after(row)] {
                    add(same(column), side);
                    add(other, side);
                }
            }
        }
        found
    }
}

/// Half a bead for an odd index, nothing for an even one.
fn half(index: usize) -> f32 {
    if index % 2 == 1 { 0.5 } else { 0.0 }
}

/// The whole number of beads before a coordinate, none before the start.
fn index(value: f32) -> Option<usize> {
    (value >= 0.0).then(|| value.floor() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATIO: f32 = 0.8;

    #[test]
    fn a_loom_bead_is_found_where_it_stands() {
        let cell = Cell { column: 3, row: 2 };
        let (x, y) = Grid::Loom.origin(cell, RATIO);

        assert_eq!(
            Grid::Loom.cell_at(x + 0.1, y + 0.1, 5, 5, RATIO),
            Some(cell)
        );
    }

    #[test]
    fn an_odd_peyote_column_is_half_a_bead_lower() {
        let (_, even) = Grid::Peyote.origin(Cell { column: 0, row: 1 }, RATIO);
        let (_, odd) = Grid::Peyote.origin(Cell { column: 1, row: 1 }, RATIO);

        assert_eq!(odd - even, 0.5);
        assert_eq!(
            Grid::Peyote.cell_at(RATIO + 0.1, 0.2, 5, 5, RATIO),
            None,
            "the top of an odd column is empty"
        );
    }

    #[test]
    fn an_odd_brick_row_is_half_a_bead_to_the_right() {
        let (x, y) = Grid::Brick.origin(Cell { column: 0, row: 1 }, RATIO);

        assert_eq!(x, 0.5 * RATIO);
        assert_eq!(
            Grid::Brick.cell_at(x + 0.1, y + 0.1, 5, 5, RATIO),
            Some(Cell { column: 0, row: 1 })
        );
        assert_eq!(Grid::Brick.cell_at(0.1, y + 0.1, 5, 5, RATIO), None);
    }

    #[test]
    fn a_peyote_bead_touches_six() {
        let even = Grid::Peyote.neighbours(Cell { column: 2, row: 2 }, 5, 5);
        let odd = Grid::Peyote.neighbours(Cell { column: 1, row: 2 }, 5, 5);

        assert_eq!(even.len(), 6);
        assert!(even.contains(&Cell { column: 1, row: 1 }));
        assert!(!even.contains(&Cell { column: 1, row: 3 }));
        assert!(odd.contains(&Cell { column: 0, row: 3 }));
        assert!(!odd.contains(&Cell { column: 0, row: 1 }));
    }

    #[test]
    fn a_neighbour_is_a_neighbour_both_ways() {
        for grid in Grid::ALL {
            for cell in (0..16).map(|index| Cell {
                column: index % 4,
                row: index / 4,
            }) {
                for other in grid.neighbours(cell, 4, 4) {
                    assert!(
                        grid.neighbours(other, 4, 4).contains(&cell),
                        "{grid:?} {cell:?} {other:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_corner_has_no_neighbour_outside() {
        let found = Grid::Brick.neighbours(Cell { column: 0, row: 0 }, 3, 3);

        assert!(found.iter().all(|cell| cell.column < 3 && cell.row < 3));
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn every_bead_is_found_at_its_centre() {
        for grid in Grid::ALL {
            for row in 0..4 {
                for column in 0..4 {
                    let cell = Cell { column, row };
                    let (x, y) = grid.origin(cell, RATIO);
                    let found = grid.cell_at(x + RATIO / 2.0, y + 0.5, 4, 4, RATIO);
                    assert_eq!(found, Some(cell), "{grid:?}");
                }
            }
        }
    }
}
