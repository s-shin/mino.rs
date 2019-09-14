use std::fmt;
use std::ops::Range;

/// Low level grid object.
/// Columns are numbered from left to right, and rows from bottom to top.
#[derive(Debug)]
pub struct Grid<C> {
    num_rows: usize,
    num_cols: usize,
    cells: Vec<C>,
}

impl<C> Grid<C>
where
    C: Default + Copy,
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

    pub fn get_cell_index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.num_cols);
        assert!(y < self.num_rows);
        x + y * self.num_cols
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: C) {
        let idx = self.get_cell_index(x, y);
        self.cells[idx] = cell;
    }

    pub fn get_cell(&self, x: usize, y: usize) -> C {
        self.cells[self.get_cell_index(x, y)]
    }
}

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
    C: Default + Copy + fmt::Display,
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
                if let Err(r) = self.grid.get_cell(x, y).fmt(formatter) {
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

//---

#[derive(Debug, Copy, Clone)]
pub enum Cell<T> {
    Empty,
    Block(T),
    Garbage,
}

impl<T> Default for Cell<T> {
    fn default() -> Self {
        Cell::Empty
    }
}

impl<T: fmt::Display> fmt::Display for Cell<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => write!(formatter, " "),
            Cell::Block(b) => write!(formatter, "{}", b),
            Cell::Garbage => write!(formatter, "x"),
        }
    }
}

//---

#[derive(Debug, Copy, Clone)]
pub enum Rotation {
    Cw0,
    Cw90,
    Cw180,
    Cw270,
}

impl Rotation {
    pub fn rotate_cw(&self, n: i8) -> Rotation {
        match ((*self as i16) + (n as i16)) % 4 {
            0 => Rotation::Cw0,
            1 => Rotation::Cw90,
            2 => Rotation::Cw180,
            3 => Rotation::Cw270,
            _ => panic!("never matched"),
        }
    }
    pub fn cw(&self) -> Rotation {
        self.rotate_cw(1)
    }
    pub fn ccw(&self) -> Rotation {
        self.rotate_cw(-1)
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::Cw0
    }
}

//---

pub struct FallingPiece<Block> {
    pub piece: Block,
    pub x: usize,
    pub y: usize,
    pub rotation: Rotation,
}

pub struct Playfield<B> {
    pub visible_rows: usize,
    pub grid: Grid<Cell<B>>,
}

pub type Gravity = f32;

// 60 fps
pub type Frames = u8;

pub struct GameParams {
    pub soft_drop: Gravity,
    pub lock_delay: Frames,
    pub das_delay: Frames,
    pub das_period: Frames,
    pub are: Frames,
    pub line_clear_delay: Frames,
}

// PPT: das 11f, 2f

#[derive(Debug, Copy, Clone)]
pub enum Input {
    HardDrop,
    SoftDrop,
    MoveLeft,
    MoveRight,
    RotateCw,
    RotateCcw,
    Hold,
}

pub struct InputFrameCounter {
    pub move_left: usize,
    pub move_right: usize,
}

pub struct GameState<Block> {
    pub playfield: Playfield<Block>,
    pub falling_piece: Option<FallingPiece<Block>>,
    pub hold_pieces: Vec<Block>,
    pub next_pieces: Vec<Block>,
    pub gravity: Gravity,
    pub input_frame_counter: InputFrameCounter,
}

impl<B> GameState<B> {
    pub fn update(&mut self, input: Input) {
        match input {
            //
        }
        // TODO
    }
}

// pub const NUM_INPUT_VARIANTS: usize = 6;

// pub struct InputFrameCounter {
//     counts: [usize; NUM_INPUT_VARIANTS],
// }

// impl InputFrameCounter {
//     pub fn new() -> InputFrameCounter {
//         InputFrameCounter {
//             counts: [0; NUM_INPUT_VARIANTS],
//         }
//     }

//     pub fn add(&mut self, input: Input, delta: usize) -> usize {
//         self.counts[input as usize] += delta;
//         self.get(input)
//     }

//     pub fn get(&self, input: Input) -> usize {
//         self.counts[input as usize]
//     }

//     pub fn set(&mut self, input: Input, n: usize) {
//         self.counts[input as usize] = n
//     }

//     pub fn reset(&mut self) {
//         for i in 0..NUM_INPUT_VARIANTS {
//             self.counts[i] = 0
//         }
//     }
// }
