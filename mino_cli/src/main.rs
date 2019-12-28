extern crate grid;
extern crate mino_core;
extern crate termion;
extern crate tokio;
extern crate tui;
use mino_core::common::{
    new_input_manager, Game, GameConfig, GameParams, GameStateData, Input, Playfield,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use termion::event::{Event, Key};
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Paragraph, Text, Widget};
use tui::Terminal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = {
        let config = GameConfig {
            params: GameParams {
                gravity: 0.2,
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
            next_pieces: VecDeque::from(vec![
                Piece::J,
                Piece::O,
                Piece::I,
                Piece::L,
                Piece::T,
                Piece::S,
                Piece::Z,
            ]),
            input_mgr: new_input_manager(config.params.das, config.params.arr),
        };
        Game::new(config, data)
    };

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let mut stdin = termion::async_stdin().bytes();
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));

    for frame in 0..6000 {
        let mut input = Input::default();
        if let Some(Ok(item)) = stdin.next() {
            if let Ok(ev) = termion::event::parse_event(item, &mut stdin) {
                match ev {
                    Event::Key(key) => match key {
                        Key::Char('q') => break,
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

        let mut pf = game.data.playfield.clone();
        if let Some(fp) = game.data.falling_piece {
            fp.put_onto(&mut pf);
        }

        terminal.draw(|mut f| {
            let size = f.size();
            Block::default()
                .style(Style::default().bg(Color::Black))
                .render(&mut f, size);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(10), Constraint::Percentage(90)].as_ref())
                .split(size);
            for y in 0..pf.visible_rows {
                for x in 0..pf.grid.num_cols() {
                    let rect = Rect::new(x as u16, (pf.visible_rows - y) as u16, 1, 1);
                    let s = format!("{}", pf.grid.cell(x, y));
                    let text = [Text::styled(&s, Style::default().fg(Color::Yellow))];
                    Paragraph::new(text.iter()).render(&mut f, rect);
                }
            }
            {
                let text = [Text::raw(format!("{}\n{:?}", frame, game))];
                Paragraph::new(text.iter())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .wrap(true)
                    .render(&mut f, chunks[1]);
            }
        })?;

        interval.tick().await;
    }

    Ok(())
}
