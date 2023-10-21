use crate::board::board::{Board, ALL_PIECES, KING};
use crate::movegen::move_list::MoveList;
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::sq_attacked;
use crate::movegen::moves::{Move, MoveType};

pub fn gen_king_castle<'a>(b: &Board, ml: &mut impl MoveList) {
    let colour_rights = b.castle_state() >> (2 * (b.opp_ctm()));
    let can_kingside = colour_rights & 0b10 > 0;
    let can_queenside = colour_rights & 1 > 0;
    let piece = KING + b.ctm();

    let from = b.king_idx(b.ctm()) as u32;

    let kingside_mask = 0x60 << (b.ctm() * 56);
    if can_kingside && b.all_occ() & kingside_mask == 0 {
        let move_type = MoveType::kingside(b.ctm());
        ml.add_move(Move::new(from, from + 2, piece as u32, 0, move_type));
    }

    let queenside_mask = 0xE << (b.ctm() * 56);
    if can_queenside && b.all_occ() & queenside_mask == 0 {
        let move_type = MoveType::queenside(b.ctm());
        ml.add_move(Move::new(from, from - 2, piece as u32, 0, move_type));
    }
}

pub fn king_safe_quiet_moves(b: &Board) -> u64 {
    let from = b.king_idx(b.ctm());
    let mut moves = MT::king_moves(from) & !b.all_occ();

    let mut safe = 0;

    while moves > 0 {
        let to = moves.trailing_zeros();
        moves &= moves - 1;

        if !sq_attacked(b, to as usize, b.opp_ctm()) {
            safe |= 1 << to
        }
    }

    safe
}

pub fn king_safe_attack_moves(b: &Board) -> u64 {
    let from = b.king_idx(b.ctm());
    let mut moves = MT::king_moves(from) & b.occ(b.opp_ctm());

    let mut safe = 0;

    while moves > 0 {
        let to = moves.trailing_zeros();
        moves &= moves - 1;

        if !sq_attacked(b, to as usize, b.opp_ctm()) {
            safe |= 1 << to
        }
    }

    safe
}
