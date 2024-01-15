use crate::board::{Board, BISHOP, BLACK, KING, KNIGHT, QUEEN, ROOK, WHITE};
use crate::move_info::{FA, FH, MT, R2, R7, RIGHT_DIR, SQUARES, UP_DIR, UP_LEFT_DIR, UP_RIGHT_DIR};
use crate::move_list::MoveList;
use crate::moves::{Move, MoveType, PrevMoves};
use std::cmp::{max, min};

pub const ALL_SQUARES: u64 = u64::MAX;
pub const NO_SQUARES: u64 = 0;

pub fn gen_moves(b: &Board, ml: &mut impl MoveList, in_check: bool) {
    if in_check {
        gen_check_moves(b, ml);
    } else {
        gen_all_moves(b, ml);
    }
}

pub fn gen_all_attacks(b: &Board, ml: &mut impl MoveList) {
    gen_attack(b, ml, NO_SQUARES, ALL_SQUARES);
}

pub fn gen_all_moves(b: &Board, ml: &mut impl MoveList) {
    gen_attack(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_quiet(b, ml, NO_SQUARES, ALL_SQUARES);
    gen_king_castle(b, ml);
}

pub fn gen_check_moves(b: &Board, ml: &mut impl MoveList) {
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

pub fn gen_move_piece_to_quiet(
    b: &Board,
    ml: &mut impl MoveList,
    piece: usize,
    from: usize,
    to: usize,
) {
    let pinned = !SQUARES[from];
    let target = SQUARES[to];

    match piece {
        0 => gen_wpawn_quiet(b, ml, pinned, target),
        1 => gen_bpawn_quiet(b, ml, pinned, target),
        2 | 3 => gen_knight_quiet(b, ml, pinned, target),
        4 | 5 => gen_rook_quiet(b, ml, pinned, target),
        6 | 7 => gen_bishop_quiet(b, ml, pinned, target),
        8 | 9 => gen_queen_quiet(b, ml, pinned, target),
        10 | 11 => {
            gen_king_quiet(b, ml, pinned, target);
            gen_king_castle(b, ml)
        }
        _ => panic!(),
    }
}

pub fn gen_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_quiet_no_king(b, ml, pinned, target);
    gen_king_quiet(b, ml, pinned, target);
}

pub fn gen_quiet_no_king(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_quiet(b, ml, pinned, target);
    gen_bishop_quiet(b, ml, pinned, target);
    gen_rook_quiet(b, ml, pinned, target);
    gen_knight_quiet(b, ml, pinned, target);
    gen_wpawn_quiet(b, ml, pinned, target);
    gen_bpawn_quiet(b, ml, pinned, target);
}

pub fn gen_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_attack_no_king(b, ml, pinned, target);
    gen_king_attack(b, ml, pinned, target);
}
pub fn gen_attack_no_king(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    gen_queen_attack(b, ml, pinned, target);
    gen_bishop_attack(b, ml, pinned, target);
    gen_rook_attack(b, ml, pinned, target);
    gen_knight_attack(b, ml, pinned, target);
    gen_wpawn_attack(b, ml, pinned, target);
    gen_bpawn_attack(b, ml, pinned, target);
}

const PROMO_PIECES: [u32; 4] = [QUEEN as u32, KNIGHT as u32, ROOK as u32, BISHOP as u32];
const PROMO_CAPS: [MoveType; 4] = [
    MoveType::QPromoCap,
    MoveType::RPromoCap,
    MoveType::NPromoCap,
    MoveType::BPromoCap,
];

pub fn gen_wpawn_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    if b.ctm() == BLACK {
        return;
    }

    let pawns = b.pawns(WHITE) & !pinned;
    let occ = b.all_occ() | !target;
    let quiet = pawns & !(occ >> 8);

    let push = quiet & !R7;
    add_wpawn_push(ml, push);

    let double = (pawns & R2) & !(occ >> 16) & !(b.all_occ() >> 8);
    add_wpawn_double(ml, double);

    let promo = quiet & R7;
    add_wpawn_promo(ml, promo);
}

fn add_wpawn_push(ml: &mut impl MoveList, mut push: u64) {
    while push > 0 {
        let from = push.trailing_zeros();
        push &= push - 1;

        ml.add_move(Move::new(from, from + 8, 0, 0, MoveType::Quiet));
    }
}

fn add_wpawn_double(ml: &mut impl MoveList, mut double: u64) {
    while double > 0 {
        let from = double.trailing_zeros();
        double &= double - 1;

        ml.add_move(Move::new(from, from + 16, 0, 0, MoveType::Double));
    }
}

fn add_wpawn_promo(ml: &mut impl MoveList, mut promo: u64) {
    while promo > 0 {
        let from = promo.trailing_zeros();
        promo &= promo - 1;

        for promo_piece in PROMO_PIECES {
            ml.add_move(Move::new(from, from + 8, 0, promo_piece, MoveType::Promo))
        }
    }
}

pub fn gen_wpawn_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    if b.ctm() == BLACK {
        return;
    }

    let pawns = b.pawns(WHITE) & !pinned;
    let opp = b.occ(BLACK) & target;

    let lefts = (pawns & !FA) & (opp >> 7);
    let rights = (pawns & !FH) & (opp >> 9);

    let up_left = lefts & !R7;
    add_wpawn_attacks(b, ml, up_left, 7);

    let up_right = rights & !R7;
    add_wpawn_attacks(b, ml, up_right, 9);

    let up_left_promos = lefts & R7;
    add_wpawn_attack_promos(b, ml, up_left_promos, 7);

    let up_right_promos = rights & R7;
    add_wpawn_attack_promos(b, ml, up_right_promos, 9);

    add_wpawn_ep(b, ml, pawns, FA, 7, target);
    add_wpawn_ep(b, ml, pawns, FH, 9, target);
}

fn add_wpawn_attacks(b: &Board, ml: &mut impl MoveList, mut attacks: u64, direction: u32) {
    while attacks > 0 {
        let from = attacks.trailing_zeros();
        let xpiece = get_xpiece(b, from + direction).unwrap();
        attacks &= attacks - 1;

        ml.add_move(Move::new(from, from + direction, 0, xpiece, MoveType::Cap));
    }
}

fn add_wpawn_attack_promos(b: &Board, ml: &mut impl MoveList, mut attacks: u64, direction: u32) {
    while attacks > 0 {
        let from = attacks.trailing_zeros();
        let xpiece = get_xpiece(b, from + direction).unwrap();
        attacks &= attacks - 1;

        for move_type in PROMO_CAPS {
            ml.add_move(Move::new(from, from + direction, 0, xpiece, move_type));
        }
    }
}

fn add_wpawn_ep(b: &Board, ml: &mut impl MoveList, pawns: u64, file: u64, shift: u32, target: u64) {
    let ep = b.ep() as u32;
    if ep < 64 && (SQUARES[ep as usize] & ((pawns & !file) << shift) & target << 8) > 0 {
        ml.add_move(Move::new(ep - shift, ep, 0, 1, MoveType::Ep))
    }
}

pub fn gen_bpawn_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    if b.ctm() == WHITE {
        return;
    }

    let pawns = b.pawns(BLACK) & !pinned;
    let occ = b.all_occ() | !target;
    let quiet = pawns & !(occ << 8);

    let push = quiet & !R2;
    add_bpawn_push(ml, push);

    let double = (pawns & R7) & !(occ << 16) & !(b.all_occ() << 8);
    add_bpawn_double(ml, double);

    let promo = quiet & R2;
    add_bpawn_promo(ml, promo);
}

fn add_bpawn_push(ml: &mut impl MoveList, mut push: u64) {
    while push > 0 {
        let from = push.trailing_zeros();
        push &= push - 1;

        ml.add_move(Move::new(from, from - 8, 1, 0, MoveType::Quiet));
    }
}

fn add_bpawn_double(ml: &mut impl MoveList, mut double: u64) {
    while double > 0 {
        let from = double.trailing_zeros();
        double &= double - 1;

        ml.add_move(Move::new(from, from - 16, 1, 0, MoveType::Double));
    }
}

fn add_bpawn_promo(ml: &mut impl MoveList, mut promo: u64) {
    while promo > 0 {
        let from = promo.trailing_zeros();
        promo &= promo - 1;

        for promo_piece in PROMO_PIECES {
            ml.add_move(Move::new(
                from,
                from - 8,
                1,
                promo_piece + BLACK as u32,
                MoveType::Promo,
            ))
        }
    }
}

pub fn gen_bpawn_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    if b.ctm() == WHITE {
        return;
    }

    let pawns = b.pawns(BLACK) & !pinned;
    let opp = b.occ(WHITE) & target;

    let lefts = (pawns & !FA) & (opp << 9);
    let rights = (pawns & !FH) & (opp << 7);

    let down_left = lefts & !R2;
    add_bpawn_attacks(b, ml, down_left, 9);

    let down_right = rights & !R2;
    add_bpawn_attacks(b, ml, down_right, 7);

    let down_left_promos = lefts & R2;
    add_bpawn_attack_promos(b, ml, down_left_promos, 9);

    let down_right_promos = rights & R2;
    add_bpawn_attack_promos(b, ml, down_right_promos, 7);

    add_bpawn_ep(b, ml, pawns, FA, 9, target);
    add_bpawn_ep(b, ml, pawns, FH, 7, target);
}

fn add_bpawn_attacks(b: &Board, ml: &mut impl MoveList, mut attacks: u64, direction: u32) {
    while attacks > 0 {
        let from = attacks.trailing_zeros();
        let xpiece = get_xpiece(b, from - direction).unwrap();
        attacks &= attacks - 1;

        ml.add_move(Move::new(from, from - direction, 1, xpiece, MoveType::Cap));
    }
}

fn add_bpawn_attack_promos(b: &Board, ml: &mut impl MoveList, mut attacks: u64, direction: u32) {
    while attacks > 0 {
        let from = attacks.trailing_zeros();
        let xpiece = get_xpiece(b, from - direction).unwrap();
        attacks &= attacks - 1;

        for move_type in PROMO_CAPS {
            ml.add_move(Move::new(from, from - direction, 1, xpiece, move_type));
        }
    }
}

fn add_bpawn_ep(b: &Board, ml: &mut impl MoveList, pawns: u64, file: u64, shift: u32, target: u64) {
    let ep = b.ep() as u32;
    if ep < 64 && (SQUARES[ep as usize] & ((pawns & !file) >> shift) & target >> 8) > 0 {
        ml.add_move(Move::new(ep + shift, ep, 1, 0, MoveType::Ep))
    }
}

pub fn gen_knight_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = MT::knight_moves;
    gen_piece_quiet(b, ml, KNIGHT, moves, pinned, target)
}

pub fn gen_knight_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = MT::knight_moves;
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
    let moves = MT::king_moves;
    gen_piece_quiet(b, ml, KING, moves, pinned, target)
}

pub fn gen_king_attack(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    let moves = MT::king_moves;
    gen_piece_attack(b, ml, KING, moves, pinned, target)
}

pub fn gen_king_castle(b: &Board, ml: &mut impl MoveList) {
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

fn gen_piece_quiet<F: Fn(usize) -> u64>(
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

fn add_attack(b: &Board, ml: &mut impl MoveList, from: u32, mut attack: u64, piece: u32) {
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

    let dir = match diff {
        _ if higher / 8 == lower / 8 => RIGHT_DIR,
        _ if diff % 8 == 0 => UP_DIR,
        _ if diff % 7 == 0 => UP_LEFT_DIR,
        _ if diff % 9 == 0 => UP_RIGHT_DIR,
        _ => return 0,
    };

    MT::rays(dir, lower) & (SQUARES[higher] - 1)
}

#[inline]
pub fn get_piece(board: &Board, sq: u32) -> Option<u32> {
    find_piece(board, sq, board.ctm())
}

#[inline]
pub fn get_xpiece(board: &Board, sq: u32) -> Option<u32> {
    find_piece(board, sq, board.opp_ctm())
}

fn find_piece(board: &Board, sq: u32, ctm: usize) -> Option<u32> {
    let sq = SQUARES[sq as usize];
    board
        .pieces_iter()
        .enumerate()
        .skip(ctm)
        .step_by(2)
        .find(|(_, pieces)| sq & *pieces > 0)
        .map(|(piece, _)| piece as u32)
}

#[inline]
pub fn sq_attacked(board: &Board, sq: usize, attacker_colour: usize) -> bool {
    get_attackers(board, sq, attacker_colour) > 0
}

#[inline]
pub fn get_all_attackers(board: &Board, sq: usize) -> u64 {
    get_attackers(board, sq, WHITE) | get_attackers(board, sq, BLACK)
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

#[inline]
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
