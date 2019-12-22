use input_counter::{InputCounter, InputManager};
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
    fn grid(&self, rotation: Rotation) -> &PieceGrid<Self>;
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

pub type PieceGrid<P> = grid::Grid<Cell<P>>;

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
    pub fn grid(&self) -> &PieceGrid<P> {
        self.piece.grid(self.rotation)
    }
    pub fn grid_top_padding(&self) -> usize {
        self.piece.grid_top_padding(self.rotation)
    }
    pub fn grid_bottom_padding(&self) -> usize {
        self.piece.grid_bottom_padding(self.rotation)
    }
    pub fn is_lock_out(&self, playfield: &Playfield<P>) -> bool {
        let padding = self.grid_bottom_padding();
        self.y + padding as i32 >= playfield.visible_rows as i32
    }
    pub fn is_partial_lock_out(&self, playfield: &Playfield<P>) -> bool {
        let padding = self.grid_top_padding();
        self.y + (self.grid().num_rows() - padding) as i32 >= playfield.visible_rows as i32
    }
    pub fn can_put_onto(&self, playfield: &Playfield<P>) -> bool {
        playfield
            .grid
            .check_overlay(self.x, self.y, &self.grid())
            .is_empty()
    }
    pub fn put_onto(&self, playfield: &mut Playfield<P>) -> grid::OverlayResult {
        playfield.grid.overlay(self.x, self.y, &self.grid())
    }
    pub fn droppable_rows(&self, playfield: &Playfield<P>) -> usize {
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

#[derive(Debug, Copy, Clone)]
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

pub trait GameLogic<P: Piece>: fmt::Debug {
    /// Create new falling piece at initial position.
    fn spawn_piece(
        &self,
        piece: P,
        num_cols: usize,
        num_rows: usize,
        num_visible_rows: usize,
    ) -> FallingPiece<P>;
    /// Rotate `falling_piece` on `playfield` by `cw`.
    /// If not rotatable, return None.
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

const INPUTS: [Input; 8] = [
    Input::HARD_DROP,
    Input::SOFT_DROP,
    Input::FIRM_DROP,
    Input::MOVE_LEFT,
    Input::MOVE_RIGHT,
    Input::ROTATE_CW,
    Input::ROTATE_CCW,
    Input::HOLD,
];

pub struct InputIterator {
    input: Input,
    next_idx: usize,
}

impl InputIterator {
    pub fn new(input: Input) -> Self {
        Self {
            input: input,
            next_idx: 0,
        }
    }
}

impl Iterator for InputIterator {
    type Item = Input;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next_idx < INPUTS.len() {
            if self.input.contains(INPUTS[self.next_idx]) {
                return Some(INPUTS[self.next_idx]);
            }
            self.next_idx += 1;
        }
        None
    }
}

impl IntoIterator for Input {
    type Item = Input;
    type IntoIter = InputIterator;

    fn into_iter(self) -> Self::IntoIter {
        InputIterator::new(self)
    }
}

pub fn new_input_manager(das: Frames, arr: Frames) -> InputManager<Input, Frames> {
    let mut mgr = InputManager::default();
    mgr.register(Input::HARD_DROP, InputCounter::new(0, 0));
    mgr.register(Input::SOFT_DROP, InputCounter::new(0, 0));
    mgr.register(Input::FIRM_DROP, InputCounter::new(0, 0));
    mgr.register(Input::MOVE_LEFT, InputCounter::new(das, arr));
    mgr.register(Input::MOVE_RIGHT, InputCounter::new(das, arr));
    mgr.register(Input::ROTATE_CW, InputCounter::new(0, 0));
    mgr.register(Input::ROTATE_CCW, InputCounter::new(0, 0));
    mgr.register(Input::HOLD, InputCounter::new(0, 0));
    mgr
}

#[derive(Debug, Clone)]
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
    pub input_mgr: InputManager<Input, Frames>,
}

#[derive(Debug, Copy, Clone)]
pub enum GameStateId {
    Init,
    Play,
    Lock,
    LineClear,
    SpawnPiece,
    GameOver,
    Error,
}

pub trait GameState<P: Piece, L>: fmt::Debug {
    fn id(&self) -> GameStateId;
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
    // fn exit(&mut self, _data: &mut GameStateData<P>, _config: &GameConfig<L>) {}
}

#[derive(Debug, Clone)]
struct GameStateError {
    reason: String,
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateError {
    fn id(&self) -> GameStateId {
        GameStateId::Error
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone, Default)]
struct GameStatePlay {
    gravity_counter: Gravity,
    lock_delay_counter: Frames,
    is_piece_held: bool,
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
        let input_mgr = &mut data.input_mgr;
        input_mgr.update(input.into_iter());
        let fp = data.falling_piece.as_mut().unwrap();
        let playfield = &data.playfield;
        let num_droppable_rows = fp.droppable_rows(playfield);

        // HARD_DROP
        if input.contains(Input::HARD_DROP) {
            fp.y -= num_droppable_rows as i32;
            return Ok(Some(Box::new(GameStateLock::default())));
        }

        // HOLD
        if !self.is_piece_held && input_mgr.handle(Input::HOLD) {
            self.is_piece_held = true;
            let np = if let Some(p) = data.hold_piece {
                p
            } else {
                if data.next_pieces.is_empty() {
                    return Err("no next pieces".into());
                }
                data.next_pieces.pop_front().unwrap()
            };
            let sfp = config.logic.spawn_piece(
                np,
                playfield.grid.num_cols(),
                playfield.grid.num_rows(),
                playfield.visible_rows,
            );
            if !sfp.can_put_onto(playfield) {
                return Ok(Some(Box::new(GameStateGameOver::default())));
            }
            data.hold_piece = Some(fp.piece);
            data.falling_piece = Some(sfp);
            self.gravity_counter = 0.0;
            self.lock_delay_counter = 0;
            return Ok(None);
        }

        // Others
        if num_droppable_rows == 0 {
            self.gravity_counter = 0.0;
            self.lock_delay_counter += 1;
            let should_lock = self.lock_delay_counter > config.params.lock_delay
                || (config.params.lock_delay_cancel && input.contains(Input::SOFT_DROP));
            if should_lock {
                return Ok(Some(Box::new(GameStateLock::default())));
            }
        } else if input.contains(Input::FIRM_DROP) {
            fp.y -= num_droppable_rows as i32;
            self.gravity_counter = 0.0;
            self.lock_delay_counter = 0;
            return Ok(None);
        } else {
            self.gravity_counter += config.params.gravity;
            if input.contains(Input::SOFT_DROP) {
                self.gravity_counter += config.params.soft_drop_gravity;
            }
        }
        let mut moved = fp.clone();
        let dx = if input_mgr.handle(Input::MOVE_LEFT) {
            -1
        } else if input_mgr.handle(Input::MOVE_RIGHT) {
            1
        } else {
            0
        };
        if dx != 0 {
            let mut t = moved;
            t.x -= dx;
            if t.can_put_onto(playfield) {
                moved = t;
            }
        }
        let rotate = if input_mgr.handle(Input::ROTATE_CW) {
            (true, true)
        } else if input_mgr.handle(Input::ROTATE_CCW) {
            (true, false)
        } else {
            (false, false)
        };
        if rotate.0 {
            if let Some(rotated) = config.logic.rotate(rotate.1, &moved, playfield) {
                moved = rotated;
            }
        }
        let num_droppable_rows = moved.droppable_rows(playfield);
        if num_droppable_rows == 0 {
            self.gravity_counter = 0.0;
        } else if self.gravity_counter >= 1.0 {
            moved.y -= std::cmp::min(num_droppable_rows, self.gravity_counter as usize) as i32;
            self.gravity_counter = 0.0;
            self.lock_delay_counter = 0;
        }
        data.falling_piece = Some(moved);
        Ok(None)
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct GameStateLock;

impl GameStateLock {
    fn lock<P: Piece, L: GameLogic<P>>(
        &mut self,
        data: &mut GameStateData<P>,
        config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        let fp = &data.falling_piece.unwrap();
        let r = config.params.loss_condition.check(fp, &data.playfield);
        if !r.is_empty() {
            return Ok(Some(Box::new(GameStateGameOver::by_lock_out(r))));
        }
        let r = fp.put_onto(&mut data.playfield);
        assert!(r.is_empty());
        for y in 0..data.playfield.visible_rows {
            if data.playfield.grid.is_row_filled(y) {
                return Ok(Some(Box::new(GameStateLineClear::default())));
            }
        }
        Ok(Some(Box::new(GameStateSpawnPiece::default())))
    }
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateLock {
    fn id(&self) -> GameStateId {
        GameStateId::Lock
    }
    fn enter(
        &mut self,
        data: &mut GameStateData<P>,
        _config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if data.falling_piece.is_none() {
            return Err("falling_piece should not be none".into());
        }
        // NOTE: call self.lock() here if zero frame transition required
        Ok(None)
    }
    fn update(
        &mut self,
        data: &mut GameStateData<P>,
        config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        self.lock(data, config)
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct GameStateLineClear {
    frame_count: Frames,
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateLineClear {
    fn id(&self) -> GameStateId {
        GameStateId::LineClear
    }
    fn update(
        &mut self,
        data: &mut GameStateData<P>,
        config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if self.frame_count == 0 {
            let n = data.playfield.grid.pluck_filled_rows(Some(Cell::Empty));
            if n == 0 {
                // TODO: no lines cleared!?
            }
        }
        self.frame_count += 1;
        if self.frame_count <= config.params.line_clear_delay {
            return Ok(None);
        }
        Ok(Some(Box::new(GameStateSpawnPiece::default())))
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct GameStateSpawnPiece {
    frame_count: Frames,
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateSpawnPiece {
    fn id(&self) -> GameStateId {
        GameStateId::SpawnPiece
    }
    fn update(
        &mut self,
        data: &mut GameStateData<P>,
        config: &GameConfig<L>,
        input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if self.frame_count == 0 {
            if let Some(next) = data.next_pieces.pop_front() {
                let fp = config.logic.spawn_piece(
                    next,
                    data.playfield.grid.num_cols(),
                    data.playfield.grid.num_rows(),
                    data.playfield.visible_rows,
                );
                data.falling_piece = Some(fp);
            } else {
                return Err("no next piece found".into());
            };
        }
        self.frame_count += 1;
        data.input_mgr.update(input.into_iter());
        if self.frame_count < config.params.are {
            return Ok(None);
        }
        Ok(Some(Box::new(GameStatePlay::default())))
    }
}

#[derive(Debug, Copy, Clone, Default)]
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
}

//---

#[derive(Debug)]
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
