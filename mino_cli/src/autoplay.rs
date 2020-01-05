use super::helper;
use mino_core::common::{Game, GameConfig, GameData, GameParams, Input, PieceGrid, Playfield};
use mino_core::tetro::{Piece, WorldRuleLogic};
use std::error::Error;
use termion::event::{Event, Key};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Paragraph, Text, Widget};

// trait Player<P: mino_core::common::Piece, L> {
//     fn decide_moves(&mut self, game: &Game<P, L>) -> Result<Vec<Input>, Box<dyn Error>>;
// }

pub fn run() -> Result<(), Box<dyn Error>> {
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
            helper::generate_pieces(),
            &config.params,
        );
        Game::new(config, data)
    };

    let (mut terminal, mut stdin) = helper::full_screen::init_terminal()?;

    loop {
        if game.data().next_pieces.len() <= Piece::num() {
            let mut ps = helper::generate_pieces();
            game.append_next_pieces(&mut ps);
        }

        if let Some(Ok(item)) = stdin.next() {
            if let Ok(ev) = termion::event::parse_event(item, &mut stdin) {
                match ev {
                    Event::Key(key) => match key {
                        Key::Char('q') => break,
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                break;
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
            helper::full_screen::render(&mut f, game.data(), None, (0, 0));
            // Right pane
            {
                let text = [Text::raw("INFO")];
                Paragraph::new(text.iter())
                    .style(Style::default().fg(Color::White).bg(Color::Black))
                    .wrap(true)
                    .render(&mut f, chunks[1]);
            }
        })?;

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}
