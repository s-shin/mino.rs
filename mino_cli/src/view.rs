extern crate cursive;
use cursive::direction::Direction;
use cursive::event::{Event, EventResult};
use cursive::vec::Vec2;
use cursive::Printer;
use mino_core::common::GameStateData;
use mino_core::tetro::Piece;

pub struct GameView {
    data: *const GameStateData<Piece>,
}

impl GameView {
    pub fn new(data: *const GameStateData<Piece>) -> Self {
        Self { data: data }
    }
}

impl cursive::view::View for GameView {
    fn draw(&self, printer: &Printer) {
        printer.print((0, 0), "TODO");
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            _ => {
                //
            }
        }

        EventResult::Ignored
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        (10, 20).into()
    }
}
