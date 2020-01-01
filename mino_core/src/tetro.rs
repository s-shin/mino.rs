use super::common::{FallingPiece, GameLogic, Piece as PieceTrait, Playfield, Rotation, TSpin};
use grid::IsEmpty;
use lazy_static::lazy_static;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Piece {
    I,
    T,
    O,
    S,
    Z,
    J,
    L,
}

impl Piece {
    pub fn num() -> usize {
        7
    }
    pub fn slice() -> &'static [Piece; 7] {
        static PIECES: [Piece; 7] = [
            Piece::I,
            Piece::T,
            Piece::O,
            Piece::S,
            Piece::Z,
            Piece::J,
            Piece::L,
        ];
        &PIECES
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub type PieceGrid = super::common::PieceGrid<Piece>;

pub struct PieceDefinition {
    grids: Vec<PieceGrid>,
}

fn gen_piece_definitions() -> Vec<PieceDefinition> {
    use grid::Grid;
    type Cell = super::common::Cell<Piece>;

    let e = Cell::Empty;
    let i = Cell::Block(Piece::I);
    let t = Cell::Block(Piece::T);
    let o = Cell::Block(Piece::O);
    let s = Cell::Block(Piece::S);
    let z = Cell::Block(Piece::Z);
    let j = Cell::Block(Piece::J);
    let l = Cell::Block(Piece::L);

    let mut grid_i = Grid::new(
        5,
        5,
        vec![
            e, e, e, e, e, //
            e, e, e, e, e, //
            e, i, i, i, i, //
            e, e, e, e, e, //
            e, e, e, e, e, //
        ],
    );
    grid_i.reverse_rows();

    let mut grid_t = Grid::new(
        3,
        3,
        vec![
            e, t, e, //
            t, t, t, //
            e, e, e, //
        ],
    );
    grid_t.reverse_rows();

    let mut grid_o = Grid::new(
        3,
        3,
        vec![
            e, o, o, //
            e, o, o, //
            e, e, e, //
        ],
    );
    grid_o.reverse_rows();

    let mut grid_s = Grid::new(
        3,
        3,
        vec![
            e, s, s, //
            s, s, e, //
            e, e, e, //
        ],
    );
    grid_s.reverse_rows();

    let mut grid_z = Grid::new(
        3,
        3,
        vec![
            z, z, e, //
            e, z, z, //
            e, e, e, //
        ],
    );
    grid_z.reverse_rows();

    let mut grid_j = Grid::new(
        3,
        3,
        vec![
            j, e, e, //
            j, j, j, //
            e, e, e, //
        ],
    );
    grid_j.reverse_rows();

    let mut grid_l = Grid::new(
        3,
        3,
        vec![
            e, e, l, //
            l, l, l, //
            e, e, e, //
        ],
    );
    grid_l.reverse_rows();

    vec![
        // I
        PieceDefinition {
            grids: vec![
                grid_i.clone(),
                grid_i.rotate1(),
                grid_i.rotate2(),
                grid_i.rotate3(),
            ],
        },
        // T
        PieceDefinition {
            grids: vec![
                grid_t.clone(),
                grid_t.rotate1(),
                grid_t.rotate2(),
                grid_t.rotate3(),
            ],
        },
        // O
        PieceDefinition {
            grids: vec![
                grid_o.clone(),
                grid_o.rotate1(),
                grid_o.rotate2(),
                grid_o.rotate3(),
            ],
        },
        // S
        PieceDefinition {
            grids: vec![
                grid_s.clone(),
                grid_s.rotate1(),
                grid_s.rotate2(),
                grid_s.rotate3(),
            ],
        },
        // Z
        PieceDefinition {
            grids: vec![
                grid_z.clone(),
                grid_z.rotate1(),
                grid_z.rotate2(),
                grid_z.rotate3(),
            ],
        },
        // J
        PieceDefinition {
            grids: vec![
                grid_j.clone(),
                grid_j.rotate1(),
                grid_j.rotate2(),
                grid_j.rotate3(),
            ],
        },
        // L
        PieceDefinition {
            grids: vec![
                grid_l.clone(),
                grid_l.rotate1(),
                grid_l.rotate2(),
                grid_l.rotate3(),
            ],
        },
    ]
}

lazy_static! {
    static ref PIECE_DEFINITIONS: Vec<PieceDefinition> = gen_piece_definitions();
    static ref OFFSET_DATA_I: [Vec<(i32, i32)>; 4] = [
        vec![(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        vec![(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        vec![(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        vec![(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
    ];
    static ref OFFSET_DATA_O: [Vec<(i32, i32)>; 4] =
        [vec![(0, 0)], vec![(0, -1)], vec![(-1, -1)], vec![(-1, 0)]];
    static ref OFFSET_DATA_JLSTZ: [Vec<(i32, i32)>; 4] = [
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    ];
}

impl PieceTrait for Piece {
    fn grid(&self, rotation: Rotation) -> &PieceGrid {
        &PIECE_DEFINITIONS[*self as usize].grids[rotation as usize]
    }
}

//---

#[derive(Debug, Default)]
pub struct WorldRuleLogic {}

impl GameLogic<Piece> for WorldRuleLogic {
    fn spawn_piece(&self, piece: Piece, playfield: &Playfield<Piece>) -> FallingPiece<Piece> {
        let g = piece.grid(Rotation::default());
        let top_pad = piece.grid_top_padding(Rotation::default());
        let mut fp = FallingPiece {
            piece: piece,
            x: ((playfield.grid.num_cols() - g.num_cols()) as i32) / 2,
            y: (playfield.visible_rows as i32) - (g.num_rows() - top_pad) as i32 + 1,
            rotation: Rotation::default(),
        };
        if !fp.can_put_onto(playfield) {
            fp.y += 1;
        }
        fp
    }
    /// References:
    /// * https://harddrop.com/wiki/SRS#How_Guideline_SRS_Really_Works
    /// * https://harddrop.com/wiki/T-Spin
    fn rotate(
        &self,
        cw: bool,
        falling_piece: &FallingPiece<Piece>,
        playfield: &Playfield<Piece>,
    ) -> Option<(FallingPiece<Piece>, TSpin)> {
        let mut fp = falling_piece.clone();
        fp.rotation = if cw {
            fp.rotation.cw()
        } else {
            fp.rotation.ccw()
        };
        let offset_data = &match fp.piece {
            Piece::I => &*OFFSET_DATA_I,
            Piece::O => &*OFFSET_DATA_O,
            _ => &*OFFSET_DATA_JLSTZ,
        };
        let offsets1 = &offset_data[falling_piece.rotation as usize];
        let offsets2 = &offset_data[fp.rotation as usize];
        for i in 0..offsets1.len() {
            let mut fp = fp.clone();
            fp.x += offsets1[i].0 - offsets2[i].0;
            fp.y += offsets1[i].1 - offsets2[i].1;
            if fp.can_put_onto(playfield) {
                let tspin = if fp.piece == Piece::T {
                    // check corder
                    let mut n = 0;
                    let center = (fp.x + 1, fp.y + 1);
                    for dy in &[-1, 1] {
                        for dx in &[-1, 1] {
                            let x = center.0 + dx;
                            let y = center.1 + dy;
                            // outside or block
                            if (x < 0 || y < 0)
                                || !playfield.grid.is_valid_cell_index(x as usize, y as usize)
                                || !playfield.grid.cell(x as usize, y as usize).is_empty()
                            {
                                n += 1;
                            }
                        }
                    }
                    if n >= 3 {
                        // Check cell behinde the T piece.
                        let d = match fp.rotation {
                            Rotation::Cw0 => (0, -1),
                            Rotation::Cw90 => (-1, 0),
                            Rotation::Cw180 => (0, 1),
                            Rotation::Cw270 => (1, 0),
                        };
                        let x = center.0 + d.0;
                        let y = center.1 + d.1;
                        // outside or block
                        if (x < 0 || y < 0)
                            || !playfield.grid.is_valid_cell_index(x as usize, y as usize)
                            || !playfield.grid.cell(x as usize, y as usize).is_empty()
                        {
                            if n == 4 {
                                TSpin::Normal // T-Spin triple variants
                            } else {
                                TSpin::Mini
                            }
                        } else {
                            TSpin::Normal
                        }
                    } else {
                        TSpin::None
                    }
                } else {
                    TSpin::None
                };
                return Some((fp, tspin));
            }
        }
        None
    }
}
