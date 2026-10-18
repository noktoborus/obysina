//! The kinds of grid: where each bead stands on a field with no end.
//!
//! A position is in bead units: one bead is `ratio` wide and one high. Cell
//! (0, 0) has its top left corner at (0, 0). The canvas multiplies by the
//! size it draws a bead at.

use serde::{Deserialize, Serialize};

/// The place of one bead on the field: column and row, either side of zero.
/// The default is cell (0, 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    /// Column; it grows to the right.
    pub column: i32,
    /// Row; it grows down.
    pub row: i32,
}

/// A box of cells, both corners in. The default is the box of cell (0, 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bounds {
    /// The top left cell.
    pub min: Cell,
    /// The bottom right cell.
    pub max: Cell,
}

impl Bounds {
    /// The box of one cell.
    pub fn of(cell: Cell) -> Bounds {
        Bounds {
            min: cell,
            max: cell,
        }
    }

    /// The box grown to hold `cell` too.
    pub fn with(self, cell: Cell) -> Bounds {
        Bounds {
            min: Cell {
                column: self.min.column.min(cell.column),
                row: self.min.row.min(cell.row),
            },
            max: Cell {
                column: self.max.column.max(cell.column),
                row: self.max.row.max(cell.row),
            },
        }
    }

    /// Columns.
    pub fn width(&self) -> usize {
        self.max.column.abs_diff(self.min.column) as usize + 1
    }

    /// Rows.
    pub fn height(&self) -> usize {
        self.max.row.abs_diff(self.min.row) as usize + 1
    }

    /// True when `cell` is in the box.
    pub fn contains(&self, cell: Cell) -> bool {
        (self.min.column..=self.max.column).contains(&cell.column)
            && (self.min.row..=self.max.row).contains(&cell.row)
    }

    /// True when `other` lies wholly in the box.
    pub fn holds(&self, other: Bounds) -> bool {
        self.contains(other.min) && self.contains(other.max)
    }

    /// Every cell of the box, row by row.
    pub fn cells(self) -> impl Iterator<Item = Cell> {
        (self.min.row..=self.max.row).flat_map(move |row| {
            (self.min.column..=self.max.column).map(move |column| Cell { column, row })
        })
    }
}

/// How the beads of a pattern stand to each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grid {
    /// Loom and square stitch: rows and columns line up.
    Loom,
    /// Peyote: every odd column of the field is half a bead lower.
    Peyote,
}

impl Grid {
    /// Every kind, in the order they are offered.
    pub const ALL: [Grid; 2] = [Grid::Loom, Grid::Peyote];

    /// The top left corner of a bead, in bead units.
    pub fn origin(self, cell: Cell, ratio: f32) -> (f32, f32) {
        let x = cell.column as f32 * ratio;
        let y = cell.row as f32;
        match self {
            Grid::Loom => (x, y),
            Grid::Peyote => (x, y + half(cell.column)),
        }
    }

    /// The bead under a point given in bead units.
    pub fn cell_at(self, x: f32, y: f32, ratio: f32) -> Cell {
        let column = (x / ratio).floor() as i32;
        let lower = match self {
            Grid::Loom => 0.0,
            Grid::Peyote => half(column),
        };
        let row = (y - lower).floor() as i32;
        Cell { column, row }
    }

    /// The cells whose beads lie, even in part, between two points given in
    /// bead units.
    pub fn cells_in(self, min: (f32, f32), max: (f32, f32), ratio: f32) -> Bounds {
        let lower = match self {
            Grid::Loom => 0.0,
            Grid::Peyote => 0.5,
        };
        Bounds {
            min: Cell {
                column: (min.0 / ratio).floor() as i32,
                row: (min.1 - lower).floor() as i32,
            },
            max: Cell {
                column: (max.0 / ratio).floor() as i32,
                row: max.1.floor() as i32,
            },
        }
    }

    /// The corners of the beads of `bounds`, in bead units: top left and
    /// bottom right.
    pub fn span(self, bounds: Bounds, ratio: f32) -> ((f32, f32), (f32, f32)) {
        let left = bounds.min.column as f32 * ratio;
        let right = (bounds.max.column + 1) as f32 * ratio;
        let (top, bottom) = match self {
            Grid::Loom => (0.0, 0.0),
            Grid::Peyote => {
                let single = bounds.width() == 1;
                let lower = half(bounds.min.column);
                let top = if single { lower } else { 0.0 };
                let bottom = if single { lower } else { 0.5 };
                (top, bottom)
            }
        };
        (
            (left, bounds.min.row as f32 + top),
            (right, (bounds.max.row + 1) as f32 + bottom),
        )
    }
}

/// Half a bead for an odd column of the field, nothing for an even one. A
/// column left of zero takes the parity of its place, not of its distance.
fn half(column: i32) -> f32 {
    if column.rem_euclid(2) == 1 { 0.5 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATIO: f32 = 0.8;

    #[test]
    fn a_loom_bead_is_found_where_it_stands() {
        let cell = Cell { column: 3, row: 2 };
        let (x, y) = Grid::Loom.origin(cell, RATIO);

        assert_eq!(Grid::Loom.cell_at(x + 0.1, y + 0.1, RATIO), cell);
    }

    #[test]
    fn an_odd_peyote_column_is_half_a_bead_lower() {
        let (_, even) = Grid::Peyote.origin(Cell { column: 0, row: 1 }, RATIO);
        let (_, odd) = Grid::Peyote.origin(Cell { column: 1, row: 1 }, RATIO);

        assert_eq!(odd - even, 0.5);
        assert_eq!(
            Grid::Peyote.cell_at(RATIO + 0.1, 0.2, RATIO),
            Cell { column: 1, row: -1 },
            "the top of an odd column holds the bead above"
        );
    }

    /// Column -1 is odd: a shift by an odd number of columns would lift it.
    #[test]
    fn a_column_left_of_zero_keeps_the_parity_of_its_place() {
        let (_, minus_one) = Grid::Peyote.origin(Cell { column: -1, row: 0 }, RATIO);
        let (_, minus_two) = Grid::Peyote.origin(Cell { column: -2, row: 0 }, RATIO);

        assert_eq!(minus_one, 0.5);
        assert_eq!(minus_two, 0.0);
    }

    #[test]
    fn every_bead_is_found_at_its_centre() {
        for grid in Grid::ALL {
            for row in -3..3 {
                for column in -3..3 {
                    let cell = Cell { column, row };
                    let (x, y) = grid.origin(cell, RATIO);
                    let found = grid.cell_at(x + RATIO / 2.0, y + 0.5, RATIO);
                    assert_eq!(found, cell, "{grid:?}");
                }
            }
        }
    }

    /// A bead whose corner is in the span is one of the cells in it.
    #[test]
    fn the_cells_in_a_span_hold_every_bead_there() {
        for grid in Grid::ALL {
            let found = grid.cells_in((-1.0, -1.0), (2.0, 2.0), RATIO);
            for cell in (Bounds {
                min: Cell {
                    column: -5,
                    row: -5,
                },
                max: Cell { column: 5, row: 5 },
            })
            .cells()
            {
                let (x, y) = grid.origin(cell, RATIO);
                let inside = x + RATIO > -1.0 && x < 2.0 && y + 1.0 > -1.0 && y < 2.0;
                assert!(!inside || found.contains(cell), "{grid:?} {cell:?}");
            }
        }
    }

    #[test]
    fn the_span_of_a_peyote_box_reaches_the_lower_columns() {
        let bounds = Bounds {
            min: Cell { column: 1, row: 0 },
            max: Cell { column: 3, row: 1 },
        };

        assert_eq!(Grid::Peyote.span(bounds, 1.0), ((1.0, 0.0), (4.0, 2.5)));
        assert_eq!(
            Grid::Peyote.span(Bounds::of(Cell { column: 1, row: 0 }), 1.0),
            ((1.0, 0.5), (2.0, 1.5))
        );
    }

    #[test]
    fn a_box_grows_to_hold_its_cells() {
        let bounds = Bounds::of(Cell { column: 2, row: 1 }).with(Cell { column: -1, row: 4 });

        assert_eq!((bounds.width(), bounds.height()), (4, 4));
        assert!(bounds.contains(Cell { column: 0, row: 2 }));
        assert_eq!(bounds.cells().count(), 16);
    }
}
