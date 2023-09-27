use crate::board::board::{Board, ALL_PIECES, BISHOP, KING, QUEEN, ROOK, WHITE};
use crate::movegen::move_info::{FA, FH, R2, R7, SQUARES};
use crate::movegen::movegen::{get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};
use crate::print_bb;

const PROMO_PIECES: [u32; 4] = [
    QUEEN as u32 + 1,
    KING as u32 + 1,
    ROOK as u32 + 1,
    BISHOP as u32 + 1,
];
const PROMO_CAPS: [MoveType; 4] = [
    MoveType::QPromoCap,
    MoveType::RPromoCap,
    MoveType::NPromoCap,
    MoveType::BPromoCap,
];

pub struct BlackPawnQuietIterator {
    quiet: u64,
    double: u64,
    promo_pieces: u64, // all pieces with quiet moves to R1
    promo_piece: usize,
    current_promos: u64, // pawns to process for the current promo piece
}

impl BlackPawnQuietIterator {
    pub fn new(b: &Board, pinned: u64, target: u64) -> BlackPawnQuietIterator {
        if b.ctm == 0 {
            return BlackPawnQuietIterator {
                quiet: 0,
                double: 0,
                promo_pieces: 0,
                promo_piece: 0,
                current_promos: 0,
            };
        }

        let pawns = b.pieces[1] & !pinned;
        let occ = b.util[ALL_PIECES] | !target;

        let (quiet, double, promo_pieces) = gen_bquiet_moves(b, pawns, occ);

        BlackPawnQuietIterator {
            quiet,
            double,
            promo_pieces,
            promo_piece: 0,
            current_promos: promo_pieces,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.quiet == 0 {
            return None;
        }

        let from = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(from, from - 8, 1, 0, MoveType::Quiet))
    }

    fn next_double(&mut self) -> Option<Move> {
        if self.double == 0 {
            return None;
        }

        let from = self.double.trailing_zeros();
        self.double &= self.double - 1;

        Some(Move::new(from, from - 16, 1, 0, MoveType::Double))
    }

    fn next_promo(&mut self) -> Option<Move> {
        if self.promo_pieces == 0 {
            return None;
        }

        if self.current_promos == 0 {
            self.current_promos = self.promo_pieces;
            self.promo_piece += 1;
        }

        if self.promo_piece >= PROMO_PIECES.len() {
            return None;
        }

        let from = self.current_promos.trailing_zeros();
        self.current_promos &= self.current_promos - 1;

        Some(Move::new(
            from,
            from - 8,
            1,
            PROMO_PIECES[self.promo_piece],
            MoveType::Promo,
        ))
    }
}

impl Iterator for BlackPawnQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
            .or_else(|| self.next_double())
            .or_else(|| self.next_promo())
    }
}

fn gen_bquiet_moves(b: &Board, pawns: u64, occ: u64) -> (u64, u64, u64) {
    let quiet = pawns & !(occ << 8);
    let push = quiet & !R2;
    let double = (pawns & R7) & !(occ << 16) & !(b.util[ALL_PIECES] << 8);
    let promo = quiet & R2;

    (push, double, promo)
}

pub struct BlackPawnAttackIterator<'a> {
    board: &'a Board,
    down_left: u64,
    down_left_promo: Option<[u64; 4]>,
    down_left_ep: Option<u32>,
    down_right: u64,
    down_right_promo: Option<[u64; 4]>,
    down_right_ep: Option<u32>,
}

impl<'a> BlackPawnAttackIterator<'a> {
    pub fn new(b: &'a Board, pinned: u64, target: u64) -> BlackPawnAttackIterator<'a> {
        if b.ctm == 0 {
            return BlackPawnAttackIterator {
                board: b,
                down_left: 0,
                down_left_promo: None,
                down_left_ep: None,
                down_right: 0,
                down_right_promo: None,
                down_right_ep: None,
            };
        }

        let pawns = b.pieces[1] & !pinned;
        let opp = b.util[WHITE] & target;

        let (left, left_promo, left_ep) = gen_battack_moves(b, pawns, opp, FA, 9, target);
        let (right, right_promo, right_ep) = gen_battack_moves(b, pawns, opp, FH, 7, target);

        BlackPawnAttackIterator {
            board: b,
            down_left: left,
            down_left_promo: left_promo,
            down_left_ep: left_ep,
            down_right: right,
            down_right_promo: right_promo,
            down_right_ep: right_ep,
        }
    }

    fn next_down_right_attack(&mut self) -> Option<Move> {
        if self.down_right == 0 {
            return None;
        }

        let from = self.down_right.trailing_zeros();
        let xpiece = get_xpiece(self.board, from - 7).unwrap();
        self.down_right &= self.down_right - 1;
        Some(Move::new(from, from - 7, 1, xpiece, MoveType::Cap))
    }

    fn next_down_left_attack(&mut self) -> Option<Move> {
        if self.down_left == 0 {
            return None;
        }

        let from = self.down_left.trailing_zeros();
        let xpiece = get_xpiece(self.board, from - 9).unwrap();
        self.down_left &= self.down_left - 1;
        Some(Move::new(from, from - 9, 1, xpiece, MoveType::Cap))
    }

    fn next_down_right_promo_attack(&mut self) -> Option<Move> {
        if self.down_right_promo.is_none() {
            return None;
        }

        let promos = self.down_right_promo.as_mut().unwrap();

        for (piece, pawns) in promos.iter_mut().enumerate() {
            if *pawns == 0 {
                continue;
            }

            let from = pawns.trailing_zeros();
            let xpiece = get_xpiece(self.board, from - 7).unwrap();
            *pawns &= *pawns - 1;

            return Some(Move::new(from, from - 7, 1, xpiece, PROMO_CAPS[piece]));
        }

        None
    }

    fn next_down_left_promo_attack(&mut self) -> Option<Move> {
        if self.down_left_promo.is_none() {
            return None;
        }

        let promos = self.down_left_promo.as_mut().unwrap();

        for (piece, pawns) in promos.iter_mut().enumerate() {
            if *pawns == 0 {
                continue;
            }

            let from = pawns.trailing_zeros();
            let xpiece = get_xpiece(self.board, from - 9).unwrap();
            *pawns &= *pawns - 1;

            return Some(Move::new(from, from - 9, 1, xpiece, PROMO_CAPS[piece]));
        }

        None
    }

    fn next_ep(&mut self) -> Option<Move> {
        self.down_right_ep
            .take()
            .and_then(|ep| Some(Move::new(ep + 7, ep, 1, 0, MoveType::Ep)))
            .or_else(|| {
                self.down_left_ep
                    .take()
                    .and_then(|ep| Some(Move::new(ep + 9, ep, 1, 0, MoveType::Ep)))
            })
    }
}

impl<'a> Iterator for BlackPawnAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_down_right_attack()
            .or_else(|| self.next_down_left_attack())
            .or_else(|| self.next_down_right_promo_attack())
            .or_else(|| self.next_down_left_promo_attack())
            .or_else(|| self.next_ep())
    }
}

fn gen_battack_moves(
    b: &Board,
    pawns: u64,
    opp: u64,
    file: u64,
    shift: usize,
    target: u64,
) -> (u64, Option<[u64; 4]>, Option<u32>) {
    let pawns_to_move = (pawns & !file) & (opp << shift);

    let attacks = pawns_to_move & !R2;
    let promo_pieces = pawns_to_move & R2;
    let promo = if promo_pieces == 0 {
        None
    } else {
        Some([promo_pieces, promo_pieces, promo_pieces, promo_pieces])
    };

    let ep_sq = b.ep as u32;
    let ep = if ep_sq < 64 && SQUARES[ep_sq as usize] & ((pawns & !file) >> shift) & target >> 8 > 0
    {
        Some(ep_sq)
    } else {
        None
    };

    (attacks, promo, ep)
}
#[test]
fn black_pawn_moves() {
    crate::init();
    let mut b = Board::new();
    b.ctm ^= 1;

    let quiet = BlackPawnQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    let attack = BlackPawnQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);

    quiet.chain(attack).for_each(|m| println!("{m}"));
}
