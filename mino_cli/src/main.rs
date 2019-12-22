extern crate grid;
extern crate mino_core;
use mino_core::common::{
    new_input_manager, Game, GameConfig, GameParams, GameStateData, Input, Playfield,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use std::collections::VecDeque;

fn main() {
    let mut game = {
        let config = GameConfig {
            params: GameParams {
                gravity: 0.0,
                are: 0,
                line_clear_delay: 0,
                ..GameParams::default()
            },
            logic: WorldRuleLogic {},
        };
        let data = GameStateData {
            playfield: Playfield {
                visible_rows: 20,
                grid: PieceGrid::new(10, 40, vec![]),
            },
            falling_piece: Option::None,
            hold_piece: Option::None,
            next_pieces: VecDeque::from(vec![Piece::J, Piece::O, Piece::I]),
            input_mgr: new_input_manager(config.params.das, config.params.arr),
        };
        Game::new(config, data)
    };
    for _i in 0..3 {
        println!("{:?}", game.current_state());
        game.update(Input::default());
    }
    println!("{:?}", game);
    let mut pf = game.data.playfield.clone();
    if let Some(fp) = game.data.falling_piece {
        let r = fp.put_onto(&mut pf);
        println!("yes {:?}", r)
    }
    println!(
        "{}",
        grid::GridFormatter {
            grid: &pf.grid,
            opts: grid::GridFormatOptions {
                str_begin_of_line: "|",
                str_end_of_line: "|",
                range_y: Some(0..pf.visible_rows),
                ..grid::GridFormatOptions::default()
            }
        }
    );
}
