mod cmd {
    use std::error::Error;
    use std::fmt;
    use std::str::{FromStr, SplitAsciiWhitespace};

    pub struct KeyValue<'a, V> {
        pub key: &'a str,
        pub value: V,
    }

    impl<'a, V> KeyValue<'a, V> {
        fn new(k: &'a str, v: V) -> Self {
            KeyValue { key: k, value: v }
        }
    }

    pub type Arg<'a> = KeyValue<'a, Option<&'a str>>;

    impl<'a> Arg<'a> {
        pub fn parse_value<T: FromStr>(&self) -> Result<KeyValue<'a, Option<T>>, T::Err> {
            Ok(KeyValue::new(
                self.key,
                if let Some(v) = self.value {
                    Some(v.parse::<T>()?)
                } else {
                    None
                },
            ))
        }
    }

    impl<'a> fmt::Display for Arg<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            if let Some(v) = self.value {
                write!(f, "{}={}", self.key, v)
            } else {
                write!(f, "{}", self.key)
            }
        }
    }

    pub struct ArgIterator<'a, Iter: Iterator<Item = &'a str>> {
        iter: Iter,
    }

    impl<'a, Iter: Iterator<Item = &'a str>> ArgIterator<'a, Iter> {
        pub fn new(iter: Iter) -> Self {
            ArgIterator { iter: iter }
        }
    }

    impl<'a, Iter: Iterator<Item = &'a str>> Iterator for ArgIterator<'a, Iter> {
        type Item = Arg<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            for item in &mut self.iter {
                let ss: Vec<&str> = item.splitn(2, '=').collect();
                match ss.len() {
                    1 => return Some(KeyValue::new(ss[0], None)),
                    2 => return Some(KeyValue::new(ss[0], Some(ss[1]))),
                    _ => continue,
                }
            }
            None
        }
    }

    pub fn parse_command_line(
        line: &str,
    ) -> Result<(&str, ArgIterator<SplitAsciiWhitespace>), Box<dyn Error>> {
        let mut iter = line.trim().split_ascii_whitespace();
        let cmd = {
            let item = iter.next();
            if let Some(cmd) = item {
                cmd
            } else {
                return Err("TODO".into());
            }
        };
        Ok((cmd, ArgIterator::new(iter)))
    }
}

use super::helper;
use mino_core::common::{
    Cell, Game, GameConfig, GameData, GameEvent, GameParams, GameStateId, Input, Playfield, TSpin,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::VecDeque;
use std::error::Error;
use std::io;
use termion::color;

fn format_game_data(data: &GameData<Piece>) -> String {
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

trait Renderer {
    fn render_game_data(&mut self, data: &GameData<Piece>) -> Result<(), Box<dyn Error>>;
    fn render_error(&mut self, err: &dyn Error) -> Result<(), Box<dyn Error>>;
    fn render_error_str(&mut self, err: &str) -> Result<(), Box<dyn Error>> {
        let err: Box<dyn Error> = err.into();
        self.render_error(&*err)
    }
    fn render_message(&mut self, msg: &str) -> Result<(), Box<dyn Error>>;
}

struct HumanReadableRenderer<W: io::Write> {
    w: W,
}

impl<W: io::Write> Renderer for HumanReadableRenderer<W> {
    fn render_game_data(&mut self, data: &GameData<Piece>) -> Result<(), Box<dyn Error>> {
        write!(self.w, "{}", format_game_data(data))?;
        Ok(())
    }
    fn render_error(&mut self, err: &dyn Error) -> Result<(), Box<dyn Error>> {
        writeln!(self.w, "ERROR: {}", err)?;
        Ok(())
    }
    fn render_message(&mut self, msg: &str) -> Result<(), Box<dyn Error>> {
        writeln!(self.w, "{}", msg)?;
        Ok(())
    }
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

#[derive(Debug, Clone, Copy, Default)]
struct Opts {
    autogen: bool,
}

struct App {
    game: Game<Piece, WorldRuleLogic>,
    opts: Opts,
}

impl App {
    fn new() -> Self {
        Self {
            game: new_game(),
            opts: Opts::default(),
        }
    }

    fn parse_line<R: Renderer>(
        &mut self,
        line: &str,
        renderer: &mut R,
    ) -> Result<(), Box<dyn Error>> {
        let (cmd, args) = {
            match cmd::parse_command_line(line) {
                Ok(x) => x,
                Err(err) => {
                    renderer.render_error(&*err)?;
                    return Ok(());
                }
            }
        };
        if cmd.chars().nth(0) == Some('#') {
            return Ok(());
        }
        match cmd {
            "help" | "?" => {
                renderer.render_message("TODO")?;
            }
            "quit" | "q" => {
                renderer.render_message("TODO")?;
            }
            "setup" => {
                self.game = new_game();
            }
            "print" | "p" => {
                renderer.render_game_data(self.game.data())?;
            }
            "move" | "mv" => {
                for arg in args {
                    let (mv, count) = match arg.parse_value::<usize>() {
                        Ok(kv) => (kv.key, kv.value.unwrap_or(1)),
                        Err(e) => return renderer.render_error(&e),
                    };
                    for _ in 0..count {
                        match &*mv.to_lowercase() {
                            "l" | "left" => {
                                self.input(Input::MOVE_LEFT);
                            }
                            "ll" => {
                                for _ in 0..10 {
                                    self.input(Input::MOVE_LEFT);
                                }
                            }
                            "r" | "right" => {
                                self.input(Input::MOVE_RIGHT);
                            }
                            "rr" => {
                                for _ in 0..10 {
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
                                return renderer.render_error_str(&format!("unknown move: {}", mv));
                            }
                        }
                    }
                }
                renderer.render_game_data(self.game.data())?;
            }
            "set" => {
                for arg in args {
                    match arg.key {
                        "autogen" => {
                            if let Some(v) = arg.value {
                                self.opts.autogen = match v.parse::<bool>() {
                                    Ok(v) => v,
                                    Err(err) => {
                                        return renderer.render_error(&err);
                                    }
                                };
                                self.gen(false);
                            } else {
                                return renderer.render_error_str("value is required");
                            }
                        }
                        _ => {
                            return renderer
                                .render_error_str(&format!("unknown option: {}", arg.key));
                        }
                    }
                }
            }
            "next" => {
                for arg in args {
                    match arg.key {
                        "set" | "add" => {
                            let mut pieces: VecDeque<Piece> = VecDeque::new();
                            for c in arg.value.unwrap_or("").chars() {
                                pieces.push_back((&c.to_string()).parse::<Piece>()?);
                            }
                            if arg.key == "set" {
                                self.game.set_next_pieces(pieces);
                            } else {
                                self.game.append_next_pieces(&mut pieces);
                            }
                        }
                        "auto" => match arg.value {
                            Some("force") => self.gen(true),
                            Some(v) => {
                                return renderer.render_error_str(&format!("invalid value: {}", v));
                            }
                            None => self.gen(false),
                        },
                        _ => {
                            return renderer.render_error_str(&format!("unknown op: {}", arg.key));
                        }
                    }
                }
            }
            "history" => {
                renderer.render_message("TODO")?;
            }
            _ => {
                return renderer.render_error_str(&format!("unknown command: {}", cmd));
            }
        }
        Ok(())
    }

    fn input(&mut self, input: Input) {
        self.game.update(input);
        helper::update_util(&mut self.game, GameStateId::Play, 1000);
        self.gen(false);
    }

    fn gen(&mut self, force: bool) {
        if force || (self.opts.autogen && self.game.data().next_pieces.len() <= 7) {
            self.generate_next_pieces();
        }
    }

    fn generate_next_pieces(&mut self) {
        let mut ps = helper::generate_pieces();
        self.game.append_next_pieces(&mut ps);
    }
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let mut renderer = HumanReadableRenderer { w: io::stdout() };
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                app.parse_line(&line, &mut renderer)?;
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
