use crate::board::board::{Board, BLACK, WHITE};
use crate::movegen::move_info::SQUARES;
use crate::movegen::movegen::{get_all_attackers, NO_SQUARES};
use crate::movegen::moves::{Move, MoveType};
use crate::search::eval::PIECE_VALUES;
use crate::search::search::Searcher;
use std::cmp::max;

const BEST_MOVE_SCORE: i32 = i32::MAX;
const CAP_SCORE_OFFSET: i32 = 10000;
const KILLER_OFFSET: i32 = 10000;

pub fn score_move(b: &Board, s: &impl Searcher, depth: usize, m: Move) -> i32 {
    if s.get_tt_best_move(b.hash()) == Some(m) {
        return BEST_MOVE_SCORE;
    }

    match m.move_type() {
        MoveType::Quiet
        | MoveType::Double
        | MoveType::WKingSide
        | MoveType::BKingSide
        | MoveType::WQueenSide
        | MoveType::BQueenSide
        | MoveType::Promo
        | MoveType::Ep => score_quiet(b, s, depth, m),

        MoveType::Cap
        | MoveType::NPromoCap
        | MoveType::RPromoCap
        | MoveType::BPromoCap
        | MoveType::QPromoCap => see(b, m) + CAP_SCORE_OFFSET,
    }
}

fn score_quiet(b: &Board, s: &impl Searcher, depth: usize, m: Move) -> i32 {
    let km = s.km_get(depth);

    if km[0].filter(|killer| *killer == m).is_some() {
        return KILLER_OFFSET + 1;
    }

    if km[1].filter(|killer| *killer == m).is_some() {
        return KILLER_OFFSET;
    }

    s.get_hh_score(b.ctm(), m.from() as usize, m.to() as usize) as i32
}

fn see(b: &Board, m: Move) -> i32 {
    // trying to understand the https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
    let mut gain: [i32; 32] = [0; 32];
    let mut depth = 0;
    let (from, to, mut piece, xpiece, _) = m.all();

    // froms is a bb of the next possible piece to move that attacks the to square
    let mut from_piece = SQUARES[from];
    let mut occ = b.all_occ();
    let mut attackers = get_all_attackers(b, to);

    // can_xray is just all pieces that arent knights
    // as there is no sliding piece that can be behind a knight that could attack the target
    let can_xray = occ ^ b.knights(WHITE) ^ b.knights(BLACK);

    gain[depth] = PIECE_VALUES[xpiece];
    while from_piece > 0 {
        depth += 1;

        // add this score into the and cut off if it cannot increase the score
        gain[depth] = PIECE_VALUES[piece] - gain[depth - 1];
        if max(-gain[depth - 1], gain[depth]) < 0 {
            break;
        }

        // remove this attacker
        attackers ^= from_piece;
        occ ^= from_piece;

        // recheck if there are any sliding pieces behind this attacker
        if from_piece & can_xray > 0 {
            attackers |= occ & get_all_attackers(b, to);
        }

        (piece, from_piece) = see_get_least_valuable(b, attackers, depth);
    }

    // iterate over all the stored gain values to find the max - negamax style
    for i in (1..depth).rev() {
        gain[i - 1] = -max(-gain[i - 1], gain[i]);
    }
    gain[0]
}

fn see_get_least_valuable(b: &Board, attackers: u64, depth: usize) -> (usize, u64) {
    let colour = (b.ctm() + depth) & 1;
    let piece_iter = b.pieces_iter().enumerate().skip(colour).step_by(2);

    for (piece, pieces) in piece_iter {
        let p_in_attackers = *pieces & attackers;
        if p_in_attackers > 0 {
            return (piece, p_in_attackers & p_in_attackers.wrapping_neg());
        }
    }

    (12, NO_SQUARES)
}

#[test]
fn test_see_scores() {
    use crate::board::board::{KNIGHT, ROOK};

    crate::init();

    let positions = vec![
        (
            Board::new_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -").unwrap(),
            Move::new(4, 36, ROOK as u32, BLACK as u32, MoveType::Cap),
            100,
        ),
        (
            Board::new_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -").unwrap(),
            Move::new(19, 36, KNIGHT as u32, BLACK as u32, MoveType::Cap),
            -225,
        ),
    ];

    for (b, m, score) in positions {
        assert_eq!(score, see(&b, m))
    }
}
