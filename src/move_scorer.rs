use std::cmp::max;
use crate::board::{Board, KNIGHT};
use crate::eval::PIECE_VALUES;
use crate::move_info::SQUARES;
use crate::movegen::{get_all_attackers, NO_SQUARES};
use crate::moves::{KillerMoves, Move, MoveType};



// pub struct MoveScorer;
// 
// impl MoveScorer {
//     pub fn score_move(
//         m: Move,
//         board: &Board,
//         best_move: Option<Move>,
//         _km: &KillerMoves,
//         _depth: usize
//     ) -> i32 {
//         if Some(m) == best_move {
//             return BEST_MOVE_SCORE;
//         } //else if let Some(score) = self.km.get_move(depth) {
//         //     return score
//         // }
//             // todo see if killer moves do anything (add it back in later)
//         // let mut score = 0;
//         match m.move_type() {
//             MoveType::Quiet | MoveType::Double | MoveType::WKingSide | MoveType::BKingSide |
//             MoveType::WQueenSide | MoveType::BQueenSide | MoveType::Promo | MoveType::Ep => {
//                 // let piece = m.piece() as usize;
//                 // score = PST[piece][m.to() as usize] as i32 * PIECE_VALUES[piece];
//                 0
//             }
//             MoveType::Cap | MoveType::NPromoCap | MoveType::RPromoCap | MoveType::BPromoCap |
//             MoveType::QPromoCap => MoveScorer::see(board, m) + CAP_SCORE_OFFSET
//         }
//     }
// 
//     fn see_get_least_valuable(board: &Board, attackers: u64, colour: usize) -> (usize, u64) {
//         let piece_iter = board.pieces.iter()
//             .enumerate()
//             .skip(colour)
//             .step_by(2);
// 
//         for (piece, pieces) in piece_iter {
//             let p_in_attackers = *pieces & attackers;
//             if p_in_attackers > 0 {
//                 return (piece, p_in_attackers & p_in_attackers.wrapping_neg());
//             }
//         }
// 
//         (12, NO_SQUARES)
//     }
// 
//     fn see(board: &Board, m: Move) -> i32 {
//         // trying to understand the https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
// 
//         let mut gain: [i32; 32] = [0;32];
//         let mut depth = 0;
// 
//         let (from, to, mut piece, xpiece, _) = m.all();
// 
//         // froms is a bb of the next possible piece to move that attacks the to square
//         let mut from_piece = SQUARES[from];
//         let mut occ = board.util[2];
//         let mut attackers = get_all_attackers(board,to);
// 
//         // can_xray is just all pieces that arent knights
//         // as there is no sliding piece that can be behind a knight that could attack the target
//         let can_xray = occ ^ board.pieces[KNIGHT] ^ board.pieces[KNIGHT+1];
// 
//         gain[depth] = PIECE_VALUES[xpiece];
// 
//         while from_piece > 0 {
//             depth += 1;
// 
//             // add this score into the and cut off if it cannot increase the score
//             gain[depth] = PIECE_VALUES[piece] - gain[depth-1];
//             if max(-gain[depth-1], gain[depth]) < 0 { break; }
// 
//             // remove this attacker
//             attackers ^= from_piece;
//             occ ^= from_piece;
// 
//             // recheck if there are any sliding pieces behind this attacker
//             if from_piece & can_xray > 0 {
//                 attackers |= occ & get_all_attackers(board, to);
//             }
// 
//             (piece, from_piece) = MoveScorer::see_get_least_valuable(
//                 board,
//                 attackers,
//                 board.colour_to_move ^ (depth & 1)
//             );
//         }
// 
//         // iterate over all the stored gain values to find the max - negamax style
//         for i in (1..depth).rev() {
//             gain[i-1] = -max(-gain[i-1], gain[i]);
//         }
// 
//         gain[0]
//     }
// }

// #[test]
// fn see_evaluation() {
//     let board1 = Board::new_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -");
//     let ml = MoveList::new( 100, 100);
//
//     let rxe5 = Move::new(4, 36, ROOK as u32, 1, MoveType::Cap);
//     assert_eq!(ml.see(&board1, rxe5), PIECE_VALUES[PAWN]);
//
//     let board2 = Board::new_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
//     let ml = MoveList::new(100, 100);
//
//     let nxe5 = Move::new(19, 36, KNIGHT as u32, 1, MoveType::Cap);
//     assert_eq!(ml.see(&board2, nxe5), -225)
// }
//
// #[test]
// fn move_ordering() {
//     let km = KillerMoves::new();
//
//     let board = Board::new_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
//     let ml_scored = MoveList::all_scored(&board, false,None, &km, 0);
//     let scored: Vec<Move> = ml_scored.collect();
//     let unscored: Vec<Move> = MoveList::all(&board, false).moves;
//
//     assert_eq!(scored.len(), unscored.len())
// }
