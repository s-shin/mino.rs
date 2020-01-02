use super::helper;
use mino_core::common::{
    Cell, Game, GameConfig, GameData, GameEvent, GameParams, GameStateId, Input, Playfield, TSpin,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::error::Error;
use std::io;
use termion::color;

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
                        s.push_str(&format!(
                            "{}{}{}",
                            color::Fg(color::Yellow),
                            p,
                            color::Fg(color::Reset)
                        ));
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

fn new_game() -> Game<Piece, WorldRuleLogic> {
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
    let mut game = Game::new(config, data);
    helper::update_util(&mut game, GameStateId::Play, 1000);
    game
}

struct App {
    game: Game<Piece, WorldRuleLogic>,
    autogen: bool,
}

impl App {
    fn new() -> Self {
        Self {
            game: new_game(),
            autogen: true,
        }
    }

    fn parse_line<W: io::Write>(&mut self, line: &str, w: &mut W) -> Result<(), Box<dyn Error>> {
        let args: Vec<(&str, Option<&str>)> = line
            .trim()
            .split_ascii_whitespace()
            .map(|s| {
                let ss: Vec<&str> = s.split('=').collect();
                if ss.len() == 1 {
                    (ss[0], None)
                } else {
                    (ss[0], Some(ss[1]))
                }
            })
            .collect();
        if args.len() == 0 || args[0].0.chars().nth(0) == Some('#') {
            return Ok(());
        }
        match args[0].0 {
            "help" | "?" => {
                writeln!(w, "TODO")?;
            }
            "setup" => {
                self.setup();
            }
            "print" | "p" => {
                self.print();
            }
            "move" | "mv" => {
                for mv in args.iter().skip(1) {
                    let count = if let Some(s) = mv.1 {
                        match s.parse::<usize>() {
                            Ok(x) => x,
                            Err(e) => {
                                writeln!(w, "{}: {}", e, s)?;
                                return Ok(());
                            }
                        }
                    } else {
                        1
                    };
                    for _ in 0..count {
                        let s = &*mv.0;
                        match &*s.to_lowercase() {
                            "l" | "left" => {
                                self.input(Input::MOVE_LEFT);
                            }
                            "ll" => {
                                for _ in 0..5 {
                                    self.input(Input::MOVE_LEFT);
                                }
                            }
                            "r" | "right" => {
                                self.input(Input::MOVE_RIGHT);
                            }
                            "rr" => {
                                for _ in 0..5 {
                                    self.input(Input::MOVE_RIGHT);
                                }
                            }
                            "d" | "softdrop" => {
                                self.input(Input::SOFT_DROP);
                            }
                            "hd" | "harddrop" => {
                                self.input(Input::HARD_DROP);
                            }
                            "fd" | "firmdrop" => {
                                self.input(Input::FIRM_DROP);
                            }
                            "cw" => {
                                self.input(Input::ROTATE_CW);
                            }
                            "ccw" => {
                                self.input(Input::ROTATE_CCW);
                            }
                            "h" | "hold" => {
                                self.input(Input::HOLD);
                            }
                            _ => {
                                writeln!(w, "unknown move string: {}", s)?;
                                return Ok(());
                            }
                        }
                    }
                }
                self.print();
            }
            "autogen" => {
                self.autogen = !self.autogen;
                self.autogen();
                writeln!(
                    w,
                    "autogen {}",
                    if self.autogen { "enabled" } else { "disabled" }
                )?;
            }
            "gen" => {
                self.gen();
            }
            "history" => {
                writeln!(w, "TODO")?;
            }
            _ => {
                println!("unknown command: {}", args[0].0);
            }
        }
        Ok(())
    }

    fn setup(&mut self) {
        self.game = new_game();
    }

    fn print(&self) {
        print!("{}", render_game(self.game.data()));
    }

    fn input(&mut self, input: Input) {
        self.game.update(input);
        helper::update_util(&mut self.game, GameStateId::Play, 1000);
        self.autogen();
    }

    fn autogen(&mut self) {
        if self.autogen && self.game.data().next_pieces.len() <= 7 {
            self.gen();
        }
    }

    fn gen(&mut self) {
        let mut ps = helper::generate_pieces();
        self.game.append_next_pieces(&mut ps);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                app.parse_line(&line, &mut io::stdout())?;
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
