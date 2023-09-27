use crate::board::board::{Board, ALL_PIECES, BISHOP, BLACK, KING, QUEEN, ROOK};
use crate::movegen::move_info::{FA, FH, R2, R7, R8, SQUARES};
use crate::movegen::movegen::{get_xpiece, ALL_SQUARES, NO_SQUARES};
use crate::movegen::movegen_iters::MovegenIterator;
use crate::movegen::moves::{Move, MoveType};

const PROMO_PIECES: [u32; 4] = [QUEEN as u32, KING as u32, ROOK as u32, BISHOP as u32];
const PROMO_CAPS: [MoveType; 4] = [
    MoveType::QPromoCap,
    MoveType::RPromoCap,
    MoveType::NPromoCap,
    MoveType::BPromoCap,
];

pub struct WhitePawnQuietIterator {
    quiet: u64,
    double: u64,
    promo: Option<[u64; 4]>,
}

impl WhitePawnQuietIterator {
    pub fn new(b: &Board, pinned: u64, target: u64) -> WhitePawnQuietIterator {
        if b.ctm == 1 {
            return WhitePawnQuietIterator {
                quiet: 0,
                double: 0,
                promo: None,
            };
        }

        let pawns = b.pieces[0] & !pinned;
        let occ = b.util[ALL_PIECES] | !target;

        let (quiet, double, promo_pieces) = gen_wquiet_moves(b, pawns, occ);

        let promo = if promo_pieces == 0 {
            None
        } else {
            Some([promo_pieces, promo_pieces, promo_pieces, promo_pieces])
        };

        WhitePawnQuietIterator {
            quiet,
            double,
            promo,
        }
    }

    fn next_quiet(&mut self) -> Option<Move> {
        if self.quiet == 0 {
            return None;
        }

        let from = self.quiet.trailing_zeros();
        self.quiet &= self.quiet - 1;

        Some(Move::new(from, from + 8, 0, 0, MoveType::Quiet))
    }

    fn next_double(&mut self) -> Option<Move> {
        if self.double == 0 {
            return None;
        }

        let from = self.double.trailing_zeros();
        self.double &= self.double - 1;

        Some(Move::new(from, from + 16, 0, 0, MoveType::Double))
    }

    fn next_promo(&mut self) -> Option<Move> {
        if self.promo.is_none() {
            return None;
        }

        let promos = self.promo.as_mut().unwrap();

        for (piece, pawns) in promos.iter_mut().enumerate() {
            if *pawns == 0 {
                continue;
            }

            let from = pawns.trailing_zeros();
            *pawns &= *pawns - 1;

            return Some(Move::new(
                from,
                from + 8,
                0,
                PROMO_PIECES[piece],
                MoveType::Promo,
            ));
        }

        None
    }
}

impl Iterator for WhitePawnQuietIterator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_quiet()
            .or_else(|| self.next_double())
            .or_else(|| self.next_promo())
    }
}

fn gen_wquiet_moves(b: &Board, pawns: u64, occ: u64) -> (u64, u64, u64) {
    let quiet = pawns & !(occ >> 8);
    let push = quiet & !R7;
    let double = (pawns & R2) & !(occ >> 16) & !(b.util[ALL_PIECES] >> 8);
    let promo = quiet & R7;

    (push, double, promo)
}

pub struct WhitePawnAttackIterator<'a> {
    board: &'a Board,
    up_left: u64,
    up_left_promo: Option<[u64; 4]>,
    up_left_ep: Option<u32>,
    up_right: u64,
    up_right_promo: Option<[u64; 4]>,
    up_right_ep: Option<u32>,
}

impl<'a> WhitePawnAttackIterator<'a> {
    pub fn new(b: &'a Board, pinned: u64, target: u64) -> WhitePawnAttackIterator<'a> {
        if b.ctm == 1 {
            return WhitePawnAttackIterator {
                board: b,
                up_left: 0,
                up_left_promo: None,
                up_left_ep: None,
                up_right: 0,
                up_right_promo: None,
                up_right_ep: None,
            };
        }

        let pawns = b.pieces[0] & !pinned;
        let opp = b.util[BLACK] & target;

        let (left, left_promo, left_ep) = gen_wattack_moves(b, pawns, opp, FA, 7, target);
        let (right, right_promo, right_ep) = gen_wattack_moves(b, pawns, opp, FH, 9, target);

        WhitePawnAttackIterator {
            board: b,
            up_left: left,
            up_left_promo: left_promo,
            up_left_ep: left_ep,
            up_right: right,
            up_right_promo: right_promo,
            up_right_ep: right_ep,
        }
    }

    fn next_up_left_attack(&mut self) -> Option<Move> {
        if self.up_left == 0 {
            return None;
        }

        let from = self.up_left.trailing_zeros();
        let xpiece = get_xpiece(self.board, from + 7).unwrap();
        self.up_left &= self.up_left - 1;
        Some(Move::new(from, from + 7, 0, xpiece, MoveType::Cap))
    }

    fn next_up_right_attack(&mut self) -> Option<Move> {
        if self.up_right == 0 {
            return None;
        }

        let from = self.up_right.trailing_zeros();
        let xpiece = get_xpiece(self.board, from + 9).unwrap();
        self.up_right &= self.up_right - 1;
        Some(Move::new(from, from + 9, 0, xpiece, MoveType::Cap))
    }

    fn next_up_left_promo_attack(&mut self) -> Option<Move> {
        if self.up_left_promo.is_none() {
            return None;
        }

        let promos = self.up_left_promo.as_mut().unwrap();

        for (piece, pawns) in promos.iter_mut().enumerate() {
            if *pawns == 0 {
                continue;
            }

            let from = pawns.trailing_zeros();
            let xpiece = get_xpiece(self.board, from + 7).unwrap();
            *pawns &= *pawns - 1;

            return Some(Move::new(from, from + 7, 0, xpiece, PROMO_CAPS[piece]));
        }

        None
    }

    fn next_up_right_promo_attack(&mut self) -> Option<Move> {
        if self.up_right_promo.is_none() {
            return None;
        }

        let promos = self.up_right_promo.as_mut().unwrap();

        for (piece, pawns) in promos.iter_mut().enumerate() {
            if *pawns == 0 {
                continue;
            }

            let from = pawns.trailing_zeros();
            let xpiece = get_xpiece(self.board, from + 9).unwrap();
            *pawns &= *pawns - 1;

            return Some(Move::new(from, from + 9, 0, xpiece, PROMO_CAPS[piece]));
        }

        None
    }

    fn next_ep(&mut self) -> Option<Move> {
        self.up_left_ep
            .take()
            .and_then(|ep| Some(Move::new(ep - 7, ep, 0, 1, MoveType::Ep)))
            .or_else(|| {
                self.up_right_ep
                    .take()
                    .and_then(|ep| Some(Move::new(ep - 9, ep, 0, 1, MoveType::Ep)))
            })
    }
}

impl<'a> Iterator for WhitePawnAttackIterator<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_up_left_attack()
            .or_else(|| self.next_up_right_attack())
            .or_else(|| self.next_up_left_promo_attack())
            .or_else(|| self.next_up_right_promo_attack())
            .or_else(|| self.next_ep())
    }
}

fn gen_wattack_moves(
    b: &Board,
    pawns: u64,
    opp: u64,
    file: u64,
    shift: usize,
    target: u64,
) -> (u64, Option<[u64; 4]>, Option<u32>) {
    let pawns_to_move = (pawns & !file) & (opp >> shift);

    let attacks = pawns_to_move & !R7;
    let promo_pieces = pawns_to_move & R7;
    let promo = if promo_pieces == 0 {
        None
    } else {
        Some([promo_pieces, promo_pieces, promo_pieces, promo_pieces])
    };

    let ep_sq = b.ep as u32;
    let ep = if ep_sq < 64 && SQUARES[ep_sq as usize] & ((pawns & !file) << shift) & target << 8 > 0
    {
        Some(ep_sq)
    } else {
        None
    };

    (attacks, promo, ep)
}

#[test]
fn white_pawn_moves() {
    crate::init();
    let b = Board::new();

    // let quiet = WhitePawnQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);
    // let attack = WhitePawnQuietIterator::new(&b, NO_SQUARES, ALL_SQUARES);

    // quiet.chain(attack).for_each(|m| println!("{m}"));
}
