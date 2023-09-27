use crate::movegen::moves::Move;

mod bishop_iter;
mod black_pawn_iter;
mod king_iter;
mod knight_iter;
pub mod movegen_iter;
mod queen_iter;
mod rook_iter;
mod white_pawn_iter;

trait MovegenIterator: Iterator {
    type MovegenIter;
}
