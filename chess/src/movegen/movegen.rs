use crate::board::board::Board;
use crate::movegen::move_info::SQUARES;
use crate::movegen::move_tables::MT;
use crate::movegen::moves::{Move, MoveType, PrevMoves};

pub const ALL_SQUARES: u64 = u64::MAX;
pub const NO_SQUARES: u64 = 0;

pub const ALL_CAP: usize = 218;
pub const CHECK_CAP: usize = 75;
pub const ATTACK_CAP: usize = 100;

pub const CAP_SCORE_OFFSET: i32 = 100000;

pub fn get_piece(board: &Board, sq: u32) -> Option<u32> {
    let bb_sq = SQUARES[sq as usize];
    let s = board.ctm() as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces(p as usize) > 0 {
            return Some(p);
        }
    }

    None
}

pub fn get_xpiece(board: &Board, sq: u32) -> Option<u32> {
    let bb_sq = SQUARES[sq as usize];
    let s = board.opp_ctm() as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces(p as usize) > 0 {
            return Some(p);
        }
    }

    None
}

#[inline]
pub fn sq_attacked(board: &Board, sq: usize, attacker_colour: usize) -> bool {
    get_attackers(board, sq, attacker_colour) > 0
}

#[inline]
pub fn get_all_attackers(board: &Board, sq: usize) -> u64 {
    get_attackers(board, sq, 0) | get_attackers(board, sq, 1)
}

pub fn get_attackers(b: &Board, sq: usize, attacker_colour: usize) -> u64 {
    let mut attackers = 0;
    let pawns = b.pawns(attacker_colour);
    attackers |= MT::pawn_attacks(attacker_colour ^ 1, sq) & pawns;

    let knights = b.knights(attacker_colour);
    attackers |= MT::knight_moves(sq) & knights;

    let king = b.king(attacker_colour);
    attackers |= MT::king_moves(sq) & king;

    let bishop_queen = b.queens(attacker_colour) | b.bishops(attacker_colour);
    attackers |= MT::bishop_moves(b.all_occ(), sq) & bishop_queen;

    let rook_queen = b.rooks(attacker_colour) | b.queens(attacker_colour);
    attackers |= MT::rook_moves(b.all_occ(), sq) & rook_queen;

    attackers
}

#[inline]
pub fn moved_into_check(board: &Board, m: Move) -> bool {
    let ksq = board.king_idx(board.opp_ctm());
    let from_sq = SQUARES[m.from() as usize];
    let superray = MT::superrays(ksq);
    let is_in_ray = from_sq & superray > 0;
    is_in_ray && sq_attacked(board, ksq, board.ctm())
}

pub fn is_in_check(board: &Board) -> bool {
    sq_attacked(board, board.king_idx(board.ctm()), board.opp_ctm())
}

// assumes the board has not been added to prev_moves, so checks if the count is 2
// (as adding the board would make it 3 and therefore three move repetition)
pub fn is_legal_move(board: &Board, m: Move, prev_moves: &PrevMoves) -> bool {
    if board.halfmove() > 100 || prev_moves.get_count(board.hash()) == 2 {
        return false;
    }

    match m.move_type() {
        // check castle moves to see if the king passes through an attacked square
        MoveType::WKingSide => !sq_attacked(board, 5, 1) & !sq_attacked(board, 6, 1),
        MoveType::WQueenSide => !sq_attacked(board, 3, 1) & !sq_attacked(board, 2, 1),
        MoveType::BKingSide => !sq_attacked(board, 61, 0) & !sq_attacked(board, 62, 0),
        MoveType::BQueenSide => !sq_attacked(board, 59, 0) & !sq_attacked(board, 58, 0),
        _ => true,
    }
}
