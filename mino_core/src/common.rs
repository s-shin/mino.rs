use std::fmt;
use std::ops::Range;

/// Low level grid object.
/// Columns are numbered from left to right, and rows from bottom to top.
#[derive(Debug, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct FallingPiece<Block> {
    pub piece: Block,
    pub x: usize,
    pub y: usize,
    pub rotation: Rotation,
}

#[derive(Debug, Clone)]
pub struct Playfield<B> {
    pub visible_rows: usize,
    pub grid: Grid<Cell<B>>,
}

/// G = cells / frame
pub type Gravity = f32;

/// 60 fps
pub type Frames = u8;

pub struct GameParams {
    pub gravity: Gravity,
    pub soft_drop_gravity: Gravity,
    pub lock_delay: Frames,
    pub das_delay: Frames,
    pub das_period: Frames,
    pub are: Frames,
    pub line_clear_delay: Frames,
}

impl Default for GameParams {
    fn default() -> Self {
        // TODO
        GameParams {
            gravity: 0.167,
            soft_drop_gravity: 1,
            lock_delay: 60,
            das_delay: 11,
            das_period: 2,
            are: 40,
            line_clear_delay: 40,
        }
    }
}

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

#[derive(Debug, Copy, Clone)]
pub struct Counter {
    pub move_left: Frames,
    pub move_right: Frames,
    pub gravity: Gravity,
    pub are: Frames,
}

impl Counter {
    pub fn rows_to_be_dropped(&self) -> usize {
        self.gravity as usize
    }
}

#[derive(Debug, Clone)]
pub struct GameState<Block> {
    pub playfield: Playfield<Block>,
    pub falling_piece: Option<FallingPiece<Block>>,
    pub hold_pieces: Vec<Block>,
    pub next_pieces: Vec<Block>,
    pub counter: Counter,
}

pub trait GameLogic<B> {
    fn can_put(&self, p: FallingPiece<B>, dst: &Grid<Cell<B>>) -> bool;
    /// If overwritten, return true.
    fn put(&self, p: FallingPiece<B>, dst: &Grid<Cell<B>>) -> bool;
    fn rows_dropped_by_hard_drop(&self, field: &Grid<Cell<B>>, p: FallingPiece<B>)
        -> Option<usize>;
    fn generate_next(&self) -> Vec<B>;
}

pub fn update<B, Logic: GameLogic<B>>(
    logic: &Logic,
    params: &GameParams,
    state: &mut GameState<B>,
    input: Input,
) {
    if let Some(falling_piece) = &state.falling_piece {
        match input {
            Input::HardDrop => {
                // TODO
            }
            _ => {}
        }
    } else {
        // wait for ARE.
        if state.counter.are <= params.are {
            state.counter.are += 1;
            return;
        }
        // generate next pieces.
        state.counter.are = 0;
        let mut bs = logic.generate_next();
        state.next_pieces.append(&mut bs);
        // set falling piece.
        if let Some(b) = state.next_pieces.pop() {
            state.falling_piece = Some(FallingPiece {
                piece: b,
                x: 0,
                y: 0,
                rotation: Rotation::default(),
            });
        }
    }
}
