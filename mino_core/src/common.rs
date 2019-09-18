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

bitflags! {
    pub struct OverlayResult: u32 {
        const OK = 0b00000000;
        const OVERFLOW = 0b00000001;
        const OVERLAP = 0b00000010;
    }
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

    /// (x, y) is at the center of grid.
    pub fn check_overlay(&self, x: usize, y: usize, grid: &Grid<C>) -> OverlayResult {
        // TODO
        OverlayResult::OK
    }

    pub fn overlay(&self, x: usize, y: usize, grid: &Grid<C>) -> OverlayResult {
        // TODO
        OverlayResult::OK
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
pub enum Cell<Piece> {
    Empty,
    Block(Piece),
    Garbage,
}

impl<P> Default for Cell<P> {
    fn default() -> Self {
        Cell::Empty
    }
}

impl<P: fmt::Display> fmt::Display for Cell<P> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Empty => write!(formatter, " "),
            Cell::Block(p) => write!(formatter, "{}", p),
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

pub trait Piece<P> {
    fn grid(&self, rotation: Rotation) -> &Grid<Cell<P>>;
}

#[derive(Debug, Copy, Clone)]
pub struct FallingPiece<Piece> {
    pub piece: Piece,
    pub x: usize,
    pub y: usize,
    pub rotation: Rotation,
}

#[derive(Debug, Clone)]
pub struct Playfield<Piece> {
    pub visible_rows: usize,
    pub grid: Grid<Cell<Piece>>,
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
    /// `falling_piece_y = visible_rows + spawning_row_offset`
    pub spawning_row_offset: i32,
}

impl Default for GameParams {
    fn default() -> Self {
        // TODO
        GameParams {
            gravity: 0.167,
            soft_drop_gravity: 1.0,
            lock_delay: 60,
            das_delay: 11,
            das_period: 2,
            are: 40,
            line_clear_delay: 40,
            spawning_row_offset: -1,
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
pub struct GameState<Piece> {
    pub playfield: Playfield<Piece>,
    pub falling_piece: Option<FallingPiece<Piece>>,
    pub hold_pieces: Vec<Piece>,
    pub next_pieces: Vec<Piece>,
    pub counter: Counter,
}

pub trait GameLogic<P> {
    fn can_put(&self, p: FallingPiece<P>, dst: &Grid<Cell<P>>) -> bool;
    /// If overwritten, return true.
    fn put(&self, p: FallingPiece<P>, dst: &Grid<Cell<P>>) -> bool;
    fn rows_dropped_by_hard_drop(&self, field: &Grid<Cell<P>>, p: FallingPiece<P>)
        -> Option<usize>;
    fn generate_next(&self) -> Vec<P>;
}

pub fn update<P, Logic: GameLogic<P>>(
    logic: &Logic,
    params: &GameParams,
    state: &mut GameState<P>,
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
        let mut ps = logic.generate_next();
        state.next_pieces.append(&mut ps);
        // set falling piece.
        if let Some(p) = state.next_pieces.pop() {
            state.falling_piece = Some(FallingPiece {
                piece: p,
                x: state.playfield.grid.num_cols / 2,
                y: ((state.playfield.visible_rows as i32) + params.spawning_row_offset) as usize,
                rotation: Rotation::default(),
            });
        }
    }
}
