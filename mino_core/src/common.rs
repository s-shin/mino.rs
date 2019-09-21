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
    pub x: i32,
    pub y: i32,
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

/// http://harddrop.com/wiki/Lock_delay
#[derive(Debug, Copy, Clone)]
pub enum LockDelayReset {
    EntryReset,
    StepReset,
    MoveReset,
}

impl Default for LockDelayReset {
    fn default() -> Self {
        LockDelayReset::StepReset
    }
}

bitflags! {
    /// http://harddrop.com/wiki/Top_out
    pub struct LossCondition: u32 {
        const BLOCK_OUT = 0b00000001;
        const LOCK_OUT = 0b00000010;
        const PARTIAL_LOCK_OUT = 0b00000100;
        const GARBAGE_OUT = 0b00001000;
    }
}

impl Default for LossCondition {
    fn default() -> Self {
        LossCondition::BLOCK_OUT | LossCondition::LOCK_OUT | LossCondition::GARBAGE_OUT
    }
}

pub struct GameParams {
    pub gravity: Gravity,
    pub soft_drop_gravity: Gravity,
    pub lock_delay: Frames,
    pub lock_dekay_reset: LockDelayReset,
    pub lock_delay_cancel: bool,
    pub das_delay: Frames,
    pub das_period: Frames,
    pub are: Frames,
    pub line_clear_delay: Frames,
    pub loss_condition: LossCondition,
    pub num_hold_pieces: usize,
}

impl Default for GameParams {
    fn default() -> Self {
        // TODO
        GameParams {
            gravity: 0.1667, // 1/60
            soft_drop_gravity: 1.0,
            lock_delay: 60,
            lock_dekay_reset: LockDelayReset::default(),
            lock_delay_cancel: true,
            das_delay: 11,
            das_period: 2,
            are: 40,
            line_clear_delay: 40,
            loss_condition: LossCondition::default(),
            num_hold_pieces: 1,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Input {
    HardDrop,
    SoftDrop,
    // Useful for automation.
    FirmDrop,
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
    pub lock: Frames,
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
    fn piece_grid(&self, piece: P, rotation: Rotation) -> &grid::Grid<Cell<P>>;
    fn generate_next_pieces(&self) -> Vec<P>;
    fn spawn_piece(&self, piece: P, playfield: &Playfield<P>) -> FallingPiece<P>;
}

pub fn update<P: Piece, Logic: GameLogic<P>>(
    logic: &Logic,
    params: &GameParams,
    state: &mut GameState<P>,
    input: Input,
) {
    if let Some(falling_piece) = state.falling_piece.as_mut() {
        let piece_grid = logic.piece_grid(falling_piece.piece, falling_piece.rotation);

        let num_droppable_rows = {
            let (n, _r) = state.playfield.grid.check_overlay_toward(
                falling_piece.x as i32,
                falling_piece.y as i32,
                piece_grid,
                0,
                -1,
            );
            assert_ne!(0, n);
            n - 1
        } as i32;

        match input {
            Input::HardDrop => {
                // Lock falling piece.
                let r = state.playfield.grid.overlay(
                    falling_piece.x as i32,
                    falling_piece.y as i32 - num_droppable_rows,
                    piece_grid,
                );
                assert!(r.is_empty());
                state.falling_piece = None;
                state.counter.gravity = 0.0;
                state.counter.lock = 0;
            }
            Input::SoftDrop => {
                let g = state.counter.gravity + params.gravity + params.soft_drop_gravity;
                let num_rows_to_be_dropped = g as i32;
                if params.lock_delay_cancel && num_rows_to_be_dropped == 0 {
                    // Lock falling piece.
                    let r = state.playfield.grid.overlay(
                        falling_piece.x as i32,
                        falling_piece.y as i32 - num_droppable_rows,
                        piece_grid,
                    );
                    assert!(r.is_empty());
                    state.falling_piece = None;
                    state.counter.gravity = 0.0;
                    state.counter.lock = 0;
                } else {
                    state.counter.gravity -= num_rows_to_be_dropped as f32;
                    if num_rows_to_be_dropped > 0 {
                        falling_piece.y -= if num_droppable_rows < num_rows_to_be_dropped {
                            num_droppable_rows
                        } else {
                            num_rows_to_be_dropped
                        };
                    }
                    match params.lock_dekay_reset {
                        LockDelayReset::EntryReset => {}
                        _ => state.counter.lock = 0,
                    }
                }
            }
            Input::FirmDrop => {
                // TODO
            }
            Input::MoveLeft | Input::MoveRight => {
                // TODO
            }
            Input::RotateCw | Input::RotateCcw => {
                // TODO
            }
            Input::Hold => {
                // TODO
                state.falling_piece = None;
                state.counter.gravity = 0.0;
                state.counter.lock = 0;
            }
        }
    } else {
        // Wait for ARE.
        if state.counter.are <= params.are {
            state.counter.are += 1;
            return;
        }
        // ARE elapsed.
        state.counter.are = 0;
        // Generate next pieces.
        let mut ps = logic.generate_next_pieces();
        state.next_pieces.append(&mut ps);
        // Set falling piece.
        if let Some(p) = state.next_pieces.pop() {
            state.falling_piece = Some(logic.spawn_piece(p, &state.playfield));
            // TODO: check overlay
        }
    }
}
