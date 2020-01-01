use super::helper;
use mino_core::common::{
    Cell, Game, GameConfig, GameData, GameEvent, GameParams, Input, Playfield, TSpin,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn render_game(data: &GameData<Piece>) -> String {
    let mut s = String::with_capacity(1024);
    //---
    s.push_str("Hold: ");
    if let Some(p) = data.hold_piece {
        s.push_str(&format!("{}", p));
    }
    s.push('\n');
    //---
    s.push_str("Next: ");
    for i in 0..5 {
        if let Some(p) = data.next_pieces.get(i) {
            s.push_str(&format!("{}", p));
        } else {
            break;
        }
    }
    s.push('\n');
    //---
    s.push_str("--+----------\n");
    //---
    let pf = &data.playfield;
    let fp = &data.falling_piece;
    for py in (0..pf.visible_rows).rev() {
        s.push_str(&format!("{:>02}|", py + 1));
        for px in 0..pf.grid.num_cols() {
            if let Some(fp) = fp {
                let x = px as i32 - fp.x;
                let y = py as i32 - fp.y;
                if x >= 0 && y >= 0 && fp.grid().is_valid_cell_index(x as usize, y as usize) {
                    if let Cell::Block(p) = fp.grid().cell(x as usize, y as usize) {
                        s.push_str(&format!("{}", p));
                        continue;
                    }
                }
            }
            if let Cell::Block(p) = pf.grid.cell(px, py) {
                s.push_str(&format!("{}", p));
            } else {
                s.push(' ');
            }
        }
        s.push('\n');
    }
    //---
    s.push_str("--+----------\n");
    s.push_str("  |1234567890\n");
    //---
    for event in &data.events {
        match event {
            GameEvent::LineCleared(n, t) => {
                match t {
                    TSpin::None => {
                        if *n == 4 {
                            s.push_str("Tetris!");
                        } else if *n == 1 {
                            s.push_str("1 line cleared!");
                        } else {
                            s.push_str(&format!("{} lines cleared!", n));
                        }
                    }
                    TSpin::Mini => s.push_str(&format!(
                        "T-Spin Mini {}",
                        helper::tspin_num_to_en_str_long(*n as u8)
                    )),
                    TSpin::Normal => s.push_str(&format!(
                        "T-Spin {}",
                        helper::tspin_num_to_en_str_long(*n as u8)
                    )),
                }
                s.push('\n');
            }
            _ => {}
        }
    }
    //---
    s
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = {
        let config = GameConfig {
            params: GameParams {
                gravity: 0.0,
                are: 0,
                lock_delay: 60 * 60 * 60 * 24,
                line_clear_delay: 0,
                ..GameParams::default()
            },
            logic: WorldRuleLogic::default(),
        };
        let mut data = GameData::new(
            Playfield {
                visible_rows: 20,
                grid: PieceGrid::new(10, 40, vec![]),
            },
            None,
            None,
            helper::generate_pieces(),
            &config.params,
        );
        data.input_manager = mino_core::common::create_input_manager_for_automation();
        Game::new(config, data)
    };

    for _ in 0..20 {
        game.update(Input::default());
    }

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                // TODO
                if line == "p" {
                    print!("{}", render_game(game.data()));
                }
            }
            Err(ReadlineError::Interrupted) => {
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
