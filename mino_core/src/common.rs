use input_counter::{Contains, InputCounter, InputManager};
use std::collections::{HashMap, VecDeque};
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
        match ((*self as i16) + (n as i16) + 4) % 4 {
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

//--- Piece, FallingPiece, Playfield

pub trait Piece: Copy {
    fn grid(&self, rotation: Rotation) -> &PieceGrid<Self>;
    fn grid_top_padding(&self, rotation: Rotation) -> usize {
        self.grid(rotation).top_padding()
    }
    fn grid_bottom_padding(&self, rotation: Rotation) -> usize {
        self.grid(rotation).bottom_padding()
    }
}

pub type PieceGrid<P> = grid::Grid<Cell<P>>;

#[derive(Debug, Copy, Clone)]
pub enum Cell<P: Piece> {
    Empty,
    Block(P),
    Ghost(P),
    Garbage,
}

impl<P: Piece> grid::IsEmpty for Cell<P> {
    fn is_empty(&self) -> bool {
        match self {
            Cell::Empty | Cell::Ghost(_) => true,
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
            Cell::Empty | Cell::Ghost(_) => write!(formatter, " "),
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
        if n == 0 {
            0
        } else {
            n - 1
        }
    }
}

#[derive(Debug, Clone)]
pub struct Playfield<P: Piece> {
    pub visible_rows: usize,
    pub grid: grid::Grid<Cell<P>>,
}

//--- GameParams, GameLogic, GameConfig

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
    pub struct TopOutCondition: u32 {
        const LOCK_OUT = 0b00000001;
        const PARTIAL_LOCK_OUT = 0b00000010;
        const GARBAGE_OUT = 0b00000100;
    }
}

impl TopOutCondition {
    fn check<P: Piece>(
        self,
        falling_piece: &FallingPiece<P>,
        playfield: &Playfield<P>,
    ) -> TopOutCondition {
        if self.contains(TopOutCondition::LOCK_OUT) {
            if falling_piece.is_lock_out(playfield) {
                return self;
            }
        }
        if self.contains(TopOutCondition::PARTIAL_LOCK_OUT) {
            if falling_piece.is_partial_lock_out(playfield) {
                return self;
            }
        }
        return Self::empty();
    }
}

impl Default for TopOutCondition {
    fn default() -> Self {
        TopOutCondition::LOCK_OUT | TopOutCondition::GARBAGE_OUT
    }
}

impl fmt::Display for TopOutCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum GameOverReason {
    BlockOut,
    LockOut,
    PartialLockOut,
    GarbageOut,
}

impl From<TopOutCondition> for Option<GameOverReason> {
    fn from(c: TopOutCondition) -> Self {
        if c.contains(TopOutCondition::PARTIAL_LOCK_OUT) {
            return Some(GameOverReason::PartialLockOut);
        }
        if c.contains(TopOutCondition::LOCK_OUT) {
            return Some(GameOverReason::LockOut);
        }
        if c.contains(TopOutCondition::GARBAGE_OUT) {
            return Some(GameOverReason::GarbageOut);
        }
        None
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
    pub top_out_condition: TopOutCondition,
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
            top_out_condition: TopOutCondition::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TSpin {
    None,
    Normal,
    Mini,
}

impl Default for TSpin {
    fn default() -> Self {
        TSpin::None
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
    ) -> Option<(FallingPiece<P>, TSpin)>;
}

#[derive(Debug, Clone)]
pub struct GameConfig<Logic> {
    pub logic: Logic,
    pub params: GameParams,
}

//--- Input

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
            let idx = self.next_idx;
            self.next_idx += 1;
            if self.input.contains(INPUTS[idx]) {
                return Some(INPUTS[idx]);
            }
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

impl Contains<Input> for Input {
    fn contains(&self, input: Input) -> bool {
        Input::contains(self, input)
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

//--- GameEventHandler

#[derive(Debug)]
pub enum GameEvent {
    Update(Input),
    LineCleared(usize, TSpin),
}

pub trait GameEventHandler<G>: fmt::Debug {
    fn handle(&mut self, _game: &G, _event: &GameEvent) {}
}

pub type GameEventHandlerId = u32;

#[derive(Debug)]
pub struct GameEventHandlerManager<G> {
    last_id: GameEventHandlerId,
    handlers: HashMap<GameEventHandlerId, Box<dyn GameEventHandler<G>>>,
}

impl<G> GameEventHandlerManager<G> {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            handlers: HashMap::new(),
        }
    }
    pub fn add(&mut self, handler: Box<dyn GameEventHandler<G>>) -> GameEventHandlerId {
        let id = self.last_id;
        self.last_id += 1;
        self.handlers.insert(id, handler);
        id
    }
    pub fn remove(&mut self, id: GameEventHandlerId) -> bool {
        self.handlers.remove(&id).is_some()
    }
    // pub fn get(&self, id: GameEventHandlerId) -> Option<&Box<dyn GameEventHandler<G>>> {
    //     self.handlers.get(&id)
    // }
    // pub fn get_mut(&mut self, id: GameEventHandlerId) -> Option<&mut Box<dyn GameEventHandler<G>>> {
    //     self.handlers.get_mut(&id)
    // }
}

impl<G: fmt::Debug> GameEventHandler<G> for GameEventHandlerManager<G> {
    fn handle(&mut self, game: &G, event: &GameEvent) {
        for (_, handler) in &mut self.handlers {
            handler.handle(game, event);
        }
    }
}

//--- GameData

#[derive(Debug, Clone)]
pub struct GameData<P: Piece> {
    pub playfield: Playfield<P>,
    pub falling_piece: Option<FallingPiece<P>>,
    pub hold_piece: Option<P>,
    pub next_pieces: VecDeque<P>,
    pub input_mgr: InputManager<Input, Frames>,
    pub tspin: TSpin,
}

//--- GameState

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GameStateId {
    Init,
    Play,
    Lock,
    LineClear,
    SpawnPiece,
    GameOver,
    Error,
}

trait GameStateEventHandler {
    fn handle(&mut self, event: &GameEvent);
}

trait GameState<P: Piece, L>: fmt::Debug {
    fn id(&self) -> GameStateId;
    fn enter(
        &mut self,
        _data: &mut GameData<P>,
        _config: &GameConfig<L>,
        // _event_handler: &mut H,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        Ok(None)
    }
    fn update(
        &mut self,
        _data: &mut GameData<P>,
        _config: &GameConfig<L>,
        _input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        Ok(None)
    }
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
        data: &mut GameData<P>,
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
        data: &mut GameData<P>,
        _config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        if data.falling_piece.is_none() {
            return Err("falling_piece should not be none".into());
        }
        Ok(None)
    }
    fn update(
        &mut self,
        data: &mut GameData<P>,
        config: &GameConfig<L>,
        input: Input,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        let input_mgr = &mut data.input_mgr;
        input_mgr.update(input);
        let fp = data.falling_piece.as_mut().unwrap();
        let playfield = &data.playfield;
        let num_droppable_rows = fp.droppable_rows(playfield);

        // HARD_DROP
        if input.contains(Input::HARD_DROP) {
            fp.y -= num_droppable_rows as i32;
            return Ok(Some(Box::new(GameStateLock::new())));
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
                return Ok(Some(Box::new(GameStateGameOver::new(
                    GameOverReason::BlockOut,
                ))));
            }
            data.hold_piece = Some(fp.piece);
            data.falling_piece = Some(sfp);
            data.tspin = TSpin::None;
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
                return Ok(Some(Box::new(GameStateLock::new())));
            }
        } else if input.contains(Input::FIRM_DROP) {
            fp.y -= num_droppable_rows as i32;
            data.tspin = TSpin::None;
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
            t.x += dx;
            if t.can_put_onto(playfield) {
                moved = t;
                data.tspin = TSpin::None;
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
            if let Some(r) = config.logic.rotate(rotate.1, &moved, playfield) {
                moved = r.0;
                data.tspin = r.1;
            }
        }
        let num_droppable_rows = moved.droppable_rows(playfield);
        if num_droppable_rows == 0 {
            self.gravity_counter = 0.0;
        } else if self.gravity_counter >= 1.0 {
            moved.y -= std::cmp::min(num_droppable_rows, self.gravity_counter as usize) as i32;
            data.tspin = TSpin::None;
            self.gravity_counter = 0.0;
            self.lock_delay_counter = 0;
        }
        data.falling_piece = Some(moved);
        Ok(None)
    }
}

#[derive(Debug, Copy, Clone)]
struct GameStateLock {}

impl GameStateLock {
    fn new() -> Self {
        Self {}
    }

    fn lock<P: Piece, L: GameLogic<P>>(
        &mut self,
        data: &mut GameData<P>,
        config: &GameConfig<L>,
    ) -> Result<Option<Box<dyn GameState<P, L>>>, String> {
        let fp = &data.falling_piece.unwrap();
        let r = config.params.top_out_condition.check(fp, &data.playfield);
        if !r.is_empty() {
            let r: Option<GameOverReason> = r.into();
            return Ok(Some(Box::new(GameStateGameOver::new(r.unwrap()))));
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
        data: &mut GameData<P>,
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
        data: &mut GameData<P>,
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
        data: &mut GameData<P>,
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
        data: &mut GameData<P>,
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
                if !fp.can_put_onto(&data.playfield) {
                    return Ok(Some(Box::new(GameStateGameOver::new(
                        GameOverReason::LockOut,
                    ))));
                }
            } else {
                return Err("no next piece found".into());
            };
        }
        self.frame_count += 1;
        data.input_mgr.update(input);
        if self.frame_count <= config.params.are {
            return Ok(None);
        }
        Ok(Some(Box::new(GameStatePlay::default())))
    }
}

#[derive(Debug, Copy, Clone)]
struct GameStateGameOver {
    reason: GameOverReason,
}

impl GameStateGameOver {
    fn new(reason: GameOverReason) -> Self {
        Self { reason: reason }
    }
}

impl<P: Piece, L: GameLogic<P>> GameState<P, L> for GameStateGameOver {
    fn id(&self) -> GameStateId {
        GameStateId::GameOver
    }
}

//--- Game

#[derive(Debug)]
pub struct Game<P: Piece, L> {
    config: GameConfig<L>,
    data: GameData<P>,
    event_handler_mgr: GameEventHandlerManager<Self>,
    frame_num: u64,
    state: Box<dyn GameState<P, L>>,
}

impl<P: Piece, L: GameLogic<P>> Game<P, L> {
    pub fn new(config: GameConfig<L>, data: GameData<P>) -> Self {
        Self {
            config: config,
            data: data,
            event_handler_mgr: GameEventHandlerManager::new(),
            frame_num: 0,
            state: Box::new(GameStateInit {}),
        }
    }

    pub fn config(&self) -> &GameConfig<L> {
        &self.config
    }
    pub fn data(&self) -> &GameData<P> {
        &self.data
    }
    pub fn frame_num(&self) -> u64 {
        self.frame_num
    }
    pub fn state_id(&self) -> GameStateId {
        self.state.id()
    }

    pub fn update(&mut self, input: Input) {
        self.frame_num += 1;
        let r = self.state.update(&mut self.data, &self.config, input);
        self.handle_result(r);
    }

    fn handle_result(&mut self, result: Result<Option<Box<dyn GameState<P, L>>>, String>) {
        match result {
            Ok(maybe_next) => {
                if let Some(next) = maybe_next {
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

    pub fn append_next_pieces(&mut self, pieces: &mut VecDeque<P>) {
        self.data.next_pieces.append(pieces)
    }
}
