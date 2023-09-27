use crate::board::board::{Board, ALL_PIECES, BISHOP, KING, KNIGHT, QUEEN, ROOK};
use crate::movegen::move_list::MoveList;
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::{get_attackers, get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::moves::{Move, MoveType};

mod black_pawn;
mod king;
mod white_pawn;

use crate::movegen::movegen_iters::movegen_iter::{gen_pinned_rays, get_ray_inbetween};
use crate::movegen::piece_gen::king::{king_safe_attack_moves, king_safe_quiet_moves};
use crate::print_bb;
pub use black_pawn::{gen_bpawn_attack, gen_bpawn_quiet};
pub use king::gen_king_castle;
pub use white_pawn::{gen_wpawn_attack, gen_wpawn_quiet};

fn gen_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_quiet_no_king(b, ml, pinned, target);
    gen_king_quiet(b, ml, pinned, target);
}

fn gen_quiet_no_king(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_quiet(b, ml, pinned, target);
    gen_bishop_quiet(b, ml, pinned, target);
    gen_rook_quiet(b, ml, pinned, target);
    gen_knight_quiet(b, ml, pinned, target);
    gen_wpawn_quiet(b, ml, pinned, target);
    gen_bpawn_quiet(b, ml, pinned, target);
}

fn gen_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_attack_no_king(b, ml, pinned, target);
    gen_king_attack(b, ml, pinned, target);
}
fn gen_attack_no_king(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_attack(b, ml, pinned, target);
    gen_bishop_attack(b, ml, pinned, target);
    gen_rook_attack(b, ml, pinned, target);
    gen_knight_attack(b, ml, pinned, target);
    gen_wpawn_attack(b, ml, pinned, target);
    gen_bpawn_attack(b, ml, pinned, target);
}

pub fn gen_all_moves(b: &Board, ml: &mut impl MoveList) {
    gen_attack(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_quiet(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_king_castle(b, ml);
}

pub fn gen_check_moves(b: &Board, ml: &mut impl MoveList) {
    gen_king_quiet(b, ml, NO_SQUARES, king_safe_quiet_moves(b));
    gen_king_attack(b, ml, NO_SQUARES, king_safe_attack_moves(b));

    let ksq = b.king_idx(b.ctm);
    let attacker = get_attackers(b, ksq, b.ctm ^ 1);

    // only king moves matter if there is more than one attacker
    if attacker.count_ones() >= 2 {
        return;
    }

    assert_ne!(attacker, 0);

    let pinned_rays = gen_pinned_rays(b, ksq);
    let pinned = pinned_rays & b.occ(b.ctm);

    let attacker_sq = attacker.trailing_zeros() as usize;
    let attacker_ray = get_ray_inbetween(ksq, attacker_sq);

    // print_bb!(pinned_rays, pinned, attacker_ray);

    gen_attack_no_king(b, ml, pinned, attacker);

    let attack_piece = get_xpiece(b, attacker_sq as u32).unwrap();
    // return if attacker is not a sliding piece
    if attack_piece < ROOK as u32 && attack_piece < KING as u32 {
        return;
    }

    gen_quiet_no_king(b, ml, pinned, attacker_ray);
}

pub fn gen_knight_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::knight_moves(from);
    gen_piece_quiet(b, ml, KNIGHT, moves, pinned, target)
}

pub fn gen_knight_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::knight_moves(from);
    gen_piece_attack(b, ml, KNIGHT, moves, pinned, target)
}

pub fn gen_rook_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::rook_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, ROOK, moves, pinned, target)
}

pub fn gen_rook_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::rook_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, ROOK, moves, pinned, target)
}

pub fn gen_bishop_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, BISHOP, moves, pinned, target)
}

pub fn gen_bishop_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, BISHOP, moves, pinned, target)
}

pub fn gen_queen_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from) | MT::rook_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, QUEEN, moves, pinned, target)
}

pub fn gen_queen_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from) | MT::rook_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, QUEEN, moves, pinned, target)
}

pub fn gen_king_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::king_moves(from);
    gen_piece_quiet(b, ml, KING, moves, pinned, target)
}

pub fn gen_king_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::king_moves(from);
    gen_piece_attack(b, ml, KING, moves, pinned, target)
}

fn gen_piece_quiet<F: Fn(usize) -> u64>(
    b: &Board,
    ml: &mut impl MoveList,
    mut piece: usize,
    moves: F,
    pinned: u64,
    target: u64,
) {
    piece += b.ctm;
    let mut pieces = b.pieces[piece] & !pinned;

    while pieces > 0 {
        let from = pieces.trailing_zeros();
        pieces &= pieces - 1;

        let quiet = moves(from as usize) & !b.util[ALL_PIECES] & target;
        add_quiet(ml, from, quiet, piece as u32);
    }
}

fn add_quiet(ml: &mut impl MoveList, from: u32, mut quiet: u64, piece: u32) {
    while quiet > 0 {
        let to = quiet.trailing_zeros();
        ml.add_move(Move::new(from, to, piece, 0, MoveType::Quiet));
        quiet &= quiet - 1;
    }
}

fn gen_piece_attack<F: Fn(usize) -> u64>(
    b: &Board,
    ml: &mut impl MoveList,
    piece: usize,
    moves: F,
    pinned: u64,
    target: u64,
) {
    let piece = piece + b.ctm;
    let mut pieces = b.pieces[piece] & !pinned;
    let opp = b.util[b.ctm ^ 1] & target;

    while pieces > 0 {
        let from = pieces.trailing_zeros();
        pieces &= pieces - 1;

        let attack = moves(from as usize) & opp;
        add_attack(b, ml, from, attack, piece as u32);
    }
}

fn add_attack(b: &Board, ml: &mut impl MoveList, from: u32, mut attack: u64, piece: u32) {
    while attack > 0 {
        let to = attack.trailing_zeros();
        let xpiece = get_xpiece(b, to).unwrap();
        ml.add_move(Move::new(from, to, piece, xpiece, MoveType::Cap));
        attack &= attack - 1;
    }
}
