use crate::board::board::{Board, ALL_PIECES, BISHOP, KING, KNIGHT, QUEEN, ROOK};
use crate::movegen::move_info::{RIGHT_DIR, SQUARES, UP_DIR, UP_LEFT_DIR, UP_RIGHT_DIR};
use crate::movegen::move_list::MoveList;
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::get_xpiece;
use crate::movegen::moves::{Move, MoveType};
use crate::movegen::piece_gen::black_pawn::{gen_bpawn_attack, gen_bpawn_quiet};
use crate::movegen::piece_gen::white_pawn::{gen_wpawn_attack, gen_wpawn_quiet};
use std::cmp::{max, min};

pub fn gen_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_quiet_no_king(b, ml, pinned, target);
    gen_king_quiet(b, ml, pinned, target);
}

pub fn gen_quiet_no_king<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_quiet(b, ml, pinned, target);
    gen_bishop_quiet(b, ml, pinned, target);
    gen_rook_quiet(b, ml, pinned, target);
    gen_knight_quiet(b, ml, pinned, target);
    gen_wpawn_quiet(b, ml, pinned, target);
    gen_bpawn_quiet(b, ml, pinned, target);
}

pub fn gen_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_attack_no_king(b, ml, pinned, target);
    gen_king_attack(b, ml, pinned, target);
}
pub fn gen_attack_no_king<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_attack(b, ml, pinned, target);
    gen_bishop_attack(b, ml, pinned, target);
    gen_rook_attack(b, ml, pinned, target);
    gen_knight_attack(b, ml, pinned, target);
    gen_wpawn_attack(b, ml, pinned, target);
    gen_bpawn_attack(b, ml, pinned, target);
}

pub fn gen_knight_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::knight_moves(from);
    gen_piece_quiet(b, ml, KNIGHT, moves, pinned, target)
}

pub fn gen_knight_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::knight_moves(from);
    gen_piece_attack(b, ml, KNIGHT, moves, pinned, target)
}

pub fn gen_rook_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::rook_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, ROOK, moves, pinned, target)
}

pub fn gen_rook_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::rook_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, ROOK, moves, pinned, target)
}

pub fn gen_bishop_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, BISHOP, moves, pinned, target)
}

pub fn gen_bishop_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, BISHOP, moves, pinned, target)
}

pub fn gen_queen_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from) | MT::rook_moves(b.all_occ(), from);
    gen_piece_quiet(b, ml, QUEEN, moves, pinned, target)
}

pub fn gen_queen_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::bishop_moves(b.all_occ(), from) | MT::rook_moves(b.all_occ(), from);
    gen_piece_attack(b, ml, QUEEN, moves, pinned, target)
}

pub fn gen_king_quiet<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::king_moves(from);
    gen_piece_quiet(b, ml, KING, moves, pinned, target)
}

pub fn gen_king_attack<'a>(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = |from| MT::king_moves(from);
    gen_piece_attack(b, ml, KING, moves, pinned, target)
}

fn gen_piece_quiet<'a, F: Fn(usize) -> u64>(
    b: &Board,
    ml: &mut impl MoveList,
    mut piece: usize,
    moves: F,
    pinned: u64,
    target: u64,
) {
    piece += b.ctm();
    let mut pieces = b.pieces(piece) & !pinned;

    while pieces > 0 {
        let from = pieces.trailing_zeros();
        pieces &= pieces - 1;

        let quiet = moves(from as usize) & !b.all_occ() & target;
        add_quiet(ml, from, quiet, piece as u32);
    }
}

fn add_quiet<'a>(ml: &mut impl MoveList, from: u32, mut quiet: u64, piece: u32) {
    while quiet > 0 {
        let to = quiet.trailing_zeros();
        ml.add_move(Move::new(from, to, piece, 0, MoveType::Quiet));
        quiet &= quiet - 1;
    }
}

fn gen_piece_attack<'a, F: Fn(usize) -> u64>(
    b: &Board,
    ml: &mut impl MoveList,
    mut piece: usize,
    moves: F,
    pinned: u64,
    target: u64,
) {
    piece += b.ctm();
    let mut pieces = b.pieces(piece) & !pinned;
    let opp = b.occ(b.opp_ctm()) & target;

    while pieces > 0 {
        let from = pieces.trailing_zeros();
        pieces &= pieces - 1;

        let attack = moves(from as usize) & opp;
        add_attack(b, ml, from, attack, piece as u32);
    }
}

fn add_attack<'a>(b: &Board, ml: &mut impl MoveList, from: u32, mut attack: u64, piece: u32) {
    while attack > 0 {
        let to = attack.trailing_zeros();
        let xpiece = get_xpiece(b, to).unwrap();
        ml.add_move(Move::new(from, to, piece, xpiece, MoveType::Cap));
        attack &= attack - 1;
    }
}

pub fn gen_pinned_rays(b: &Board, ksq: usize) -> u64 {
    let rq = b.rooks(b.opp_ctm()) | b.queens(b.opp_ctm());
    let rq_pinners = rq & MT::rook_xray_moves(b.all_occ(), b.occ(b.ctm()), ksq);

    let bq = b.bishops(b.opp_ctm()) | b.queens(b.opp_ctm());
    let bq_pinners = bq & MT::bishop_xray_moves(b.all_occ(), b.occ(b.ctm()), ksq);

    let mut pinners = rq_pinners | bq_pinners;
    let mut pinned_rays = 0;

    while pinners > 0 {
        let p_sq = pinners.trailing_zeros() as usize;
        pinned_rays |= get_ray_inbetween(ksq, p_sq);
        pinners &= pinners - 1;
    }

    pinned_rays
}
pub fn get_ray_inbetween(sq1: usize, sq2: usize) -> u64 {
    let higher = max(sq1, sq2);
    let lower = min(sq1, sq2);
    let diff = higher - lower;

    let dir = if higher / 8 == lower / 8 {
        RIGHT_DIR
    } else if diff % 8 == 0 {
        UP_DIR
    } else if diff % 7 == 0 {
        UP_LEFT_DIR
    } else if diff % 9 == 0 {
        UP_RIGHT_DIR
    } else {
        return 0;
    };

    MT::rays(dir, lower) & (SQUARES[higher] - 1)
}
