use crate::board::board::{Board, ALL_PIECES, BISHOP, BLACK, KING, QUEEN, ROOK};
use crate::movegen::move_info::{FA, FH, R2, R7, SQUARES};
use crate::movegen::move_list::MoveList;
use crate::movegen::movegen::get_xpiece;
use crate::movegen::moves::{Move, MoveType};

const PROMO_PIECES: [u32; 4] = [QUEEN as u32, KING as u32, ROOK as u32, BISHOP as u32];
const PROMO_CAPS: [MoveType; 4] = [
    MoveType::QPromoCap,
    MoveType::RPromoCap,
    MoveType::NPromoCap,
    MoveType::BPromoCap,
];

pub fn gen_wpawn_quiet(b: &Board, ml: &mut impl MoveList, pinned: u64, target: u64) {
    if b.ctm == BLACK {
        return;
    }

    let pawns = b.pieces[0] & !pinned;
    let occ = b.util[ALL_PIECES] | !target;
    let quiet = pawns & !(occ >> 8);

    let push = quiet & !R7;
    add_wpawn_push(ml, push);

    let double = (pawns & R2) & !(occ >> 16) & !(b.util[ALL_PIECES] >> 8);
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
    if b.ctm == 1 {
        return;
    }

    let pawns = b.pieces[0] & !pinned;
    let opp = b.util[BLACK] & target;

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
    let ep = b.ep as u32;
    if ep < 64 && (SQUARES[ep as usize] & ((pawns & !file) << shift) & target << 8) > 0 {
        ml.add_move(Move::new(ep - shift, ep, 0, 1, MoveType::Ep))
    }
}
