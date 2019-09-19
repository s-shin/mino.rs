//! Low level grid implementations.
//!
//! Columns are numbered from left to right, and rows from bottom to top.
//!
//! # Overview
//!
//! ```no_run
//!      ^
//!      |           *non-empty cell
//! (0,N)+-----------------------+
//!      |                       |
//!      |                       |
//!      |      Sub Grid         |
//!      |     +---------+       |
//!      |     |         |       |
//!      |     |  (x,y)  |       |
//!      |     |    +  *<---->*  |
//!      |     |       * |       |
//!      |     |       ^ |       |
//!      |     +-------|-+       |
//!      |             |         |
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
}

pub trait IsEmpty {
    fn is_empty(&self) -> bool;
}

bitflags! {
    pub struct OverlayResult: u32 {
        const OK = 0b00000000;
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
    C: IsEmpty,
{
    pub fn check_overlay(&self, x: usize, y: usize, sub: &Grid<C>) -> OverlayResult {
        // TODO
        OverlayResult::OK
    }

    pub fn overlay(&mut self, x: usize, y: usize, sub: &Grid<C>) -> OverlayResult {
        // TODO
        OverlayResult::OK
    }

    /// Return offset of neighbor non-empty cell or edge toward the side.
    pub fn neighbor(&self, x: usize, y: usize, sub: &Grid<C>, side: Side) -> usize {
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
