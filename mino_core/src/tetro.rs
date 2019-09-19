// use super::common::{Input, InputFrameCounter, Rotation};
// use lazy_static::lazy_static;
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

impl super::common::Piece for Piece {}

impl fmt::Display for Piece {
  fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "{:?}", self)
  }
}

pub type Cell = super::common::Cell<Piece>;
pub type Grid = super::common::grid::Grid<Cell>;
pub type GridFormatter = super::common::grid::GridFormatter<Cell>;
// pub type Piece = super::common::Piece<Block>;
// pub type FallingPiece = super::common::FallingPiece<Block>;

// lazy_static! {
//   pub static ref PIECE_I: Piece = Piece {
//     shape: Grid::new(1, 1, vec![]),
//     rotation: Rotation::default(),
//   };
// }
