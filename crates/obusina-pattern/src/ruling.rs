//! The ruling of a pattern: a line every so many columns and rows, to count
//! the beads by.

use crate::grid::Cell;

/// The columns and the rows between two lines of a new pattern.
pub const DEFAULT_RULING_STEP: [u32; 2] = [5, 5];

/// A line every `step` columns and rows, the first one `offset` cells from
/// cell (0, 0).
///
/// A step of 0 draws no line along that axis. On peyote a row of lines goes
/// up and down with the columns, as the beads of a row do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ruling {
    /// The columns, then the rows, from one line to the next.
    pub step: [u32; 2],
    /// The column, then the row, of a line, counted from cell (0, 0).
    pub offset: [i32; 2],
}

impl Default for Ruling {
    fn default() -> Self {
        Ruling {
            step: DEFAULT_RULING_STEP,
            offset: [0, 0],
        }
    }
}

impl Ruling {
    /// True when the column or the row of `cell` is a line.
    pub fn marks(&self, cell: Cell) -> bool {
        let on = |at: i32, step: u32, offset: i32| {
            step != 0 && (i64::from(at) - i64::from(offset)).rem_euclid(i64::from(step)) == 0
        };
        on(cell.column, self.step[0], self.offset[0]) || on(cell.row, self.step[1], self.offset[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_stands_every_step_from_the_offset() {
        let ruling = Ruling {
            step: [5, 4],
            offset: [2, -1],
        };
        let marks = |column, row| ruling.marks(Cell { column, row });

        assert!(marks(2, 0) && marks(7, 0) && marks(-3, 0), "columns");
        assert!(marks(0, -1) && marks(0, 3) && marks(0, -5), "rows");
        assert!(!marks(0, 0) && !marks(3, 1));
    }

    #[test]
    fn a_step_of_zero_draws_no_line_along_its_axis() {
        let ruling = Ruling {
            step: [0, 5],
            offset: [0, 0],
        };

        assert!(!ruling.marks(Cell { column: 0, row: 1 }));
        assert!(ruling.marks(Cell { column: 1, row: 5 }));
    }

    #[test]
    fn the_far_ends_of_the_field_do_not_overflow() {
        let ruling = Ruling {
            step: [u32::MAX, 3],
            offset: [i32::MAX, i32::MIN],
        };

        assert!(ruling.marks(Cell {
            column: i32::MAX,
            row: 0
        }));
    }
}
