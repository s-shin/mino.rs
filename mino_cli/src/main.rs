extern crate mino_core;

use mino_core::common::grid::GridFormatterOptions;
use mino_core::common::{Counter, Game, GameParams, GameState, Input, Playfield};
use mino_core::tetro::*;

fn main() {
    let logic = WorldRuleLogic {};
    let state = GameState::<Piece> {
        playfield: Playfield::<Piece> {
            visible_rows: 20,
            grid: Grid::new(10, 40, vec![]),
        },
        falling_piece: Option::None,
        hold_piece: Option::None,
        next_pieces: vec![],
        counter: Counter::default(),
        is_game_over: false,
    };
    let mut game = Game {
        logic: logic,
        params: GameParams::default(),
        state: state,
    };
    game.update(Input::default());

    // let mut field = Grid::new(10, 40, vec![]);
    // for x in 1..9 {
    //     field.set_cell(x, 0, Cell::Garbage);
    // }
    // field.set_cell(0, 1, Cell::Block(Piece::O));
    // field.set_cell(1, 1, Cell::Block(Piece::O));
    // field.set_cell(0, 2, Cell::Block(Piece::O));
    // field.set_cell(1, 2, Cell::Block(Piece::O));
    // let _cell = field.cell(0, 0);
    // // println!("Field: {:?}", field);
    // println!(
    //     "{}",
    //     GridFormatter {
    //         grid: field,
    //         opts: GridFormatterOptions {
    //             str_begin_of_line: "|",
    //             str_end_of_line: "|",
    //             range_y: Some(0..20),
    //             ..GridFormatterOptions::default()
    //         }
    //     }
    // )
}
