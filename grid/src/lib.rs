//! Low level grid implementations.
//!
//! Columns are numbered from left to right, and rows from bottom to top.
//!
//! ```ignore
//!      ^
//!      |
//! (0,N)+-----------------------+
//!      |                       |
//!      |                       |
//!      |      Sub Grid         |
//!      |     +---------+       |
//!      |     |         |       |
//!      |     |         |       |
//!      |     |         |       |
//!      |     |         |       |
//!      |     |         |       |
//!      |     +---------+       |
//!      |   (x,y)               |
//!      |                       |
//!      |                       |
//!      |                       |
//!      +-----------------------+-->
//!    (0,0)                   (N,0)
//! ```

use std::fmt;
use std::ops::Range;
#[macro_use]
extern crate bitflags;

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
    C: Default + Clone,
{
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }

    pub fn num_cols(&self) -> usize {
        self.num_cols
    }

    pub fn is_valid_cell_index(&self, x: usize, y: usize) -> bool {
        x < self.num_cols && y < self.num_rows
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

    pub fn fill_row(&mut self, y: usize, cell: C) {
        for x in 0..self.num_cols {
            self.set_cell(x, y, cell.clone());
        }
    }

    pub fn fill_rows(&mut self, y_range: Range<usize>, cell: C) {
        for y in y_range {
            self.fill_row(y, cell.clone());
        }
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

    pub fn rotate1(&self) -> Grid<C> {
        let mut g = Grid::new(self.num_rows, self.num_cols, vec![]);
        for y in 0..self.num_rows {
            for x in 0..self.num_cols {
                g.set_cell(y, self.num_cols - 1 - x, self.cell(x, y))
            }
        }
        g
    }
    pub fn rotate2(&self) -> Grid<C> {
        let mut g = Grid::new(self.num_cols, self.num_rows, vec![]);
        for y in 0..self.num_rows {
            for x in 0..self.num_cols {
                g.set_cell(
                    self.num_cols - 1 - x,
                    self.num_rows - 1 - y,
                    self.cell(x, y),
                )
            }
        }
        g
    }
    pub fn rotate3(&self) -> Grid<C> {
        let mut g = Grid::new(self.num_rows, self.num_cols, vec![]);
        for y in 0..self.num_rows {
            for x in 0..self.num_cols {
                g.set_cell(self.num_rows - 1 - y, x, self.cell(x, y))
            }
        }
        g
    }

    pub fn move_row(&mut self, src_y: usize, dst_y: usize, placeholder: Option<C>) {
        for x in 0..self.num_cols {
            self.set_cell(x, dst_y, self.cell(x, src_y));
            if let Some(cell) = placeholder.as_ref() {
                self.set_cell(x, src_y, cell.clone());
            }
        }
    }

    pub fn map(&mut self, cb: fn(C) -> C) {
        for y in 0..self.num_rows {
            for x in 0..self.num_cols {
                self.set_cell(x, y, cb(self.cell(x, y)));
            }
        }
    }
}

impl<C> PartialEq for Grid<C>
where
    C: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.num_cols == other.num_cols
            && self.num_rows == other.num_rows
            && self.cells == other.cells
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

impl<C> Grid<C>
where
    C: Default + Clone + IsEmpty,
{
    pub fn is_row_filled(&self, y: usize) -> bool {
        for x in 0..self.num_cols {
            if self.cell(x, y).is_empty() {
                return false;
            }
        }
        true
    }

    pub fn num_filled_rows(&self) -> usize {
        let mut n = 0;
        for y in 0..self.num_rows {
            if self.is_row_filled(y) {
                n += 1;
            }
        }
        n
    }

    pub fn pluck_filled_rows(&mut self, placeholder: Option<C>) -> usize {
        let mut n = 0;
        for y in 0..self.num_rows {
            if self.is_row_filled(y) {
                n += 1;
                continue;
            }
            if n > 0 {
                self.move_row(y, y - n, None);
            }
            if y == self.num_rows - n {
                break;
            }
        }
        if let Some(cell) = placeholder.as_ref() {
            self.fill_rows((self.num_rows - n)..self.num_rows, cell.clone());
        }
        n
    }

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

    /// Return (n, result) if overlap(x + dx * n, y + dy * n, sub) is not empty.
    pub fn check_overlay_toward(
        &self,
        x: i32,
        y: i32,
        sub: &Grid<C>,
        dx: i32,
        dy: i32,
    ) -> (usize, OverlayResult) {
        assert!(dx != 0 || dy != 0);
        let mut r: OverlayResult;
        let mut n: usize = 0;
        let mut tx = x;
        let mut ty = y;
        loop {
            r = self.check_overlay(tx, ty, &sub);
            if !r.is_empty() {
                break;
            }
            tx += dx;
            ty += dy;
            n += 1;
        }
        (n, r)
    }

    pub fn bottom_padding(&self) -> usize {
        for n in 0..self.num_rows {
            let y = n;
            for x in 0..self.num_cols {
                if !self.cell(x, y).is_empty() {
                    return n;
                }
            }
        }
        self.num_rows()
    }

    pub fn top_padding(&self) -> usize {
        for n in 0..self.num_rows {
            let y = self.num_rows - n - 1;
            for x in 0..self.num_cols {
                if !self.cell(x, y).is_empty() {
                    return n;
                }
            }
        }
        self.num_rows()
    }
}

//---

pub struct GridFormatOptions {
    pub str_begin_of_line: &'static str,
    pub str_end_of_line: &'static str,
    pub range_x: Option<Range<usize>>,
    pub range_y: Option<Range<usize>>,
}

impl Default for GridFormatOptions {
    fn default() -> Self {
        Self {
            str_begin_of_line: "",
            str_end_of_line: "",
            range_x: Option::None,
            range_y: Option::None,
        }
    }
}

pub struct GridFormatter<'a, C> {
    pub grid: &'a Grid<C>,
    pub opts: GridFormatOptions,
}

impl<'a, C> fmt::Display for GridFormatter<'a, C>
where
    C: Default + Clone + fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let range_x = match self.opts.range_x.clone() {
            None => 0..self.grid.num_cols,
            Some(x) => x,
        };
        let range_y = match self.opts.range_y.clone() {
            None => 0..self.grid.num_rows,
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

        assert_eq!(
            (0, OverlayResult::OVERLAP),
            grid.check_overlay_toward(0, 0, &sub, 1, 0)
        );
        assert_eq!(
            (2, OverlayResult::OVERFLOW),
            grid.check_overlay_toward(1, 0, &sub, 1, 0)
        );
        assert_eq!(
            (1, OverlayResult::OVERLAP),
            grid.check_overlay_toward(1, 0, &sub, -1, 0)
        );

        let r = grid.overlay(0, 1, &sub);
        assert!(r.is_empty());
        assert_eq!(1, grid.cell(0, 1));
        assert_eq!(1, grid.cell(1, 2));
    }

    #[test]
    fn padding_test() {
        let mut grid = MyGrid::new(
            2,
            5,
            vec![
                0, 0, //
                1, 0, //
                0, 1, //
                0, 0, //
                0, 0, //
            ],
        );
        grid.reverse_rows();
        assert_eq!(1, grid.top_padding());
        assert_eq!(2, grid.bottom_padding());

        let grid = MyGrid::new(1, 2, vec![]);
        assert_eq!(2, grid.top_padding());
        assert_eq!(2, grid.bottom_padding());
    }

    #[test]
    fn eq_test() {
        let grid = MyGrid::new(1, 2, vec![1, 2]);
        assert_eq!(grid, grid.clone());
    }

    #[test]
    fn rotate_test() {
        let mut grid = MyGrid::new(
            3,
            2,
            vec![
                1, 2, 3, //
                4, 5, 6, //
            ],
        );
        grid.reverse_rows();

        let mut expected1 = MyGrid::new(
            2,
            3,
            vec![
                4, 1, //
                5, 2, //
                6, 3, //
            ],
        );
        expected1.reverse_rows();

        let mut expected2 = MyGrid::new(
            3,
            2,
            vec![
                6, 5, 4, //
                3, 2, 1, //
            ],
        );
        expected2.reverse_rows();

        let mut expected3 = MyGrid::new(
            2,
            3,
            vec![
                3, 6, //
                2, 5, //
                1, 4, //
            ],
        );
        expected3.reverse_rows();

        assert_eq!(expected1, grid.rotate1());
        assert_eq!(expected2, grid.rotate2());
        assert_eq!(expected3, grid.rotate3());
    }

    #[test]
    fn formatter_test() {
        let mut grid = MyGrid::new(2, 3, vec![1, 2, 3, 4, 5, 6]);
        grid.reverse_rows();
        assert_eq!(
            "12\n34\n56\n",
            format!(
                "{}",
                GridFormatter::<MyCell> {
                    grid: &grid,
                    opts: Default::default(),
                },
            ),
        );
        assert_eq!(
            "B3E\n",
            format!(
                "{}",
                GridFormatter::<MyCell> {
                    grid: &grid,
                    opts: GridFormatOptions {
                        str_begin_of_line: "B",
                        str_end_of_line: "E",
                        range_x: Some(0..1),
                        range_y: Some(1..2),
                    }
                },
            ),
        );
    }
}
