use std::fmt;

pub mod grid;

// TODO: replace to trait alias in the future.
// https://github.com/rust-lang/rfcs/blob/master/text/1733-trait-alias.md
pub trait Piece: Copy {}

#[derive(Debug, Copy, Clone)]
pub enum Cell<P: Piece> {
    Empty,
    Block(P),
    Garbage,
}

impl<P: Piece> grid::IsEmpty for Cell<P> {
    fn is_empty(&self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false,
        }
    }
}

impl<P: Piece> Default for Cell<P> {
    fn default() -> Self {
        Cell::Empty
    }
}

impl<P: Piece + fmt::Display> fmt::Display for Cell<P> {
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

#[derive(Debug, Copy, Clone)]
pub struct FallingPiece<P: Piece> {
    pub piece: P,
    pub x: usize,
    pub y: usize,
    pub rotation: Rotation,
}

#[derive(Debug, Clone)]
pub struct Playfield<P: Piece> {
    pub visible_rows: usize,
    pub grid: grid::Grid<Cell<P>>,
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
pub struct GameState<P: Piece> {
    pub playfield: Playfield<P>,
    pub falling_piece: Option<FallingPiece<P>>,
    pub hold_pieces: Vec<P>,
    pub next_pieces: Vec<P>,
    pub counter: Counter,
}

pub trait GameLogic<P: Piece> {
    // fn can_put(&self, p: FallingPiece<P>, dst: &Grid<Cell<P>>) -> bool;
    // /// If overwritten, return true.
    // fn put(&self, p: FallingPiece<P>, dst: &Grid<Cell<P>>) -> bool;
    // fn rows_dropped_by_hard_drop(&self, field: &Grid<Cell<P>>, p: FallingPiece<P>)
    //     -> Option<usize>;
    fn piece_grid(&self, piece: P, rotation: Rotation) -> &grid::Grid<Cell<P>>;
    fn generate_next(&self) -> Vec<P>;
}

pub fn update<P: Piece, Logic: GameLogic<P>>(
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
                x: state.playfield.grid.num_cols() / 2,
                y: ((state.playfield.visible_rows as i32) + params.spawning_row_offset) as usize,
                rotation: Rotation::default(),
            });
        }
    }
}
