use grid::IsEmpty;
use mino_core::common::{Cell, FallingPiece, GameData, TSpin};
use mino_core::tetro::Piece;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Paragraph, Text, Widget};

struct ViewDataBuilder {
    ghost_piece: Option<FallingPiece<Piece>>,
}

impl ViewDataBuilder {
    fn new(data: &GameData<Piece>) -> Self {
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

    fn get_cell(&self, data: &GameData<Piece>, x: usize, y: usize) -> Cell<Piece> {
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

#[derive(Default)]
pub struct LineClearInfo {
    pub n: usize,
    pub tspin: TSpin,
}

pub fn render<B>(
    f: &mut tui::Frame<B>,
    data: &GameData<Piece>,
    line_clear_info: Option<&LineClearInfo>,
    pos: (u16, u16),
) where
    B: tui::backend::Backend,
{
    let mut top = pos.1;
    {
        let mut text = vec![Text::raw("HOLD:")];
        let t = if let Some(p) = data.hold_piece {
            format_cell(Cell::Block(p))
        } else {
            ("     ".into(), Color::Black)
        };
        text.push(Text::styled(t.0, Style::default().fg(Color::Black).bg(t.1)));
        Paragraph::new(text.iter()).render(f, Rect::new(pos.0, top, 10, 1));
        top += 1;
    }
    {
        let mut text = vec![Text::raw("NEXT:")];
        let mut ts: Vec<(String, Color)> = Vec::new();
        for i in 0..5 {
            let t = if let Some(p) = data.next_pieces.get(i) {
                format_cell(Cell::Block(*p))
            } else {
                ("     ".into(), Color::Black)
            };
            ts.push(t);
        }
        for t in ts {
            text.push(Text::styled(t.0, Style::default().fg(Color::Black).bg(t.1)));
        }
        Paragraph::new(text.iter()).render(f, Rect::new(pos.0, top, 10, 1));
        top += 1;
    }
    {
        let pf = &data.playfield;
        let vdb = ViewDataBuilder::new(&data);
        for y in 0..pf.visible_rows {
            for x in 0..pf.grid.num_cols() {
                let t = format_cell(vdb.get_cell(&data, x, y));
                let text = [Text::styled(t.0, Style::default().fg(Color::Black).bg(t.1))];
                Paragraph::new(text.iter()).render(
                    f,
                    Rect::new(
                        pos.0 + x as u16,
                        top + (pf.visible_rows - 1 - y) as u16,
                        1,
                        1,
                    ),
                );
            }
        }
        top += 20;
    }
    {
        let t = "=".repeat(10);
        let text = [Text::raw(&t)];
        Paragraph::new(text.iter()).render(f, Rect::new(0, top, 10, 1));
        top += 1;
    }
    {
        let t = if let Some(info) = line_clear_info {
            match info.tspin {
                TSpin::None => format!("{} Lines!", info.n),
                TSpin::Mini => format!("TSM{}!", "ZSTD".chars().nth(info.n).unwrap()),
                TSpin::Normal => format!("TS{}!", "ZSTD".chars().nth(info.n).unwrap()),
            }
        } else {
            " ".repeat(10)
        };
        let text = [Text::raw(&t)];
        Paragraph::new(text.iter()).render(f, Rect::new(0, top, 10, 1));
    }
}
