//! Low level grid implementations.
//!
//! Columns are numbered from left to right, and rows from bottom to top.
//!
//! # Overview
//!
//! ```ignore
//!      ^
//!      |           *non-empty cell
//! (0,N)+-----------------------+
//!      |                       |
//!      |                       |
//!      |      Sub Grid         |
//!      |     +---------+       |
//!      |     |         |       |
//!      |     |         |       |
//!      |     |       *<---->*  |
//!      |     |       * |       |
//!      |     |       ^ |       |
//!      |     +-------|-+       |
//!      |   (x,y)     |         |
//!      |         neighbor      |
//!      |             |         |
//!      |             v         |
//!      +-------------------------->
//!    (0,0)                   (N,0)
//! ```

use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Grid<C> {
    num_rows: usize,
    num_cols: usize,
    cells: Vec<C>,
}

impl<C> Grid<C>
where
    C: Default + Clone,
{
    pub fn new(cols: usize, rows: usize, mut cells: Vec<C>) -> Grid<C> {
        let num = cols * rows;
        cells.resize(num, C::default());
        Grid {
            num_cols: cols,
            num_rows: rows,
            cells: cells,
        }
    }
}

impl<C> Grid<C>
where
    C: Clone,
{
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    pub fn cell_index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.num_cols);
        assert!(y < self.num_rows);
        x + y * self.num_cols
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: C) {
        let idx = self.cell_index(x, y);
        self.cells[idx] = cell;
    }

    pub fn cell(&self, x: usize, y: usize) -> C {
        self.cells[self.cell_index(x, y)].clone()
    }

    /// Swap (x, y) for (x, num_rows - 1 - y).
    pub fn reverse_rows(&mut self) -> &mut Self {
        let n = self.num_rows / 2;
        for y in 0..n {
            let yy = self.num_rows - 1 - y;
            for x in 0..self.num_cols {
                let t = self.cell(x, y);
                self.set_cell(x, y, self.cell(x, yy));
                self.set_cell(x, yy, t);
            }
        }
        self
    }
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

bitflags! {
    #[derive(Default)]
    pub struct OverlayResult: u32 {
        const OVERFLOW = 0b00000001;
        const OVERLAP = 0b00000010;
    }
}

pub enum Side {
    Left,
    Right,
    Bottom,
    Top,
}

impl<C> Grid<C>
where
    C: Clone + IsEmpty,
{
    pub fn check_overlay(&self, x: i32, y: i32, sub: &Grid<C>) -> OverlayResult {
        let mut result = OverlayResult::empty();
        for sub_y in 0..sub.num_rows {
            for sub_x in 0..sub.num_cols {
                let sub_cell = sub.cell(sub_x, sub_y);
                if sub_cell.is_empty() {
                    continue;
                }
                let self_x = x + sub_x as i32;
                let self_y = y + sub_y as i32;
                if self_x < 0
                    || self.num_cols as i32 <= self_x
                    || self_y < 0
                    || self.num_rows as i32 <= self_y
                {
                    result |= OverlayResult::OVERFLOW;
                    continue;
                }
                let self_cell = self.cell(self_x as usize, self_y as usize);
                if !self_cell.is_empty() {
                    result |= OverlayResult::OVERLAP;
                }
            }
        }
        result
    }

    pub fn overlay(&mut self, x: i32, y: i32, sub: &Grid<C>) -> OverlayResult {
        let mut result = OverlayResult::empty();
        for sub_y in 0..sub.num_rows {
            for sub_x in 0..sub.num_cols {
                let sub_cell = sub.cell(sub_x, sub_y);
                if sub_cell.is_empty() {
                    continue;
                }
                let self_x = x + sub_x as i32;
                let self_y = y + sub_y as i32;
                if self_x < 0
                    || self.num_cols as i32 <= self_x
                    || self_y < 0
                    || self.num_rows as i32 <= self_y
                {
                    result |= OverlayResult::OVERFLOW;
                    continue;
                }
                let self_cell = self.cell(self_x as usize, self_y as usize);
                if !self_cell.is_empty() {
                    result |= OverlayResult::OVERLAP;
                } else {
                    // NOTE: completely same code as check_overlay() except here
                    self.set_cell(self_x as usize, self_y as usize, sub_cell);
                }
            }
        }
        result
    }

    /// Return offset of neighbor non-empty cell or edge toward the side.
    pub fn neighbor(&self, x: i32, y: i32, sub: &Grid<C>, side: Side) -> usize {
        // TODO
        1
    }
}

//---

pub struct GridFormatterOptions {
    pub str_begin_of_line: &'static str,
    pub str_end_of_line: &'static str,
    pub range_x: Option<Range<usize>>,
    pub range_y: Option<Range<usize>>,
}

impl Default for GridFormatterOptions {
    fn default() -> Self {
        Self {
            str_begin_of_line: "",
            str_end_of_line: "",
            range_x: Option::None,
            range_y: Option::None,
        }
    }
}

pub struct GridFormatter<C> {
    pub grid: Grid<C>,
    pub opts: GridFormatterOptions,
}

impl<C> fmt::Display for GridFormatter<C>
where
    C: Default + Clone + fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let range_x = match self.opts.range_x.clone() {
            None => 0..(self.grid.num_cols - 1),
            Some(x) => x,
        };
        let range_y = match self.opts.range_y.clone() {
            None => 0..(self.grid.num_rows - 1),
            Some(y) => y,
        };
        // write cells from top to bottom.
        for y in range_y.rev() {
            if let Err(r) = formatter.write_str(self.opts.str_begin_of_line) {
                return Err(r);
            }
            for x in range_x.clone() {
                if let Err(r) = self.grid.cell(x, y).fmt(formatter) {
                    return Err(r);
                }
            }
            if let Err(r) = formatter.write_str(self.opts.str_end_of_line) {
                return Err(r);
            }
            if let Err(r) = formatter.write_str("\n") {
                return Err(r);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MyCell = u8;

    impl IsEmpty for MyCell {
        fn is_empty(&self) -> bool {
            *self == 0
        }
    }

    type MyGrid = Grid<MyCell>;

    #[test]
    fn basic_test() {
        let mut grid = MyGrid::new(4, 8, vec![]);
        assert_eq!(4, grid.num_cols());
        assert_eq!(8, grid.num_rows());
        grid.set_cell(1, 2, 1);
        assert_eq!(0, grid.cell(0, 0));
        assert_eq!(1, grid.cell(1, 2));
        grid.reverse_rows();
        assert_eq!(0, grid.cell(1, 2));
        assert_eq!(1, grid.cell(1, 5));
    }

    #[test]
    fn overlay_test() {
        let mut grid = MyGrid::new(
            4,
            4,
            vec![
                0, 0, 0, 1, //
                0, 0, 0, 0, //
                0, 1, 0, 0, //
                0, 0, 0, 0, //
            ],
        );
        grid.reverse_rows();
        let mut sub = MyGrid::new(
            2,
            2,
            vec![
                0, 1, //
                1, 0, //
            ],
        );
        sub.reverse_rows();

        let r = grid.check_overlay(0, 0, &sub);
        assert_eq!(OverlayResult::OVERLAP, r);

        let r = grid.check_overlay(0, 1, &sub);
        assert!(r.is_empty());

        let r = grid.check_overlay(2, 3, &sub);
        assert_eq!(OverlayResult::OVERFLOW, r);

        let r = grid.check_overlay(3, 3, &sub);
        assert!(r.contains(OverlayResult::OVERFLOW));
        assert!(r.contains(OverlayResult::OVERLAP));

        let r = grid.overlay(0, 1, &sub);
        assert!(r.is_empty());
        assert_eq!(1, grid.cell(0, 1));
        assert_eq!(1, grid.cell(1, 2));
    }

    #[test]
    fn neighbor_test() {
        // TODO
    }
}
