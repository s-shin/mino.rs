use super::common::{FallingPiece, Game, GameLogic, GameParams, GameState, Playfield, Rotation};
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

impl super::common::Piece for Piece {
    type Piece = Piece;
    fn grid(&self, rotation: Rotation) -> grid::Grid<Cell<Self::Piece>> {
        //
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub type Cell = super::common::Cell<Piece>;
pub type Grid = grid::Grid<Cell>;

pub struct PieceDefinition {
    grids: Vec<Grid>,
}

fn gen_piece_definitions() -> Vec<PieceDefinition> {
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

pub struct WorldRuleLogic {}

impl GameLogic<Piece> for WorldRuleLogic {
    fn piece_grid(&self, piece: Piece, rotation: Rotation) -> &Grid {
        &PIECE_DEFINITIONS[piece as usize].grids[rotation as usize]
    }
    fn spawn_piece(
        &self,
        piece: Option<Piece>,
        playfield: &Playfield<Piece>,
    ) -> FallingPiece<Piece> {
        let p = if let Some(pp) = piece {
            pp
        } else {
            Piece::I // TODO
        };
        FallingPiece {
            piece: p,
            x: ((playfield.grid.num_cols() - self.piece_grid(p, Rotation::default()).num_cols())
                as i32)
                / 2,
            y: (playfield.visible_rows as i32) - 1,
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
