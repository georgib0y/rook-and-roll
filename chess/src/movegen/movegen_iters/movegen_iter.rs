use crate::board::board::{Board, ALL_PIECES, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::movegen::move_info::SQUARES;
use crate::movegen::move_tables::MT;
use crate::movegen::movegen::{
    get_attackers, get_xpiece, is_in_check, is_legal_move, moved_into_check, ALL_SQUARES,
    NO_SQUARES,
};
use crate::movegen::movegen_iters::bishop_iter::{BishopAttackIterator, BishopQuietIterator};
use crate::movegen::movegen_iters::black_pawn_iter::{
    BlackPawnAttackIterator, BlackPawnQuietIterator,
};
use crate::movegen::movegen_iters::king_iter::{
    KingAttackIterator, KingCastleIterator, KingMovesIterator, KingQuietIterator,
};
use crate::movegen::movegen_iters::knight_iter::{KnightAttackIterator, KnightQuietIterator};
use crate::movegen::movegen_iters::queen_iter::{QueenAttackIterator, QueenQuietIterator};
use crate::movegen::movegen_iters::rook_iter::{RookAttackIterator, RookQuietIterator};
use crate::movegen::movegen_iters::white_pawn_iter::{
    WhitePawnAttackIterator, WhitePawnQuietIterator,
};
use crate::movegen::moves::Move;
use crate::print_bb;
use std::cmp::{max, min};
use std::iter::{Chain, Map};

pub fn next_moves(board: &Board) -> impl Iterator<Item = (Move, Board)> + '_ {
    all_moves(board)
        .map(|m| (m, board.copy_make(m)))
        .filter(|(m, b)| !moved_into_check(b, *m))
}

pub fn next_check_moves(board: &Board) -> impl Iterator<Item = (Move, Board)> + '_ {
    check_moves(board)
        .map(|m| (m, board.copy_make(m)))
        .filter(|(m, b)| !moved_into_check(b, *m))
}

type QuietMoveIterator = Chain<
    Chain<
        Chain<
            Chain<Chain<QueenQuietIterator, BishopQuietIterator>, RookQuietIterator>,
            KnightQuietIterator,
        >,
        WhitePawnQuietIterator,
    >,
    BlackPawnQuietIterator,
>;

fn quiets(b: &Board, pinned: u64, target: u64) -> QuietMoveIterator {
    QueenQuietIterator::new(b, pinned, target)
        .chain(BishopQuietIterator::new(b, pinned, target))
        .chain(RookQuietIterator::new(b, pinned, target))
        .chain(KnightQuietIterator::new(b, pinned, target))
        .chain(WhitePawnQuietIterator::new(b, pinned, target))
        .chain(BlackPawnQuietIterator::new(b, pinned, target))
}

type AttackMoveIterator<'a> = Chain<
    Chain<
        Chain<
            Chain<Chain<QueenAttackIterator<'a>, BishopAttackIterator<'a>>, RookAttackIterator<'a>>,
            KnightAttackIterator<'a>,
        >,
        WhitePawnAttackIterator<'a>,
    >,
    BlackPawnAttackIterator<'a>,
>;

fn attacks(b: &Board, pinned: u64, target: u64) -> AttackMoveIterator {
    QueenAttackIterator::new(b, pinned, target)
        .chain(BishopAttackIterator::new(b, pinned, target))
        .chain(RookAttackIterator::new(b, pinned, target))
        .chain(KnightAttackIterator::new(b, pinned, target))
        .chain(WhitePawnAttackIterator::new(b, pinned, target))
        .chain(BlackPawnAttackIterator::new(b, pinned, target))
}

type AllMoveIterator<'a> = Chain<
    Chain<
        Chain<Chain<AttackMoveIterator<'a>, KingAttackIterator<'a>>, QuietMoveIterator>,
        KingQuietIterator,
    >,
    KingCastleIterator,
>;

pub fn all_moves(b: &Board) -> AllMoveIterator {
    attacks(b, NO_SQUARES, ALL_SQUARES)
        .chain(KingAttackIterator::new(b))
        .chain(quiets(b, NO_SQUARES, ALL_SQUARES))
        .chain(KingQuietIterator::new(b))
        .chain(KingCastleIterator::new(b))
}

type CheckKingMoveIterator<'a> = KingMovesIterator<'a>;
type CheckAttackMoveIterator<'a> = Chain<KingMovesIterator<'a>, AttackMoveIterator<'a>>;
type CheckAllMoveIterator<'a> =
    Chain<Chain<KingMovesIterator<'a>, AttackMoveIterator<'a>>, QuietMoveIterator>;

pub enum CheckMoveIterator<'a> {
    KingMoves(CheckKingMoveIterator<'a>),
    Attacks(CheckAttackMoveIterator<'a>),
    All(CheckAllMoveIterator<'a>),
}

impl<'a> Iterator for CheckMoveIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CheckMoveIterator::KingMoves(iter) => iter.next(),
            CheckMoveIterator::Attacks(iter) => iter.next(),
            CheckMoveIterator::All(iter) => iter.next(),
        }
    }
}

pub fn check_moves(b: &Board) -> CheckMoveIterator {
    // gen all legal king moves
    let ksq = b.pieces[KING + b.ctm].trailing_zeros() as usize;
    let attackers = get_attackers(b, ksq, b.ctm ^ 1);

    // if double check than only king moves matter
    if attackers.count_ones() >= 2 {
        return CheckMoveIterator::KingMoves(KingMovesIterator::new(b));
    }

    let pinned_rays = gen_pinned_rays(b, ksq);
    let asq = attackers.trailing_zeros() as usize;
    let attacker_ray = get_ray_inbetween(ksq, asq);
    let inbetween = mask_ray_between(ksq, asq, attacker_ray);
    let pinned_pieces = pinned_rays & b.util[ALL_PIECES];

    // print_bb!(
    //     attackers,
    //     pinned_rays,
    //     attacker_ray,
    //     inbetween,
    //     pinned_pieces
    // );
    //
    // try and move in the way of the attacker and the sliding piece
    let attack_piece = get_xpiece(b, attackers.trailing_zeros()).unwrap();
    // return if attacker is not a sliding piece
    if attack_piece < ROOK as u32 && attack_piece < KING as u32 {
        return CheckMoveIterator::Attacks(KingMovesIterator::new(b).chain(attacks(
            b,
            pinned_pieces,
            attackers,
        )));
    }

    CheckMoveIterator::All(
        KingMovesIterator::new(b)
            .chain(attacks(b, pinned_pieces, attackers))
            .chain(quiets(b, pinned_pieces, inbetween)),
    )
}

pub fn gen_pinned_rays(b: &Board, ksq: usize) -> u64 {
    let mut rq = b.pieces[ROOK + (b.ctm ^ 1)];
    rq |= b.pieces[QUEEN + (b.ctm ^ 1)];
    let rq_pinners = rq & MT::rook_xray_moves(b.util[ALL_PIECES], b.util[b.ctm], ksq);

    let mut bq = b.pieces[BISHOP + (b.ctm ^ 1)];
    bq |= b.pieces[QUEEN + (b.ctm ^ 1)];
    let bq_pinners = bq & MT::bishop_xray_moves(b.util[ALL_PIECES], b.util[b.ctm], ksq);

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

    // if right
    let dir = if higher / 8 == lower / 8 {
        3
    // if up left
    } else if diff % 7 == 0 {
        0
    // if up
    } else if diff % 8 == 0 {
        1
    // if up right
    } else if diff % 9 == 0 {
        2
    } else {
        return 0;
    };

    MT::rays(dir, lower) & (SQUARES[higher] - 1)
}

fn mask_ray_between(sq1: usize, sq2: usize, ray: u64) -> u64 {
    let high_sq = max(sq1, sq2) as u64;
    let low_sq = min(sq1, sq2) as u64;

    let above_low_sq = !1 << low_sq;
    let below_high_sq = (1 << high_sq) - 1;

    ray & above_low_sq & below_high_sq
}
