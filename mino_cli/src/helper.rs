use mino_core::tetro::Piece;
use rand::seq::SliceRandom;
use std::collections::VecDeque;

pub mod full_screen;

pub fn generate_pieces() -> VecDeque<Piece> {
    let mut rng = rand::thread_rng();
    let mut ps = Piece::slice().clone();
    ps.shuffle(&mut rng);
    ps.to_vec().into()
}

pub fn tspin_num_to_en_str_long(n: u8) -> &'static str {
    match n {
        0 => "Zero",
        1 => "Single",
        2 => "Double",
        3 => "Triple",
        _ => "",
    }
}

// pub fn update_util(
//     game: &mut Game<Piece, WorldRuleLogic>,
//     state_id: GameStateId,
//     limit: i32,
// ) -> bool {
//     for i in 0.. {
//         if game.state_id() == state_id {
//             return true;
//         }
//         game.update(Input::default());
//         if limit > 0 && i > limit {
//             return false;
//         }
//     }
//     false
// }
