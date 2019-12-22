use super::common::{FallingPiece, GameLogic, Piece as PieceTrait, Playfield, Rotation};
use lazy_static::lazy_static;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Piece {
    I,
    T,
    O,
    S,
    Z,
    J,
    L,
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
        4,
        4,
        vec![
            e, e, e, e, //
            i, i, i, i, //
            e, e, e, e, //
            e, e, e, e, //
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
        4,
        4,
        vec![
            e, e, e, e, //
            e, o, o, e, //
            e, o, o, e, //
            e, e, e, e, //
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
    pub static ref PIECE_DEFINITIONS: Vec<PieceDefinition> = gen_piece_definitions();
}

impl PieceTrait for Piece {
    fn grid(&self, rotation: Rotation) -> &PieceGrid {
        &PIECE_DEFINITIONS[*self as usize].grids[rotation as usize]
    }
}

//---

#[derive(Debug)]
pub struct WorldRuleLogic {}

impl GameLogic<Piece> for WorldRuleLogic {
    fn spawn_piece(
        &self,
        piece: Piece,
        num_cols: usize,
        _num_rows: usize,
        num_visible_rows: usize,
    ) -> FallingPiece<Piece> {
        let pg = piece.grid(Rotation::default());
        FallingPiece {
            piece: piece,
            x: ((num_cols - pg.num_cols()) as i32) / 2,
            // FIXME
            y: (num_visible_rows as i32) - 1 - (pg.num_rows() / 2) as i32,
            rotation: Rotation::default(),
        }
    }
    fn rotate(
        &self,
        _cw: bool,
        falling_piece: &FallingPiece<Piece>,
        _playfield: &Playfield<Piece>,
    ) -> Option<FallingPiece<Piece>> {
        // TODO
        Option::Some(*falling_piece)
    }
}
