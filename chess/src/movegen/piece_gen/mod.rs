use crate::board::board::{Board, KING, ROOK};
use crate::movegen::move_info::SQUARES;
use crate::movegen::move_list::MoveList;
use crate::movegen::movegen::{get_attackers, get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::piece_gen::black_pawn::gen_bpawn_quiet;
use crate::movegen::piece_gen::king::{
    gen_king_castle, king_safe_attack_moves, king_safe_quiet_moves,
};
use crate::movegen::piece_gen::piece_gen::*;
use crate::movegen::piece_gen::white_pawn::gen_wpawn_quiet;

mod black_pawn;
mod king;
mod piece_gen;
mod white_pawn;

pub fn gen_moves<'a>(b: &Board, ml: &mut impl MoveList, in_check: bool) {
    if in_check {
        gen_check_moves(b, ml);
    } else {
        gen_all_moves(b, ml);
    }
}

pub fn gen_all_attacks<'a>(b: &Board, ml: &mut impl MoveList) {
    gen_attack(b, ml, NO_SQUARES, ALL_SQUARES);
}

pub fn gen_all_moves<'a>(b: &Board, ml: &mut impl MoveList) {
    gen_attack(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_quiet(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_king_castle(b, ml);
}

pub fn gen_check_moves<'a>(b: &Board, ml: &mut impl MoveList) {
    gen_king_quiet(b, ml, NO_SQUARES, king_safe_quiet_moves(b));
    gen_king_attack(b, ml, NO_SQUARES, king_safe_attack_moves(b));

    let ksq = b.king_idx(b.ctm());
    let attacker = get_attackers(b, ksq, b.opp_ctm());

    // only king moves matter if there is more than one attacker
    if attacker.count_ones() >= 2 {
        return;
    }

    assert_ne!(attacker, 0);

    let pinned_rays = gen_pinned_rays(b, ksq);
    let pinned = pinned_rays & b.occ(b.ctm());

    let attacker_sq = attacker.trailing_zeros() as usize;
    let attacker_ray = get_ray_inbetween(ksq, attacker_sq);

    gen_attack_no_king(b, ml, pinned, attacker);

    let attack_piece = get_xpiece(b, attacker_sq as u32).unwrap();
    // return if attacker is not a sliding piece
    if attack_piece < ROOK as u32 && attack_piece < KING as u32 {
        return;
    }

    gen_quiet_no_king(b, ml, pinned, attacker_ray);
}

pub fn gen_move_piece_to_quiet<'a>(
    b: &Board,
    ml: &mut impl MoveList,
    piece: usize,
    from: usize,
    to: usize,
) {
    let pinned = !SQUARES[from];
    let target = SQUARES[to];

    match piece {
        0 => {
            gen_wpawn_quiet(b, ml, pinned, target);
        }
        1 => {
            gen_bpawn_quiet(b, ml, pinned, target);
        }
        2 | 3 => {
            gen_knight_quiet(b, ml, pinned, target);
        }
        4 | 5 => {
            gen_rook_quiet(b, ml, pinned, target);
        }
        6 | 7 => {
            gen_bishop_quiet(b, ml, pinned, target);
        }
        8 | 9 => {
            gen_queen_quiet(b, ml, pinned, target);
        }
        10 | 11 => {
            gen_king_quiet(b, ml, pinned, target);
            gen_king_castle(b, ml);
        }
        _ => panic!(),
    }
}
