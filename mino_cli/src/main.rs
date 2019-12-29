extern crate grid;
extern crate mino_core;
extern crate rand;
extern crate termion;
extern crate tui;
use grid::IsEmpty;
use mino_core::common::{
    new_input_manager, Cell, FallingPiece, Game, GameConfig, GameParams, GameStateData, Input,
    Playfield,
};
use mino_core::tetro::{Piece, PieceGrid, WorldRuleLogic};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use std::time;
use termion::event::{Event, Key};
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Paragraph, Text, Widget};
use tui::Terminal;

struct ViewData {
    ghost_piece: Option<FallingPiece<Piece>>,
}

impl ViewData {
    fn new(data: &GameStateData<Piece>) -> Self {
        Self {
            ghost_piece: if let Some(fp) = data.falling_piece {
                let n = fp.droppable_rows(&data.playfield);
                let mut gp = fp.clone();
                gp.y -= n as i32;
                Some(gp)
            } else {
                None
            },
        }
    }

    fn get_cell(&self, data: &GameStateData<Piece>, x: usize, y: usize) -> Cell<Piece> {
        let pf = &data.playfield;
        if let Some(fp) = data.falling_piece {
            let x = x as i32 - fp.x;
            let y = y as i32 - fp.y;
            if fp.grid().is_valid_cell_index(x as usize, y as usize) {
                let c = fp.grid().cell(x as usize, y as usize);
                if !c.is_empty() {
                    return c;
                }
            }
        }
        if let Some(gp) = self.ghost_piece {
            let x = x as i32 - gp.x;
            let y = y as i32 - gp.y;
            if gp.grid().is_valid_cell_index(x as usize, y as usize) {
                if let Cell::Block(p) = gp.grid().cell(x as usize, y as usize) {
                    return Cell::Ghost(p);
                }
            }
        }
        pf.grid.cell(x, y)
    }
}

fn generate_pieces() -> VecDeque<Piece> {
    let mut rng = rand::thread_rng();
    let mut ps = Piece::slice().clone();
    ps.shuffle(&mut rng);
    ps.to_vec().into()
}

fn format_cell(cell: Cell<Piece>) -> (String, Color) {
    match cell {
        Cell::Block(p) => (
            format!("{}", p),
            match p {
                Piece::I => Color::Cyan,
                Piece::O => Color::Yellow,
                Piece::T => Color::Magenta,
                Piece::J => Color::Blue,
                Piece::L => Color::LightRed,
                Piece::S => Color::Green,
                Piece::Z => Color::Red,
            },
        ),
        Cell::Ghost(p) => (format!("{}", p), Color::DarkGray),
        _ => (" ".into(), Color::Black),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const FRAME_TIME: time::Duration = time::Duration::from_micros(16666);

    let mut game = {
        let config = GameConfig {
            params: GameParams {
                // gravity: 0.0167,
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
            next_pieces: generate_pieces(),
            input_mgr: new_input_manager(config.params.das, config.params.arr),
        };
        Game::new(config, data)
    };

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let mut stdin = termion::async_stdin().bytes();

    for frame in 0..6000 {
        let frame_started_at = time::Instant::now();

        if game.data.next_pieces.len() <= Piece::num() {
            let mut ps = generate_pieces();
            game.data.next_pieces.append(&mut ps);
        }

        let mut input = Input::default();
        if let Some(Ok(item)) = stdin.next() {
            if let Ok(ev) = termion::event::parse_event(item, &mut stdin) {
                match ev {
                    Event::Key(key) => match key {
                        Key::Char('q') => break,
                        Key::Char('z') => input |= Input::ROTATE_CCW,
                        Key::Char('x') => input |= Input::ROTATE_CW,
                        Key::Char(' ') => input |= Input::HOLD,
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

        terminal.draw(|mut f| {
            let size = f.size();
            Block::default()
                .style(Style::default().bg(Color::Black))
                .render(&mut f, size);
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(10), Constraint::Percentage(90)].as_ref())
                .split(size);
            let mut top = 0;
            {
                let mut text = vec![Text::raw("HOLD:")];
                let t = if let Some(p) = game.data.hold_piece {
                    format_cell(Cell::Block(p))
                } else {
                    ("".into(), Color::Black)
                };
                text.push(Text::styled(t.0, Style::default().fg(t.1)));
                Paragraph::new(text.iter()).render(&mut f, Rect::new(0, top, 10, 1));
                top += 1;
            }
            {
                let mut text = vec![Text::raw("NEXT:")];
                let mut ts: Vec<(String, Color)> = Vec::new();
                for i in 0..5 {
                    let t = if let Some(p) = game.data.next_pieces.get(i) {
                        format_cell(Cell::Block(*p))
                    } else {
                        ("".into(), Color::Black)
                    };
                    ts.push(t);
                }
                for t in ts {
                    text.push(Text::styled(t.0, Style::default().fg(t.1)));
                }
                Paragraph::new(text.iter()).render(&mut f, Rect::new(0, top, 10, 1));
                top += 1;
            }
            let pf = &game.data.playfield;
            let vd = ViewData::new(&game.data);
            for y in 0..pf.visible_rows {
                for x in 0..pf.grid.num_cols() {
                    let t = format_cell(vd.get_cell(&game.data, x, y));
                    let text = [Text::styled(t.0, Style::default().fg(t.1))];
                    Paragraph::new(text.iter()).render(
                        &mut f,
                        Rect::new(x as u16, top + (pf.visible_rows - 1 - y) as u16, 1, 1),
                    );
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

        let dt = time::Instant::now() - frame_started_at;
        if dt < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - dt);
        }
    }

    Ok(())
}
