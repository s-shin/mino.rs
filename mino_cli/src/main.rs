extern crate grid;
extern crate mino_core;
use cursive::event::Event;
use cursive::theme::BaseColor::Black;
use cursive::theme::Color::Dark;
use cursive::theme::PaletteColor::Background;
use cursive::Cursive;
use mino_core::common::{
    new_input_manager, Game, GameConfig, GameParams, GameStateData, GameStateId, Input, Playfield,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use std::collections::VecDeque;

// fn _render(game: &Game<Piece, WorldRuleLogic>) {
//     println!("---");
//     println!("{:?}", game);
//     let mut pf = game.data.playfield.clone();
//     if let Some(fp) = game.data.falling_piece {
//         let r = fp.put_onto(&mut pf);
//         println!("yes {:?}", r)
//     }
//     println!(
//         "{}",
//         grid::GridFormatter {
//             grid: &pf.grid,
//             opts: grid::GridFormatOptions {
//                 str_begin_of_line: "|",
//                 str_end_of_line: "|",
//                 range_y: Some(0..pf.visible_rows),
//                 ..grid::GridFormatOptions::default()
//             }
//         }
//     );
// }

mod view;

const FPS: u32 = 2;

fn main() {
    let mut game = {
        let config = GameConfig {
            params: GameParams {
                gravity: 5.0,
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

    let mut siv = Cursive::default();
    siv.set_fps(FPS);
    let mut theme = siv.current_theme().clone();
    theme.shadow = false;
    theme.palette[Background] = Dark(Black);
    siv.set_theme(theme);
    siv.add_layer(view::GameView::new(&game.data));
    siv.set_user_data(game);
    siv.add_global_callback('q', |s| s.quit());
    siv.add_global_callback(Event::Refresh, |_| {
        //
    });
    siv.run();

    // let mut view_mgr = view::Manager::new(&game.data);
    // for i in 0.
    // view_mgr.run();
    // for i in 0..301 {
    //     game.update(Input::default());
    //     view_mgr.update();
    //     std::thread::sleep(std::time::Duration::from_millis(500));
    //     assert!(i < 300);
    //     if game.current_state_id() == GameStateId::Play {
    //         break;
    //     }
    // }
    // for _i in 0..300 {
    //     game.update(Input::default());
    //     view_mgr.update();
    //     std::thread::sleep(std::time::Duration::from_millis(500));
    // }
}
