use std::collections::VecDeque;
use std::fmt;
use std::hash::Hash;

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

pub trait Piece: Copy {
    fn grid(&self, rotation: Rotation) -> PieceGrid<Self>;
    fn grid_top_padding(&self, rotation: Rotation) -> usize {
        let (n, ok) = self.grid(rotation).top_padding();
        assert!(ok);
        n
    }
    fn grid_bottom_padding(&self, rotation: Rotation) -> usize {
        let (n, ok) = self.grid(rotation).bottom_padding();
        assert!(ok);
        n
    }
}

type PieceGrid<P> = grid::Grid<Cell<P>>;

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

#[derive(Debug, Copy, Clone)]
pub struct FallingPiece<P: Piece> {
    pub piece: P,
    pub x: i32,
    pub y: i32,
    pub rotation: Rotation,
}

impl<P: Piece> FallingPiece<P> {
    fn grid(&self) -> PieceGrid<P> {
        self.piece.grid(self.rotation)
    }
    fn grid_top_padding(&self) -> usize {
        self.piece.grid_top_padding(self.rotation)
    }
    fn grid_bottom_padding(&self) -> usize {
        self.piece.grid_bottom_padding(self.rotation)
    }
    fn is_lock_out(&self, playfield: &Playfield<P>) -> bool {
        let padding = self.grid_bottom_padding();
        self.y + padding as i32 >= playfield.visible_rows as i32
    }
    fn is_partial_lock_out(&self, playfield: &Playfield<P>) -> bool {
        let padding = self.grid_top_padding();
        self.y + (self.grid().num_rows() - padding) as i32 >= playfield.visible_rows as i32
    }
    fn can_put_onto(&self, playfield: &Playfield<P>) -> bool {
        playfield
            .grid
            .check_overlay(self.x, self.y, &self.grid())
            .is_empty()
    }
    fn put_onto(&self, playfield: &mut Playfield<P>) -> grid::OverlayResult {
        playfield.grid.overlay(self.x, self.y, &self.grid())
    }
    fn droppable_rows(&self, playfield: &Playfield<P>) -> usize {
        let (n, _r) =
            playfield
                .grid
                .check_overlay_toward(self.x as i32, self.y as i32, &self.grid(), 0, -1);
        n
    }
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
        const LOCK_OUT = 0b00000001;
        const PARTIAL_LOCK_OUT = 0b00000010;
        const GARBAGE_OUT = 0b00000100;
    }
}

impl LossCondition {
    fn check<P: Piece>(
        self,
        falling_piece: &FallingPiece<P>,
        playfield: &Playfield<P>,
    ) -> LossCondition {
        if self.contains(LossCondition::LOCK_OUT) {
            if falling_piece.is_lock_out(playfield) {
                return self;
            }
        }
        if self.contains(LossCondition::PARTIAL_LOCK_OUT) {
            if falling_piece.is_partial_lock_out(playfield) {
                return self;
            }
        }
        return Self::empty();
    }
}

impl Default for LossCondition {
    fn default() -> Self {
        LossCondition::LOCK_OUT | LossCondition::GARBAGE_OUT
    }
}

impl fmt::Display for LossCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct GameParams {
    pub gravity: Gravity,
    pub soft_drop_gravity: Gravity,
    pub lock_delay: Frames,
    pub lock_delay_reset: LockDelayReset,
    /// https://harddrop.com/wiki/Lock_delay
    pub lock_delay_cancel: bool,
    // Delayed Auto Shift: https://harddrop.com/wiki/DAS
    pub das: Frames,
    // Auto Repeat Rate: https://harddrop.com/wiki/DAS
    pub arr: Frames,
    // https://harddrop.com/wiki/ARE
    pub are: Frames,
    pub line_clear_delay: Frames,
    pub loss_condition: LossCondition,
}

impl Default for GameParams {
    fn default() -> Self {
        // TODO
        GameParams {
            gravity: 0.1667, // 1/60
            soft_drop_gravity: 1.0,
            lock_delay: 60,
            lock_delay_reset: LockDelayReset::default(),
            lock_delay_cancel: true,
            das: 11,
            arr: 2,
            are: 40,
            line_clear_delay: 40,
            loss_condition: LossCondition::default(),
        }
    }
}

pub trait GameLogic<P: Piece> {
    fn spawn_piece(&self, piece: Option<P>, playfield: &Playfield<P>) -> FallingPiece<P>;
    fn rotate(
        &self,
        cw: bool,
        falling_piece: &FallingPiece<P>,
        playfield: &Playfield<P>,
    ) -> Option<FallingPiece<P>>;
}

bitflags! {
    #[derive(Default)]
    pub struct Input: u32 {
        /// Generally, up in DPAD.
        const HARD_DROP = 0b00000001;
        /// Generally, down in DPAD.
        const SOFT_DROP = 0b00000010;
        /// Rarely supported. Useful for automation.
        const FIRM_DROP = 0b00000100;
        /// Generally, left in DPAD.
        const MOVE_LEFT = 0b00001000;
        /// Generally, right in DPAD.
        const MOVE_RIGHT = 0b00010000;
        /// Generally, A/circle button.
        const ROTATE_CW = 0b00100000;
        /// Generally, B/cross button.
        const ROTATE_CCW = 0b01000000;
        /// Generally, L/R button.
        const HOLD = 0b10000000;
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct InputCounter {
    pub move_left: Frames,
    pub move_right: Frames,
    pub rotate_cw: Frames,
    pub rotate_ccw: Frames,
    pub hold: Frames,
}

impl InputCounter {
    fn update(&mut self, params: &GameParams, input: Input) {
        self.move_left = if input.contains(Input::MOVE_LEFT) {
            if self.move_right > 0 {
                self.move_right = 0;
            }
            // prevent overflow
            if self.move_left == params.das + params.arr {
                params.das + 1
            } else {
                self.move_left + 1
            }
        } else {
            0
        };
        self.move_right = if input.contains(Input::MOVE_RIGHT) {
            if self.move_left > 0 {
                self.move_left = 0;
            }
            if self.move_right == params.das + params.arr {
                params.das + 1
            } else {
                self.move_right + 1
            }
        } else {
            0
        };
        self.rotate_cw = if input.contains(Input::ROTATE_CW) {
            self.rotate_ccw = 0;
            std::cmp::min(self.rotate_cw + 1, 2)
        } else {
            0
        };
        self.rotate_ccw = if input.contains(Input::ROTATE_CCW) {
            self.rotate_cw = 0;
            std::cmp::min(self.rotate_ccw + 1, 2)
        } else {
            0
        };
        self.hold = if input.contains(Input::HOLD) {
            self.hold + 1
        } else {
            0
        }
    }
    fn should_move_left(&self, params: &GameParams) -> bool {
        self.move_left == 1
            || self.move_left == params.das
            || self.move_left == params.das + params.arr
    }
    fn should_move_right(&self, params: &GameParams) -> bool {
        self.move_right == 1
            || self.move_right == params.das
            || self.move_right == params.das + params.arr
    }
    fn should_rotate_cw(&self) -> bool {
        self.rotate_cw == 1
    }
    fn should_rotate_ccw(&self) -> bool {
        self.rotate_ccw == 1
    }
    fn should_hold(&self) -> bool {
        self.hold == 1
    }
}

pub struct GameConfig<Logic> {
    pub logic: Logic,
    pub params: GameParams,
}

#[derive(Debug, Clone)]
pub struct GameStateData<P: Piece> {
    pub playfield: Playfield<P>,
    pub falling_piece: Option<FallingPiece<P>>,
    pub hold_piece: Option<P>,
    pub next_pieces: VecDeque<P>,
}

#[derive(Debug, Copy, Clone)]
pub enum GameStateId {
    Init,
    Play,
    LineClear,
    SpawnPiece,
    GameOver,
    Error,
}

pub trait GameState<P: Piece, L> {
    fn id(&self) -> GameStateId;
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "")
    }
    fn enter(
        &mut self,
        _data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        Ok(None)
    }
    fn update(
        &mut self,
        _data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        Ok(None)
    }
    fn exit(&mut self, _data: &mut GameStateData<P>, _config: &GameConfig<L>) {}
}

impl<P: Piece, L> fmt::Display for dyn GameState<P, L> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(formatter)
    }
}

impl<P: Piece, L> From<Box<dyn GameState<P, L>>> for GameStateId {
    fn from(s: Box<dyn GameState<P, L>>) -> GameStateId {
        s.id()
    }
}

struct GameStateInit;

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateInit {
    fn id(&self) -> GameStateId {
        GameStateId::Init
    }
    fn update(
        &mut self,
        data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if data.falling_piece.is_some() {
            Ok(Some(Box::new(GameStatePlay::default())))
        } else {
            Ok(Some(Box::new(GameStateSpawnPiece::default())))
        }
    }
}

struct GameStateError {
    reason: String,
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateError {
    fn id(&self) -> GameStateId {
        GameStateId::Error
    }
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.reason)
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct GameStatePlay {
    input_counter: InputCounter,
    gravity_counter: Gravity,
    is_piece_held: bool,
}

impl GameStatePlay {
    fn reset_counter(&mut self) {
        self.input_counter = Default::default();
        self.gravity_counter = Default::default();
    }
    fn update_counter(&mut self, params: &GameParams, input: Input) {
        self.input_counter.update(params, input);
        self.gravity_counter += params.gravity;
    }
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStatePlay {
    fn id(&self) -> GameStateId {
        GameStateId::Play
    }
    fn enter(
        &mut self,
        data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if data.falling_piece.is_none() {
            return Err("falling_piece should not be none".into());
        }
        Ok(None)
    }
    fn update(
        &mut self,
        data: &mut GameStateData<P>,
        config: &GameConfig<L>,
        input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        self.update_counter(&config.params, input);
        let handle_priority: Vec<Input> = vec![
            Input::HARD_DROP,
            Input::FIRM_DROP,
            Input::HOLD,
            Input::SOFT_DROP,
        ];
        for i in handle_priority {
            if !input.contains(i) {
                continue;
            }
            if i == Input::HARD_DROP {
                self.reset_counter();
                let fp = &mut data.falling_piece.unwrap();
                let n = fp.droppable_rows(&data.playfield);
                fp.y -= n as i32;
                let r = config.params.loss_condition.check(fp, &data.playfield);
                if !r.is_empty() {
                    return Ok(Some(Box::new(GameStateGameOver::by_lock_out(r))));
                }
                let r = fp.put_onto(&mut data.playfield);
                assert!(r.is_empty());
                for y in 0..data.playfield.visible_rows {
                    if data.playfield.grid.is_row_filled(y) {
                        return Ok(Some(Box::new(GameStateLineClear {})));
                    }
                }
                return Ok(Some(Box::new(GameStateSpawnPiece::default())));
            }
            if i == Input::FIRM_DROP {
                self.reset_counter();
                let fp = &mut data.falling_piece.unwrap();
                let n = fp.droppable_rows(&data.playfield);
                fp.y -= n as i32;
                break;
            }
            if i == Input::HOLD {
                if self.is_piece_held || !self.input_counter.should_hold() {
                    continue;
                }
                self.is_piece_held = true;
                let np = if let Some(p) = data.hold_piece {
                    p
                } else {
                    if data.next_pieces.is_empty() {
                        return Err("no next pieces".into());
                    }
                    data.next_pieces.pop_front().unwrap()
                };
                let fp = config.logic.spawn_piece(Some(np), &data.playfield);
                if !fp.can_put_onto(&data.playfield) {
                    return Ok(Some(Box::new(GameStateGameOver::default())));
                }
                data.hold_piece = Some(data.falling_piece.unwrap().piece);
                data.falling_piece = Some(fp);
                break;
            }
            let other = Input::SOFT_DROP
                | Input::MOVE_LEFT
                | Input::MOVE_RIGHT
                | Input::ROTATE_CW
                | Input::ROTATE_CCW;
            if other.contains(i) {
                let fp = data.falling_piece.unwrap();
                let num_droppable_rows = fp.droppable_rows(&data.playfield);
                if i == Input::SOFT_DROP {
                    if num_droppable_rows == 0 {
                        // lock
                    }
                    self.gravity_counter += config.params.soft_drop_gravity;
                }
                if self.input_counter.should_move_left(&config.params) {
                    //
                }
                if self.input_counter.should_move_right(&config.params) {
                    //
                }
                if self.input_counter.should_rotate_cw() {
                    //
                }
                if self.input_counter.should_rotate_ccw() {
                    //
                }
                if self.gravity_counter >= 1.0 {
                    //
                }
            }
        }
        Ok(None)
    }
}

struct GameStateLineClear;

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateLineClear {
    fn id(&self) -> GameStateId {
        GameStateId::LineClear
    }
    fn update(
        &mut self,
        _data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        // TODO
        Ok(None)
    }
}

#[derive(Default)]
struct GameStateSpawnPiece {}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateSpawnPiece {
    fn id(&self) -> GameStateId {
        GameStateId::SpawnPiece
    }
    fn update(
        &mut self,
        _data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        // TODO
        Ok(None)
    }
}

#[derive(Default)]
struct GameStateGameOver {
    loss_cond: LossCondition,
}

impl GameStateGameOver {
    fn by_lock_out(cond: LossCondition) -> Self {
        Self { loss_cond: cond }
    }
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateGameOver {
    fn id(&self) -> GameStateId {
        GameStateId::GameOver
    }
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "loss by {}", self.loss_cond)
    }
}

//---

pub struct Game<P: Piece, L> {
    pub config: GameConfig<L>,
    pub data: GameStateData<P>,
    state: Box<dyn GameState<P, L>>,
}

impl<P: Piece, L: GameLogic<P>> Game<P, L> {
    pub fn new(config: GameConfig<L>, data: GameStateData<P>) -> Self {
        Self {
            config: config,
            data: data,
            state: Box::new(GameStateInit {}),
        }
    }

    pub fn current_state(&self) -> GameStateId {
        self.state.id()
    }

    pub fn update(&mut self, input: Input) {
        let r = self.state.update(&mut self.data, &self.config, input);
        self.handle_result(r);
    }

    fn handle_result(&mut self, result: Result<Option<Box<dyn GameState<P, L>>>, String>) {
        match result {
            Ok(next) => {
                if let Some(next) = next {
                    self.state = next;
                    let r = self.state.enter(&mut self.data, &self.config);
                    self.handle_result(r);
                }
            }
            Err(reason) => {
                self.state = Box::new(GameStateError { reason: reason });
            }
        }
    }
}

//---

// #[derive(Debug, Copy, Clone, Default)]
// pub struct Counter {
//     pub move_left: Frames,
//     pub move_right: Frames,
//     pub gravity: Gravity,
//     pub are: Frames,
//     pub lock: Frames,
//     pub hold: bool,
//     pub line_clear: Frames,
// }

// impl Counter {
//     pub fn rows_to_be_dropped(&self) -> usize {
//         self.gravity as usize
//     }
// }

// #[derive(Debug, Copy, Clone)]
// pub enum State {
//     Init,
//     Play,
//     LineClear,
//     Are,
//     SpawnPiece,
//     GameOver,
// }

// #[derive(Debug, Clone)]
// pub struct GameState<P: Piece> {
//     pub playfield: Playfield<P>,
//     pub falling_piece: Option<FallingPiece<P>>,
//     pub hold_piece: Option<P>,
//     pub counter: Counter,
//     pub is_clearing_line: bool,
//     pub is_game_over: bool,
// }

// pub fn update<P: Piece, Logic: GameLogic<P>>(
//     logic: &Logic,
//     params: &GameParams,
//     state: &mut GameState<P>,
//     input: Input,
// ) {
//     if state.is_game_over {
//         return;
//     }

//     if input.contains(Input::MOVE_LEFT) {
//         state.counter.move_left += 1;
//     } else {
//         state.counter.move_left = 0;
//     }
//     if input.contains(Input::MOVE_RIGHT) {
//         state.counter.move_right += 1;
//     } else {
//         state.counter.move_right = 0;
//     }

//     if state.is_clearing_line {
//         state.counter.line_clear += 1;
//         if state.counter.line_clear > params.line_clear_delay {
//             state.counter.line_clear = 0;
//             state.is_clearing_line = false;
//         }
//         return;
//     }

//     if state.falling_piece.is_none() {
//         // Wait for ARE.
//         if state.counter.are <= params.are {
//             state.counter.are += 1;
//             return;
//         }
//         // ARE elapsed.
//         state.counter.are = 0;
//         // Spawn piece
//         let fp = logic.spawn_piece(None, &state.playfield);
//         let r =
//             state
//                 .playfield
//                 .grid
//                 .check_overlay(fp.x, fp.y, logic.piece_grid(fp.piece, fp.rotation));
//         if !r.is_empty() {
//             state.is_game_over = true;
//         }
//         state.falling_piece = Some(fp);
//         return;
//     }

//     if let Some(falling_piece) = state.falling_piece.as_mut() {
//         let piece_grid = logic.piece_grid(falling_piece.piece, falling_piece.rotation);

//         // TODO: shift
//         let is_shifted = true;

//         let num_droppable_rows = {
//             let (n, _r) = state.playfield.grid.check_overlay_toward(
//                 falling_piece.x as i32,
//                 falling_piece.y as i32,
//                 piece_grid,
//                 0,
//                 -1,
//             );
//             assert_ne!(0, n);
//             n - 1
//         } as i32;
//         if num_droppable_rows == 0 {
//             state.counter.lock += 1;
//         } else {
//             state.counter.gravity += params.gravity;
//         }

//         let mut should_lock = false;

//         if input.contains(Input::HARD_DROP) {
//             should_lock = true;
//         }

//         if input.contains(Input::FIRM_DROP) {
//             if num_droppable_rows > 0 {
//                 falling_piece.y -= num_droppable_rows;
//             }
//         }

//         let num_rows_to_be_dropped: i32;
//         if input.contains(Input::SOFT_DROP) {
//             state.counter.gravity += params.soft_drop_gravity;
//             num_rows_to_be_dropped = state.counter.gravity as i32;
//             should_lock = params.lock_delay_cancel && num_rows_to_be_dropped == 0;
//         // if !should_lock {
//         //     match params.lock_delay_reset {
//         //         LockDelayReset::EntryReset => {}
//         //         _ => state.counter.lock = 0,
//         //     }
//         // }
//         } else {
//             num_rows_to_be_dropped = state.counter.gravity as i32;
//         }

//         if num_rows_to_be_dropped > 0 {
//             if num_droppable_rows < num_rows_to_be_dropped {
//                 falling_piece.y -= num_droppable_rows;
//                 state.counter.gravity = 0.0;
//             } else {
//                 falling_piece.y -= num_rows_to_be_dropped;
//                 state.counter.gravity -= num_rows_to_be_dropped as f32;
//             }
//         }

//         if should_lock {
//             if params.loss_condition.contains(LossCondition::LOCK_OUT) {
//                 let padding =
//                     logic.piece_grid_bottom_padding(falling_piece.piece, falling_piece.rotation);
//                 state.is_game_over =
//                     falling_piece.y + padding as i32 >= state.playfield.visible_rows as i32;
//             }
//             if params
//                 .loss_condition
//                 .contains(LossCondition::PARTIAL_LOCK_OUT)
//             {
//                 let padding =
//                     logic.piece_grid_top_padding(falling_piece.piece, falling_piece.rotation);
//                 state.is_game_over = falling_piece.y + (piece_grid.num_rows() - padding) as i32
//                     >= state.playfield.visible_rows as i32;
//             }
//             if !state.is_game_over {
//                 let r = state.playfield.grid.overlay(
//                     falling_piece.x as i32,
//                     falling_piece.y as i32 - num_droppable_rows,
//                     piece_grid,
//                 );
//                 assert!(r.is_empty());
//                 state.falling_piece = None;
//             }
//             state.counter.gravity = 0.0;
//             state.counter.lock = 0;
//             state.counter.hold = false;
//             state.is_clearing_line = true;
//             return;
//         }

//         if state.counter.hold && input.contains(Input::HOLD) {
//             if let Some(p) = state.hold_piece {
//                 let fp = logic.spawn_piece(Some(p), &state.playfield);
//                 let r = state.playfield.grid.check_overlay(
//                     fp.x,
//                     fp.y,
//                     logic.piece_grid(fp.piece, fp.rotation),
//                 );
//                 if !r.is_empty() {
//                     state.is_game_over = true;
//                 }
//                 state.falling_piece = Some(fp);
//             }
//             state.counter.gravity = 0.0;
//             state.counter.lock = 0;
//             state.counter.hold = true;
//             return;
//         }

//         if input.contains(Input::ROTATE_CW | Input::ROTATE_CCW) {
//             let rotated = logic.rotate(
//                 input.contains(Input::ROTATE_CW),
//                 falling_piece,
//                 &state.playfield,
//             );
//             if let Some(fp) = rotated {
//                 state.falling_piece = Some(fp);
//             }
//             return;
//         }
//     }
// }
