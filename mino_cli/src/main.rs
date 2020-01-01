extern crate clap;
extern crate grid;
extern crate mino_core;
extern crate rand;
extern crate termion;
extern crate tui;
use clap::{App, SubCommand};
use mino_core::common::{
    Game, GameConfig, GameData, GameEvent, GameParams, GameStateId, Input, Playfield,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use std::time;
use termion::event::{Event, Key};
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Paragraph, Text, Widget};
use tui::Terminal;

mod renderer;

fn init_terminal() -> Result<
    (
        Terminal<TermionBackend<RawTerminal<io::Stdout>>>,
        io::Bytes<termion::AsyncReader>,
    ),
    Box<dyn std::error::Error>,
> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let stdin = termion::async_stdin().bytes();
    Ok((terminal, stdin))
}

fn generate_pieces() -> VecDeque<Piece> {
    let mut rng = rand::thread_rng();
    let mut ps = Piece::slice().clone();
    ps.shuffle(&mut rng);
    ps.to_vec().into()
}

fn update_util(game: &mut Game<Piece, WorldRuleLogic>, state_id: GameStateId, limit: i32) -> bool {
    for i in 0.. {
        if game.state_id() == state_id {
            return true;
        }
        game.update(Input::default());
        if limit > 0 && i > limit {
            return false;
        }
    }
    false
}

fn debug() -> Result<(), Box<dyn std::error::Error>> {
    const FRAME_TIME: time::Duration = time::Duration::from_micros(16666);

    let mut game = {
        let config = GameConfig {
            params: GameParams {
                // gravity: 0.0167,
                gravity: 0.0,
                are: 0,
                lock_delay: 60 * 60 * 60 * 24,
                line_clear_delay: 0,
                ..GameParams::default()
            },
            logic: WorldRuleLogic::default(),
        };
        let data = GameData::new(
            Playfield {
                visible_rows: 20,
                grid: PieceGrid::new(10, 40, vec![]),
            },
            None,
            None,
            generate_pieces(),
            &config.params,
        );
        Game::new(config, data)
    };

    let (mut terminal, mut stdin) = init_terminal()?;

    // lines, tspin, remaining frames
    let mut line_clear = (renderer::LineClearInfo::default(), 0);

    loop {
        let frame_started_at = time::Instant::now();

        if game.data().next_pieces.len() <= Piece::num() {
            let mut ps = generate_pieces();
            game.append_next_pieces(&mut ps);
        }

        let mut input = Input::default();
        if let Some(Ok(item)) = stdin.next() {
            if let Ok(ev) = termion::event::parse_event(item, &mut stdin) {
                match ev {
                    Event::Key(key) => match key {
                        Key::Char('q') => break,
                        Key::Char('z') => input |= Input::ROTATE_CCW,
                        Key::Char('x') => input |= Input::ROTATE_CW,
                        Key::Char('c') | Key::Char(' ') => input |= Input::HOLD,
                        Key::Char('s') => input |= Input::FIRM_DROP,
                        Key::Right => input |= Input::MOVE_RIGHT,
                        Key::Left => input |= Input::MOVE_LEFT,
                        Key::Up => input |= Input::HARD_DROP,
                        Key::Down => input |= Input::SOFT_DROP,
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                break;
            }
        }
        game.update(input);

        for event in &game.data().events {
            match event {
                GameEvent::LineCleared(n, t) => {
                    line_clear.0.n = *n;
                    line_clear.0.tspin = *t;
                    line_clear.1 = 60 * 2;
                    break;
                }
                _ => {}
            }
        }

        terminal.draw(|mut f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(10), Constraint::Percentage(90)].as_ref())
                .split(size);
            Block::default()
                .style(Style::default().bg(Color::Black))
                .render(&mut f, size);
            // Left pane
            renderer::render(
                &mut f,
                game.data(),
                if line_clear.1 > 0 {
                    line_clear.1 -= 1;
                    Some(line_clear.0.clone())
                } else {
                    None
                },
                (0, 0),
            );
            // Right pane
            {
                let text = [Text::raw(format!("{:?}", game))];
                Paragraph::new(text.iter())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .wrap(true)
                    .render(&mut f, chunks[1]);
            }
        })?;

        let dt = time::Instant::now() - frame_started_at;
        if dt < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - dt);
        }
    }

    Ok(())
}

fn research() -> Result<(), Box<dyn std::error::Error>> {
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
        let data = GameData::new(
            Playfield {
                visible_rows: 20,
                grid: PieceGrid::new(10, 40, vec![]),
            },
            None,
            None,
            generate_pieces(),
            &config.params,
        );
        Game::new(config, data)
    };

    let (mut terminal, mut stdin) = init_terminal()?;

    let mut line_clear_info: Option<renderer::LineClearInfo> = None;

    loop {
        if game.data().next_pieces.len() <= Piece::num() {
            let mut ps = generate_pieces();
            game.append_next_pieces(&mut ps);
        }

        if !update_util(&mut game, GameStateId::Play, 1000) {
            // TODO: error
            break;
        }

        let mut input = Input::default();
        if let Some(Ok(item)) = stdin.next() {
            if let Ok(ev) = termion::event::parse_event(item, &mut stdin) {
                match ev {
                    Event::Key(key) => match key {
                        Key::Char('q') => break,
                        Key::Char('z') => input |= Input::ROTATE_CCW,
                        Key::Char('x') => input |= Input::ROTATE_CW,
                        Key::Char('c') | Key::Char(' ') => input |= Input::HOLD,
                        Key::Char('s') => input |= Input::FIRM_DROP,
                        Key::Right => input |= Input::MOVE_RIGHT,
                        Key::Left => input |= Input::MOVE_LEFT,
                        Key::Up => input |= Input::HARD_DROP,
                        Key::Down => input |= Input::SOFT_DROP,
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                break;
            }
        }
        if !input.is_empty() {
            game.update(input);
            game.update(Input::default());
        }

        for event in &game.data().events {
            match event {
                GameEvent::LineCleared(n, t) => {
                    line_clear_info = Some(renderer::LineClearInfo { n: *n, tspin: *t })
                }
                _ => {}
            }
        }

        terminal.draw(|mut f| {
            let size = f.size();
            Block::default()
                .style(Style::default().bg(Color::Black))
                .render(&mut f, size);
            renderer::render(&mut f, game.data(), line_clear_info, (0, 0));
        })?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("mino_cli")
        .subcommand(SubCommand::with_name("play"))
        .subcommand(SubCommand::with_name("research"))
        .get_matches();

    if let Some(_matches) = matches.subcommand_matches("play") {
        return debug();
    }

    if let Some(_matches) = matches.subcommand_matches("research") {
        return research();
    }

    Ok(())
}
