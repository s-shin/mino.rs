// use super::common::{Input, InputFrameCounter, Rotation};
// use lazy_static::lazy_static;
use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum Block {
  I,
  T,
  O,
  S,
  Z,
  J,
  L,
}

impl fmt::Display for Block {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{:?}", self)
  }
}

pub type Cell = super::common::Cell<Block>;
pub type Grid = super::common::Grid<Cell>;
pub type GridFormatter = super::common::GridFormatter<Cell>;
// pub type Piece = super::common::Piece<Block>;
// pub type FallingPiece = super::common::FallingPiece<Block>;

// lazy_static! {
//   pub static ref PIECE_I: Piece = Piece {
//     shape: Grid::new(1, 1, vec![]),
//     rotation: Rotation::default(),
//   };
// }
