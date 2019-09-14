extern crate mino_core;

use mino_core::common::GridFormatterOptions;
use mino_core::tetro::*;

fn main() {
    let mut field = Grid::new(10, 40, vec![]);
    for x in 1..9 {
        field.set_cell(x, 0, Cell::Garbage);
    }
    field.set_cell(0, 1, Cell::Block(Block::O));
    field.set_cell(1, 1, Cell::Block(Block::O));
    field.set_cell(0, 2, Cell::Block(Block::O));
    field.set_cell(1, 2, Cell::Block(Block::O));
    let _cell = field.get_cell(0, 0);
    // println!("Field: {:?}", field);
    println!(
        "{}",
        GridFormatter {
            grid: field,
            opts: GridFormatterOptions {
                str_begin_of_line: "|",
                str_end_of_line: "|",
                range_y: Some(0..20),
                ..GridFormatterOptions::default()
            }
        }
    )
}
