use std::cmp::max;
use rand::prelude::*;
use crate::board::{Board, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::eval::PIECE_VALUES;
use crate::move_info::{FA, FH, R2, R7, SQUARES};
use crate::moves::{HTable, KillerMoves, Move, MoveType, PrevMoves};
// use crate::move_scorer::{CAP_SCORE_OFFSET, MoveScorer};
use crate::move_tables::MT;


const ALL_SQUARES: u64 = u64::MAX;
pub const NO_SQUARES: u64 = 0;

const ALL_CAP: usize = 218;
const CHECK_CAP: usize = 75;
const ATTACK_CAP: usize = 100;

const BEST_MOVE_SCORE: i32 = CAP_SCORE_OFFSET*2;
pub const CAP_SCORE_OFFSET: i32 = 100000;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MoveSet {
    All,
    Attacks,
    Check,
    Random
}

impl MoveSet {
    pub fn get_move_set(normal: MoveSet, board: &Board) -> MoveSet {
        if is_in_check(board) { MoveSet::Check } else { normal }
    }
}

// TODO turn movelist into an iterator that chunk generates moves (ie attacks then quiets)
// in the iterator, have it filter the illegal moves and request the next chunk (ie call gen_quiet)
// when needed

pub struct MoveList <'a>{
    pub moves: Vec<Move>,
    move_scores: Option<Vec<i32>>,
    move_set: MoveSet,
    board: &'a Board,
}

impl <'a> MoveList <'a> {
    pub fn new(
        board: &'a Board,
        move_capacity: usize,
        move_set: MoveSet,
        // score_utils: Option<(&'a KillerMoves, &'a PrevMoves, Option<Move>, usize)>,
        scored: bool,
        // best_move: Option<Move>,
        // km: Option<&'a KillerMoves>,
        // depth: usize,
    ) -> MoveList {
        MoveList {
            moves: Vec::with_capacity(move_capacity),
            move_scores: if scored { Some(Vec::with_capacity(move_capacity)) } else { None },
            move_set,
            board,
        }
    }

    // pub fn get_moves<H>(
    pub fn get_moves(
        board: &'a Board,
        move_set: MoveSet,
        km: &KillerMoves,
        best_move: Option<Move>,
        depth: usize,
        // hh: &H
    ) -> MoveList <'a> {
        // ) -> MoveList <'a> where H: HTable {

        let mut ml = match move_set {
            MoveSet::All | MoveSet::Random =>
                MoveList::new(board, ALL_CAP, move_set, true)
                    .gen_attacks(NO_SQUARES, ALL_SQUARES, false)
                    .gen_quiet(NO_SQUARES, ALL_SQUARES, false),

            MoveSet::Attacks =>
                MoveList::new(board, ATTACK_CAP, move_set, true)
                    .gen_attacks(NO_SQUARES, ALL_SQUARES, false),

            MoveSet::Check =>
                MoveList::new(board, CHECK_CAP, move_set, true)
                    .gen_check(),
        };

        if move_set == MoveSet::Random {
            (0..ml.moves.len()).for_each(|_| ml.move_scores.as_mut().unwrap().push(0));
        } else {
            // score moves
            ml.moves.iter()
                .for_each(|m| ml.move_scores.as_mut().unwrap()
                    // .push(score_move(board, *m, km, best_move, depth, hh)));
                .push(score_move(board, *m, km, best_move, depth)));
        }

        ml
    }

    pub fn get_moves_unscored(board: &'a Board, move_set: MoveSet) -> MoveList<'a> {
        match move_set {
            MoveSet::All | MoveSet::Random =>
                MoveList::new(board, ALL_CAP, move_set, false)
                    .gen_attacks(NO_SQUARES, ALL_SQUARES, false)
                    .gen_quiet(NO_SQUARES, ALL_SQUARES, false),
            MoveSet::Attacks =>
                MoveList::new(board, ATTACK_CAP, move_set, false)
                    .gen_attacks(NO_SQUARES, ALL_SQUARES, false),

            MoveSet::Check =>
                MoveList::new(board, CHECK_CAP, move_set, false)
                    .gen_check(),
        }
    }

    fn gen_attacks(mut self, pinned: u64, target: u64, check: bool) -> MoveList<'a> {
        if self.board.ctm == 0 {
            self.white_pawn_attack(pinned, target)
        } else {
            self.black_pawn_attack(pinned, target)
        };

        self.knight_attack(pinned, target)
            .rook_attack(pinned, target)
            .bishop_attack(pinned, target)
            .queen_attack(pinned, target);

        if !check { self.king_attack(); }
        self
    }

    fn gen_quiet(mut self, pinned: u64, target: u64, check: bool) -> MoveList<'a> {
        if self.board.ctm == 0 {
            self.white_pawn_quiet(pinned, target)
        } else {
            self.black_pawn_quiet(pinned, target)
        };

        self.knight_quiet(pinned, target)
            .rook_quiet(pinned, target)
            .bishop_quiet(pinned, target)
            .queen_quiet(pinned, target);

        if !check { self.king_quiet().king_castle(); }
        self
    }

    fn gen_check(mut self) -> MoveList<'a> {
        // gen all legal king moves
        self.king_in_check();
        let ksq = self.board.pieces[KING+self.board.ctm].trailing_zeros() as usize;
        let attackers = self.get_attackers(ksq);
        // if double check than only king moves matter
        if attackers.count_ones() >= 2 {
            return self;
        }

        let pinned_pieces = self.get_pinned_pieces();
        // try to cap the pinner
        self = self.gen_attacks(pinned_pieces, attackers, true);

        // try and move in the way of the attacker and the sliding piece
        let attack_piece = get_xpiece(self.board, attackers.trailing_zeros()).unwrap();
        // return if attacker is not a sliding piece
        if attack_piece < ROOK as u32 && attack_piece < KING as u32 { return self; }

        let asq = attackers.trailing_zeros() as usize;
        let inbetween = self.get_ray_inbetween(ksq, asq);

        // print_bb!(pinned_pieces, attackers, inbetween);

        self.gen_quiet(pinned_pieces, inbetween, true)
    }

    // pinned: all the pieces that are pinned by an attacker
    // target: all the squares that the pawns could move to
    fn white_pawn_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let pawns = self.board.pieces[0] & !pinned;
        let occ = self.board.util[2] | !target;
        let push = pawns & !(occ >> 8);
        let double = (pawns & R2) & !(occ >> 16) & !(self.board.util[2] >> 8);

        self.add_pawn_quiet(push & !R7, 8, MoveType::Quiet)
            .add_pawn_quiet(double, 16, MoveType::Double)
            .add_pawn_quiet_promo(push & R7, 8)
    }

    fn white_pawn_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let pawns = self.board.pieces[0] & !pinned;
        let opp = self.board.util[1] & target;
        let up_lefts = (pawns & !FA) & (opp >> 7);
        let up_rights = (pawns & !FH) & (opp >> 9);

        // print_bb!(pawns, opp, up_lefts, up_rights);

        self.add_pawn_attack(up_lefts & !R7, 7)
            .add_pawn_attack_promo(up_lefts & R7, 7)
            .add_pawn_attack(up_rights & !R7, 9)
            .add_pawn_attack_promo(up_rights & R7, 9);

        let ep = self.board.ep as u32;
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FA) << 7) & target << 8 > 0 {
            self.add_move(Move::new(ep - 7, ep, 0, 1, MoveType::Ep));
        }
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FH) << 9) & target << 8 > 0 {
            self.add_move(Move::new(ep - 9, ep, 0, 1, MoveType::Ep));
        }
        self
    }

    fn black_pawn_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let pawns = self.board.pieces[1] & !pinned;
        let occ = self.board.util[2] | !target;
        let push = pawns & !(occ << 8);
        let double = (pawns & R7) & !(occ << 16) & !(self.board.util[2] << 8);

        self.add_pawn_quiet(push & !R2, -8, MoveType::Quiet)
            .add_pawn_quiet(double, -16, MoveType::Double)
            .add_pawn_quiet_promo(push & R2, -8)
    }

    fn black_pawn_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let pawns = self.board.pieces[1] & !pinned;
        let opp = self.board.util[0] & target;
        let down_rights = (pawns & !FH) & (opp << 7);
        let down_lefts = (pawns & !FA) & (opp << 9);

        self.add_pawn_attack(down_rights & !R2, -7)
            .add_pawn_attack_promo(down_rights & R2, -7)
            .add_pawn_attack(down_lefts & !R2, -9)
            .add_pawn_attack_promo(down_lefts & R2, -9);

        let ep = self.board.ep as u32;
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FH) >> 7) & target >> 8 > 0 {
            self.add_move(Move::new(ep + 7, ep, 1, 0, MoveType::Ep));
        }
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FA) >> 9) & target >> 8 > 0 {
            self.add_move(Move::new(ep + 9, ep, 1, 0, MoveType::Ep));
        }
        self
    }

    fn knight_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 2+self.board.ctm;
        let mut knights = self.board.pieces[piece] & !pinned;
        while knights > 0 {
            let from = knights.trailing_zeros();
            let quiet = MT::knight_moves(from as usize) & !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            knights &= knights-1;
        }
        self
    }

    fn knight_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 2+self.board.ctm;
        let mut knights = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.ctm ^1] & target;
        while knights > 0 {
            let from = knights.trailing_zeros();
            let attacks = MT::knight_moves(from as usize) & opp;
            self.add_attack(attacks, from, piece as u32);
            knights &= knights-1;
        }
        self
    }


    // TODO maybe a way to cache rook and bishop moves and only query them once or maybe its not worth it
    fn rook_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 4+self.board.ctm;
        let mut rooks = self.board.pieces[piece] & !pinned;
        while rooks > 0 {
            let from = rooks.trailing_zeros();
            let mut quiet = MT::rook_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            rooks &= rooks-1;
        }
        self
    }

    fn rook_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 4+self.board.ctm;
        let mut rooks = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.ctm ^1] & target;
        while rooks > 0 {
            let from = rooks.trailing_zeros();
            let attack = MT::rook_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            rooks &= rooks-1;
        }
        self
    }

    fn bishop_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 6+self.board.ctm;
        let mut bishops = self.board.pieces[piece] & !pinned;
        while bishops > 0 {
            let from = bishops.trailing_zeros();
            let mut quiet = MT::bishop_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            bishops &= bishops -1;
        }
        self
    }

    fn bishop_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 6+self.board.ctm;
        let mut bishops = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.ctm ^1] & target;
        while bishops > 0 {
            let from = bishops.trailing_zeros();
            let attack = MT::bishop_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            bishops &= bishops -1;
        }
        self
    }

    fn queen_quiet(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 8+self.board.ctm;
        let mut queens = self.board.pieces[piece] & !pinned;
        while queens > 0 {
            let from = queens.trailing_zeros();
            let mut quiet = MT::bishop_moves(self.board.util[2], from as usize);
            quiet |= MT::rook_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;

            self.add_quiet(quiet, from, piece as u32);
            queens &= queens -1;
        }
        self
    }

    fn queen_attack(&mut self, pinned: u64, target: u64) -> &mut MoveList<'a> {
        let piece = 8+self.board.ctm;
        let mut queens = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.ctm ^1] & target;
        while queens > 0 {
            let from = queens.trailing_zeros();
            let mut attack = MT::bishop_moves(self.board.util[2], from as usize);
            attack |= MT::rook_moves(self.board.util[2], from as usize);
            attack &= opp;
            self.add_attack(attack, from, piece as u32);
            queens &= queens -1;
        }
        self
    }

    fn king_quiet(&mut self) -> &mut MoveList<'a> {
        let piece = 10+self.board.ctm;
        let from = self.board.pieces[piece].trailing_zeros();
        let quiet = MT::king_moves(from as usize) & !self.board.util[2];
        self.add_quiet(quiet, from, piece as u32)
    }

    fn king_attack(&mut self) -> &mut MoveList<'a> {
        let piece = 10+self.board.ctm;
        let from = self.board.pieces[piece].trailing_zeros();
        let attack = MT::king_moves(from as usize) & self.board.util[self.board.ctm ^1];
        self.add_attack(attack, from, piece as u32)
    }

    fn king_castle(&mut self) -> &mut MoveList<'a> {
        let piece = 10+self.board.ctm;
        let from = self.board.pieces[piece].trailing_zeros();
        let rights = self.board.castle_state >> (2*(self.board.ctm ^1));
        let kingside = rights & 0b10;
        let queenside = rights & 1;

        if kingside > 0 && self.board.util[2] & (0x60<<(self.board.ctm *56)) == 0 {
            let move_type = MoveType::kingside(self.board.ctm);
            self.add_move(Move::new(from,from+2,piece as u32,0, move_type))
        }

        if queenside > 0 && self.board.util[2] & (0xE<<(self.board.ctm *56)) == 0 {
            let move_type = MoveType::queenside(self.board.ctm);
            self.add_move(Move::new(from,from-2,piece as u32,0, move_type))
        }
        self
    }

    fn king_in_check(&mut self) -> &mut MoveList<'a> {
        let from = self.board.pieces[KING + self.board.ctm].trailing_zeros();
        let occ = self.board.util[2] & !self.board.pieces[KING + self.board.ctm];

        let mut possible = MT::king_moves(from as usize) & !self.board.util[self.board.ctm];
        let opp_colour = self.board.ctm ^1;
        // get opp pawn attacks
        possible &= if opp_colour == 0 {
            !(((self.board.pieces[0] & !FA) << 7) | ((self.board.pieces[0] & !FH) << 9))
        } else {
            !(((self.board.pieces[1] & !FH) >> 7) | ((self.board.pieces[1] & !FA) >> 9))
        };
        // get opp king moves
        possible &= !(MT::king_moves(self.board.pieces[KING + opp_colour].trailing_zeros() as usize));

        // get opp knight moves
        let mut knights = self.board.pieces[KNIGHT + opp_colour];
        while knights > 0 {
            possible &= !(MT::knight_moves(knights.trailing_zeros() as usize));
            knights &= knights - 1;
        }
        // rook/queen
        let mut rook_queen = self.board.pieces[ROOK + opp_colour] | self.board.pieces[QUEEN + opp_colour];
        while rook_queen > 0 {
            possible &= !(MT::rook_moves(occ, rook_queen.trailing_zeros() as usize));
            rook_queen &= rook_queen - 1
        }
        // bishop/queen
        let mut bishop_queen = self.board.pieces[BISHOP + opp_colour] | self.board.pieces[QUEEN + opp_colour];
        while bishop_queen > 0 {
            possible &= !(MT::bishop_moves(occ, bishop_queen.trailing_zeros() as usize));
            bishop_queen &= bishop_queen - 1
        }

        let quiet = possible & !self.board.util[2];
        let attack = possible & self.board.util[opp_colour];
        let piece = (KING+self.board.ctm) as u32;
        self.add_quiet(quiet, from, piece).add_attack(attack, from, piece)
    }

    fn add_pawn_quiet(
        &mut self,
        mut pawns: u64,
        to_diff: i32,
        move_type: MoveType
    ) -> &mut MoveList<'a> {
        let piece = self.board.ctm as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            self.add_move(Move::new(from, to,piece,0,move_type));
            pawns &= pawns-1;
        }
        self
    }

    fn add_pawn_quiet_promo(&mut self, mut pawns: u64, to_diff: i32) -> &mut MoveList<'a> {
        let piece = self.board.ctm as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            for xpiece in [QUEEN as u32,KNIGHT as u32,ROOK as u32,BISHOP as u32] {
                self.add_move(
                    Move::new(from, to, piece, xpiece + piece, MoveType::Promo)
                );
            }
            pawns &= pawns-1;
        }
        self
    }

    fn add_pawn_attack(&mut self, mut pawns: u64, to_diff: i32) -> &mut MoveList<'a> {
        let piece = self.board.ctm as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            let xpiece = get_xpiece(self.board, to).unwrap();
            self.add_move(Move::new(from, to, piece, xpiece, MoveType::Cap));
            pawns &= pawns-1;
        }
        self
    }

    fn add_pawn_attack_promo(&mut self, mut pawns: u64, to_diff: i32) -> &mut MoveList<'a> {
        let piece = self.board.ctm as u32;

        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            let xpiece = get_xpiece(self.board, to).unwrap();
            for promo_cap in [MoveType::QPromoCap,MoveType::RPromoCap,MoveType::NPromoCap,MoveType::BPromoCap] {
                self.add_move(Move::new(from, to, piece, xpiece, promo_cap));
            }
            pawns &= pawns-1;
        }
        self
    }

    fn add_quiet(&mut self, mut quiet: u64, from: u32, piece: u32) -> &mut MoveList<'a> {
        while quiet > 0 {
            let to = quiet.trailing_zeros();
            self.add_move(Move::new(from, to, piece, 0, MoveType::Quiet));
            quiet &= quiet-1;
        }
        self
    }

    fn add_attack(&mut self, mut attack: u64, from: u32, piece: u32) -> &mut MoveList<'a> {
        while attack > 0 {
            let to = attack.trailing_zeros();
            let xpiece = get_xpiece(self.board, to).unwrap();
            self.add_move(Move::new(from, to, piece, xpiece, MoveType::Cap));
            attack &= attack-1;
        }
        self
    }

    fn get_attackers(&self, sq: usize) -> u64 {
        let mut attackers = 0;
        let colour_to_move = self.board.ctm ^1;
        let pawns = self.board.pieces[PAWN + colour_to_move];
        attackers |= MT::pawn_attacks(colour_to_move ^ 1, sq) & pawns;

        let knights = self.board.pieces[KNIGHT + colour_to_move];
        attackers |= MT::knight_moves(sq) & knights;

        let king = self.board.pieces[KING + colour_to_move];
        attackers |= MT::king_moves(sq) & king;

        let bishop_queen = self.board.pieces[QUEEN + colour_to_move] | self.board.pieces[BISHOP + colour_to_move];
        attackers |= MT::bishop_moves(self.board.util[2], sq) & bishop_queen;

        let rook_queen = self.board.pieces[ROOK + colour_to_move] | self.board.pieces[QUEEN + colour_to_move];
        attackers |= MT::rook_moves(self.board.util[2], sq) & rook_queen;

        attackers
    }

    fn get_pinned_pieces(&self) -> u64 {
        let ksq = self.board.pieces[KING+self.board.ctm].trailing_zeros() as usize;

        let mut rq = self.board.pieces[ROOK + (self.board.ctm ^ 1)];
        rq |= self.board.pieces[QUEEN + (self.board.ctm ^ 1)];

        let rq_pinners = rq & MT::rook_xray_moves(
            self.board.util[2],
            self.board.util[self.board.ctm],
            ksq
        );

        let mut bq = self.board.pieces[BISHOP + (self.board.ctm ^ 1)];
        bq |= self.board.pieces[QUEEN + (self.board.ctm ^ 1)];

        let bq_pinners = bq & MT::bishop_xray_moves(
            self.board.util[2],
            self.board.util[self.board.ctm],
            ksq
        );

        let mut pinners = rq_pinners | bq_pinners;
        let mut pinned_pieces= 0;

        while pinners > 0 {
            let p_sq = pinners.trailing_zeros() as usize;

            pinned_pieces |= self.board.util[2] & self.get_ray_inbetween(ksq, p_sq);
            pinned_pieces |= self.board.util[2] & self.get_ray_inbetween(ksq, p_sq);

            pinners &= pinners-1;
        }

        pinned_pieces
    }

    fn get_ray_inbetween(&self, sq1: usize, sq2: usize) -> u64 {
        let (higher, lower) = if sq1 > sq2 { (sq1, sq2) } else { (sq2, sq1 ) };

        let mut dir = 12;
        for d in 0..4 {
            if SQUARES[higher] & MT::rays(d, lower) > 0 {
                dir = d;
                break;
            }
        }

        MT::rays(dir,lower) & (SQUARES[higher]-1)
    }

    #[inline]
    fn add_move(&mut self, m: Move) {
        self.moves.push(m);

    }

    fn pop_best_move(&mut self) -> Option<Move> {
        let mut best_idx = None;
        let mut best_score = i32::MIN;

        for (i, score) in self.move_scores.as_ref().unwrap().iter().enumerate() {
            if score > &best_score {
                best_idx = Some(i);
                best_score = *score;
            }
        }

        best_idx.and_then(|best_idx| {
            self.move_scores.as_mut().unwrap()[best_idx] = i32::MIN;
            Some(self.moves[best_idx])
        })
    }

    fn pop_rand_move(&mut self) -> Option<Move> {
        let mut rng = thread_rng();

        let start_idx = rng.gen_range(0..self.moves.len());
        let mut idx = start_idx;
        // find the next random enough move by stepping left (and wrapping) until a unsearched move is found
        while self.move_scores.as_ref().unwrap()[idx] == i32::MIN {
            idx = (idx + 1) % self.moves.len();
            if idx == start_idx { return None; } // if done a loop return nothing
        }

        self.move_scores.as_mut().unwrap()[idx] = i32::MIN;
        Some(self.moves[idx])
    }
}

#[test]
fn random_moves() {
    crate::init();
    let board = Board::new();

    let move_length = 5;

    let ml = MoveList {
        moves: (0..move_length).map(|m| Move::_new_from_u32(m)).collect(),
        move_scores: Some( (0..move_length).map(|_| 0).collect() ),
        move_set: MoveSet::Random,
        board: &board
    };

    let mut rand_moves = Vec::new();
    for m in ml {
        println!("{m}");
        rand_moves.push(m);
    }
    assert_eq!(rand_moves.len(), move_length as usize)
}


impl <'a> Iterator for MoveList<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        // while get next move is some:
            // make this move into board,
            // if legal move return this board
            //otherwise continue the loop
//TODO if there are no more moves, check if any more moves need to be generated
        // if so, (after generating and scoring) loop like above untill a move is found
        // otherwise return none

        // if let Some(m) = self.pop_best_move() { return Some(m); }
        if self.move_set == MoveSet::Random {
            self.pop_rand_move()
            // self.pop_best_move()
        } else {
            self.pop_best_move()
        }
        // None
    }
}

pub fn get_piece(board: &Board, sq: u32) -> Option<u32> {
    let bb_sq = SQUARES[sq as usize];
    let s = board.ctm as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces[p as usize] > 0 { return Some(p); }
    }

    None
}

pub fn get_xpiece(board: &Board, sq: u32) -> Option<u32> {
    let bb_sq = SQUARES[sq as usize];
    // let start = (board.colour_to_move^1) as u32;
    let s = (board.ctm ^1) as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces[p as usize] > 0 { return Some(p); }
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

fn get_attackers(board: &Board, sq: usize, attacker_colour: usize) -> u64 {
    let mut attackers = MT::pawn_attacks(attacker_colour^1, sq) & board.pieces[PAWN+attacker_colour];
    attackers |= MT::knight_moves(sq) & board.pieces[KNIGHT+attacker_colour];
    attackers |= MT::king_moves(sq) & board.pieces[KING+attacker_colour];
    let mut bq = MT::bishop_moves(board.util[2], sq);
    bq &= board.pieces[BISHOP+attacker_colour] | board.pieces[QUEEN+attacker_colour];
    attackers |= bq;
    let mut rq = MT::rook_moves(board.util[2], sq);
    rq &= board.pieces[ROOK+attacker_colour] | board.pieces[QUEEN+attacker_colour];
    attackers |= rq;

    attackers
}

#[inline]
pub fn moved_into_check(board: &Board, m: Move) -> bool {
    let ksq = board.pieces[KING + (board.ctm ^ 1)].trailing_zeros() as usize;
    SQUARES[m.from() as usize] & MT::superrays(ksq) > 0
        && sq_attacked(board, ksq, board.ctm)
}

pub fn is_in_check(board: &Board) -> bool {
    sq_attacked(
        board,
        board.pieces[KING+board.ctm].trailing_zeros() as usize,
        board.ctm ^1
    )
}

// assumes the board has not been added to prev_moves, so checks if the count is 2
// (as adding the board would make it 3 and therefore three move repetition)
pub fn is_legal_move(board: &Board, m: Move, prev_moves: &PrevMoves) -> bool {
    if board.halfmove > 100 || prev_moves.get_count(board.hash) == 2 { return false; }

    match m.move_type() {
        // check castle moves to see if the king passes through an attacked square
        MoveType::WKingSide => !sq_attacked(board, 5, 1)
            & !sq_attacked(board, 6, 1),
        MoveType::WQueenSide => !sq_attacked(board, 3, 1)
            & !sq_attacked(board, 2, 1),
        MoveType::BKingSide => !sq_attacked(board,  61, 0)
            & !sq_attacked(board, 62, 0),
        MoveType::BQueenSide => !sq_attacked(board,  59, 0)
            & !sq_attacked(board, 58, 0),
        _ => true,
    }
}


fn score_move(
    // fn score_move<H>(
        board: &Board,
        m: Move,
        km: &KillerMoves,
        best_move: Option<Move>,
        depth: usize,
        // _hh: &H
    ) -> i32 {
        if let Some(best_move) = best_move {
            if best_move == m { return BEST_MOVE_SCORE; }
        } else if let Some(score) = km.get_move_score(m, depth) {
            return score
        }

        match m.move_type() {
            MoveType::Quiet | MoveType::Double | MoveType::WKingSide | MoveType::BKingSide |
            MoveType::WQueenSide | MoveType::BQueenSide | MoveType::Promo | MoveType::Ep => {
                if let Some(score) = km.get_move_score(m, depth) {
                    score
                } else {
                    // hh.get(board.colour_to_move, m.from() as usize, m.to() as usize) as i32
                    // PST[m.piece() as usize][m.to() as usize] as i32
                    m.piece() as i32
                }


            }

            MoveType::Cap | MoveType::NPromoCap | MoveType::RPromoCap | MoveType::BPromoCap |
            MoveType::QPromoCap =>
                see(board, m, depth) + CAP_SCORE_OFFSET

        }
    }

fn see_get_least_valuable(board: &Board, attackers: u64, board_depth: usize) -> (usize, u64) {
    let colour = board.ctm ^ (board_depth & 1);
    let piece_iter = board.pieces.iter()
        .enumerate()
        .skip(colour)
        .step_by(2);

    for (piece, pieces) in piece_iter {
        let p_in_attackers = *pieces & attackers;
        if p_in_attackers > 0 {
            return (piece, p_in_attackers & p_in_attackers.wrapping_neg());
        }
    }

    (12, NO_SQUARES)
}

fn see(board: &Board, m: Move, board_depth: usize) -> i32 {
    // trying to understand the https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm

    let mut gain: [i32; 32] = [0;32];
    let mut depth = 0;

    let (from, to, mut piece, xpiece, _) = m.all();

    // froms is a bb of the next possible piece to move that attacks the to square
    let mut from_piece = SQUARES[from];
    let mut occ = board.util[2];
    let mut attackers = get_all_attackers(board,to);

    // can_xray is just all pieces that arent knights
    // as there is no sliding piece that can be behind a knight that could attack the target
    let can_xray = occ ^ board.pieces[KNIGHT] ^ board.pieces[KNIGHT+1];

    gain[depth] = PIECE_VALUES[xpiece];

    while from_piece > 0 {
        depth += 1;

        // add this score into the and cut off if it cannot increase the score
        gain[depth] = PIECE_VALUES[piece] - gain[depth-1];
        if max(-gain[depth-1], gain[depth]) < 0 { break; }

        // remove this attacker
        attackers ^= from_piece;
        occ ^= from_piece;

        // recheck if there are any sliding pieces behind this attacker
        if from_piece & can_xray > 0 {
            attackers |= occ & get_all_attackers(board, to);
        }

        (piece, from_piece) = see_get_least_valuable(board, attackers, board_depth);
    }

    // iterate over all the stored gain values to find the max - negamax style
    for i in (1..depth).rev() {
        gain[i-1] = -max(-gain[i-1], gain[i]);
    }

    gain[0]
}