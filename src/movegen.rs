use crate::board::{Board, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::move_info::{FA, FH, R1, R2, R3, R6, R7, R8, SQUARES};
// use crate::move_tables::MT;
use crate::moves::{KillerMoves, Move, PrevMoves};
use crate::{print_bb, MoveTables};
use std::cmp::{max, min};
use crate::eval::PIECE_VALUES;
use crate::tt::SeqTT;
// use crate::movegen::{gen_attacks, get_xpiece};

pub const QUIET: u32 = 0;
pub const DOUBLE: u32 = 1;
pub const CAP: u32 = 2;
pub const KINGSIDE: u32 = 3; // more readable in gen_king_castle
pub const WKINGSIDE: u32 = 3;
pub const BKINGSIDE: u32 = 4;
pub const QUEENSIDE: u32 = 5; // ^ readability
pub const WQUEENSIDE: u32 = 5;
pub const BQUEENSIDE: u32 = 6;
pub const PROMO: u32 = 7; // when promo is used, xpiece determines the promo piece type
pub const N_PROMO_CAP: u32 = 8;
pub const R_PROMO_CAP: u32 = 9;
pub const B_PROMO_CAP: u32 = 10;
pub const Q_PROMO_CAP: u32 = 11;
pub const EP: u32 = 12;

pub const MTYPE_STRS: [&str; 13] = ["QUIET", "DOUBLE", "CAP", "WKINGSIDE", "BKINGSIDE",
    "WQUEENSIDE", "BQUEENSIDE", "PROMO", "N_PROMO_CAP", "R_PROMO_CAP", "B_PROMO_CAP",
    "Q_PROMO_CAP", "EP"];


const ALL_SQUARES: u64 = u64::MAX;
const NO_SQUARES: u64 = 0;

pub const CAP_SCORE_OFFSET: i32 = 100000;

pub struct MoveList <'a> {
    pub moves: Vec<Move>,
    scores: Vec<i32>,
    board: &'a Board,
    mt: &'a MoveTables,
    best_move: Option<Move>,
}

impl <'a> MoveList <'a> {
    pub fn new(
        board: &'a Board,
        mt: &'a MoveTables,
        move_capacity: usize,
        score_capacity: usize,
    ) -> MoveList<'a> {
        MoveList {
            board,
            mt,
            moves: Vec::with_capacity(move_capacity),
            scores: Vec::with_capacity(score_capacity),
            best_move: None,
        }
    }

    pub fn all_scored(
        board: &'a Board,
        check: bool,
        mt: &'a MoveTables,
        tt: &SeqTT,
        km: &KillerMoves,
        depth: usize
    ) -> MoveList<'a> {
        let mut move_list;

        if check {
            move_list = MoveList::new(board, mt,75, 75);
            move_list.best_move = tt.get_best(board.hash);

            move_list.gen_check()
        } else {
            move_list = MoveList::new(board, mt,220, 220);
            move_list.best_move = tt.get_best(board.hash);

            move_list.gen_attacks(NO_SQUARES, ALL_SQUARES, false);
            move_list.gen_quiet(NO_SQUARES, ALL_SQUARES, false);
        };

        move_list.score_moves(km, depth);
        move_list
    }

    pub fn all(board: &'a Board, mt: &'a MoveTables, check: bool) -> MoveList<'a> {
        let mut move_list;
        if check {
            move_list = MoveList::new(board, mt,75, 0);
            move_list.gen_check()
        } else {
            move_list = MoveList::new(board, mt,220, 0);
            move_list.gen_attacks(NO_SQUARES, ALL_SQUARES, false);
            move_list.gen_quiet(NO_SQUARES, ALL_SQUARES, false);
        };

        move_list
    }

    pub fn attacks(board: &'a Board, mt: &'a MoveTables) -> MoveList<'a> {
        let mut move_list = MoveList::new(board, mt,100, 0);
        move_list.gen_attacks(NO_SQUARES, ALL_SQUARES, false);
        move_list
    }

    fn attacks_see(board: &'a Board, mt: &'a MoveTables, sq: u64) -> MoveList<'a> {
        let mut move_list = MoveList::new(board, mt,10, 0);
        move_list.gen_attacks(NO_SQUARES, sq, false);
        move_list
    }

    pub fn checks(board: &'a Board, mt: &'a MoveTables) -> MoveList <'a> {
        let mut move_list = MoveList::new(board, mt, 75, 0);
        move_list.gen_check();
        move_list
    }



    fn gen_attacks(&mut self, pinned: u64, target: u64, check: bool) {
        if self.board.colour_to_move == 0 {
            self.white_pawn_attack(pinned, target);
        } else {
            self.black_pawn_attack(pinned, target);
        }
        self.knight_attack(pinned, target);
        self.rook_attack(pinned, target);
        self.bishop_attack(pinned, target);
        self.queen_attack(pinned, target);
        if !check { self.king_attack(); }
    }

    fn gen_quiet(&mut self, pinned: u64, target: u64, check: bool) {
        if self.board.colour_to_move == 0 {
            self.white_pawn_quiet(pinned, target);
        } else {
            self.black_pawn_quiet(pinned, target);
        }
        self.knight_quiet(pinned, target);
        self.rook_quiet(pinned, target);
        self.bishop_quiet(pinned, target);
        self.queen_quiet(pinned, target);
        if !check {
            self.king_quiet();
            self.king_castle();
        }
    }

    fn gen_check(&mut self) {
        // gen all legal king moves
        self.king_in_check();

        let ksq = self.board.pieces[KING+self.board.colour_to_move].trailing_zeros() as usize;

        let attackers = self.get_attackers(ksq);
        // if double check than only king moves matter
        if attackers.count_ones() >= 2 {
            return;
        }

        let mut pinned_pieces = self.get_pinned_pieces();


        // try to cap the pinner
        self.gen_attacks(pinned_pieces, attackers, true);

        // try and move in the way of the attacker and the sliding piece
        let attack_piece = get_xpiece(self.board, attackers.trailing_zeros());
        // return if attacker is not a sliding piece
        if attack_piece < ROOK as u32 && attack_piece < KING as u32 { return }

        let asq = attackers.trailing_zeros() as usize;
        let inbetween = self.get_ray_inbetween(ksq, asq);

        // print_bb!(pinned_pieces, attackers, inbetween);

        self.gen_quiet(pinned_pieces, inbetween, true);
    }

    // pinned: all the pieces that are pinned by an attacker
    // target: all the squares that the pawns could move to
    fn white_pawn_quiet(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[0] & !pinned;
        let occ = self.board.util[2] | !target;
        let push = pawns & !(occ >> 8);
        let double = (pawns & R2) & !(occ >> 16) & !(self.board.util[2] >> 8);

        self.add_pawn_quiet(push & !R7, 8, QUIET);
        self.add_pawn_quiet(double, 16, DOUBLE);
        self.add_pawn_quiet_promo(push & R7, 8);
    }

    fn white_pawn_attack(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[0] & !pinned;
        let opp = self.board.util[1] & target;
        let up_lefts = (pawns & !FA) & (opp >> 7);
        let up_rights = (pawns & !FH) & (opp >> 9);

        // print_bb!(pawns, opp, up_lefts, up_rights);

        self.add_pawn_attack(up_lefts & !R7, 7);
        self.add_pawn_attack_promo(up_lefts & R7, 7);
        self.add_pawn_attack(up_rights & !R7, 9);
        self.add_pawn_attack_promo(up_rights & R7, 9);

        let ep = self.board.ep as u32;
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FA) << 7) & target << 8 > 0 {
            self.add_move(Move::new(ep - 7, ep, 0, 1, EP));
        }
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FH) << 9) & target << 8 > 0 {
            self.add_move(Move::new(ep - 9, ep, 0, 1, EP));
        }
    }

    fn black_pawn_quiet(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[1] & !pinned;
        let occ = self.board.util[2] | !target;
        let push = pawns & !(occ << 8);
        let double = (pawns & R7) & !(occ << 16) & !(self.board.util[2] << 8);

        self.add_pawn_quiet(push & !R2, -8, QUIET);
        self.add_pawn_quiet(double, -16, DOUBLE);
        self.add_pawn_quiet_promo(push & R2, -8);
    }

    fn black_pawn_attack(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[1] & !pinned;
        let opp = self.board.util[0] & target;
        let down_rights = (pawns & !FH) & (opp << 7);
        let mut down_lefts = (pawns & !FA) & (opp << 9);

        self.add_pawn_attack(down_rights & !R2, -7);
        self.add_pawn_attack_promo(down_rights & R2, -7);
        self.add_pawn_attack(down_lefts & !R2, -9);
        self.add_pawn_attack_promo(down_lefts & R2, -9);

        let ep = self.board.ep as u32;
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FH) >> 7) & target >> 8 > 0 {
            self.add_move(Move::new(ep + 7, ep, 1, 0, EP));
        }
        if ep < 64 && SQUARES[ep as usize] & ((pawns & !FA) >> 9) & target >> 8 > 0 {
            self.add_move(Move::new(ep + 9, ep, 1, 0, EP));
        }
    }

    fn knight_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 2+self.board.colour_to_move;
        let mut knights = self.board.pieces[piece] & !pinned;
        while knights > 0 {
            let from = knights.trailing_zeros();
            let quiet = self.mt.knight_moves[from as usize] & !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            knights &= knights-1;
        }
    }

    fn knight_attack(&mut self, pinned: u64, target: u64) {
        let piece = 2+self.board.colour_to_move;
        let mut knights = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.colour_to_move^1] & target;
        while knights > 0 {
            let from = knights.trailing_zeros();
            let attacks = self.mt.knight_moves[from as usize] & opp;
            self.add_attack(attacks, from, piece as u32);
            knights &= knights-1;
        }
    }


    // TODO maybe a way to cache rook and bishop moves and only query them once or maybe its not worth it
    fn rook_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 4+self.board.colour_to_move;
        let mut rooks = self.board.pieces[piece] & !pinned;
        while rooks > 0 {
            let from = rooks.trailing_zeros();
            let mut quiet = self.mt.get_rook_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            rooks &= rooks-1;
        }
    }

    fn rook_attack(&mut self, pinned: u64, target: u64) {
        let piece = 4+self.board.colour_to_move;
        let mut rooks = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.colour_to_move^1] & target;
        while rooks > 0 {
            let from = rooks.trailing_zeros();
            let attack = self.mt.get_rook_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            rooks &= rooks-1;
        }
    }

    fn bishop_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 6+self.board.colour_to_move;
        let mut bishops = self.board.pieces[piece] & !pinned;
        while bishops > 0 {
            let from = bishops.trailing_zeros();
            let mut quiet = self.mt.get_bishop_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;
            self.add_quiet(quiet, from, piece as u32);
            bishops &= bishops -1;
        }
    }

    fn bishop_attack(&mut self, pinned: u64, target: u64) {
        let piece = 6+self.board.colour_to_move;
        let mut bishops = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.colour_to_move^1] & target;
        while bishops > 0 {
            let from = bishops.trailing_zeros();
            let attack = self.mt.get_bishop_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            bishops &= bishops -1;
        }
    }

    fn queen_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 8+self.board.colour_to_move;
        let mut queens = self.board.pieces[piece] & !pinned;
        while queens > 0 {
            let from = queens.trailing_zeros();
            let mut quiet = self.mt.get_bishop_moves(self.board.util[2], from as usize);
            quiet |= self.mt.get_rook_moves(self.board.util[2], from as usize);
            quiet &= !self.board.util[2] & target;

            self.add_quiet(quiet, from, piece as u32);
            queens &= queens -1;
        }
    }

    fn queen_attack(&mut self, pinned: u64, target: u64) {
        let piece = 8+self.board.colour_to_move;
        let mut queens = self.board.pieces[piece] & !pinned;
        let opp = self.board.util[self.board.colour_to_move^1] & target;
        while queens > 0 {
            let from = queens.trailing_zeros();
            let mut attack = self.mt.get_bishop_moves(self.board.util[2], from as usize);
            attack |= self.mt.get_rook_moves(self.board.util[2], from as usize);
            attack &= opp;
            self.add_attack(attack, from, piece as u32);
            queens &= queens -1;
        }
    }

    fn king_quiet(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let quiet = self.mt.king_moves[from as usize] & !self.board.util[2];
        self.add_quiet(quiet, from, piece as u32);
    }

    fn king_attack(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let attack = self.mt.king_moves[from as usize] & self.board.util[self.board.colour_to_move^1];
        self.add_attack(attack, from, piece as u32);
    }

    fn king_castle(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let rights = self.board.castle_state >> (2*(self.board.colour_to_move^1));
        let kingside = rights & 0b10;
        let queenside = rights & 1;

        if kingside > 0 && self.board.util[2] & (0x60<<(self.board.colour_to_move*56)) == 0 {
            let move_type = KINGSIDE + self.board.colour_to_move as u32;
            self.add_move(Move::new(from,from+2,piece as u32,0,move_type))
        }

        if queenside > 0 && self.board.util[2] & (0xE<<(self.board.colour_to_move*56)) == 0 {
            let move_type = QUEENSIDE + self.board.colour_to_move as u32;
            self.add_move(Move::new(from,from-2,piece as u32,0,move_type))
        }
    }

    fn king_in_check(&mut self) {
        let from = self.board.pieces[KING + self.board.colour_to_move].trailing_zeros();
        let occ = self.board.util[2] & !self.board.pieces[KING + self.board.colour_to_move];

        let mut possible = self.mt.king_moves[from as usize] & !self.board.util[self.board.colour_to_move];
        let opp_colour = self.board.colour_to_move^1;
        // get opp pawn attacks
        possible &= if opp_colour == 0 {
            !(((self.board.pieces[0] & !FA) << 7) | ((self.board.pieces[0] & !FH) << 9))
        } else {
            !(((self.board.pieces[1] & !FH) >> 7) | ((self.board.pieces[1] & !FA) >> 9))
        };
        // get opp king moves
        possible &= !(self.mt.king_moves[self.board.pieces[KING + opp_colour].trailing_zeros() as usize]);

        // get opp knight moves
        let mut knights = self.board.pieces[KNIGHT + opp_colour];
        while knights > 0 {
            possible &= !(self.mt.knight_moves[knights.trailing_zeros() as usize]);
            knights &= knights - 1;
        }
        // rook/queen
        let mut rook_queen = self.board.pieces[ROOK + opp_colour] | self.board.pieces[QUEEN + opp_colour];
        while rook_queen > 0 {
            possible &= !(self.mt.get_rook_moves(occ, rook_queen.trailing_zeros() as usize));
            rook_queen &= rook_queen - 1
        }
        // bishop/queen
        let mut bishop_queen = self.board.pieces[BISHOP + opp_colour] | self.board.pieces[QUEEN + opp_colour];
        while bishop_queen > 0 {
            possible &= !(self.mt.get_bishop_moves(occ, bishop_queen.trailing_zeros() as usize));
            bishop_queen &= bishop_queen - 1
        }

        let quiet = possible & !self.board.util[2];
        let attack = possible & self.board.util[opp_colour];
        self.add_quiet(quiet, from, (KING+self.board.colour_to_move) as u32);
        self.add_attack(attack, from, (KING+self.board.colour_to_move) as u32);
    }

    fn add_pawn_quiet(&mut self, mut pawns: u64, to_diff: i32, move_type: u32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            self.add_move(Move::new(from, to,piece,0,move_type));
            pawns &= pawns-1;
        }
    }

    fn add_pawn_quiet_promo(&mut self, mut pawns: u64, to_diff: i32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            for xpiece in [QUEEN as u32,KNIGHT as u32,ROOK as u32,BISHOP as u32] {
                self.add_move(Move::new(from, to, piece, xpiece + piece, PROMO));
            }
            pawns &= pawns-1;
        }
    }

    fn add_pawn_attack(&mut self, mut pawns: u64, to_diff: i32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            let xpiece = get_xpiece(self.board, to);
            self.add_move(Move::new(from, to, piece, xpiece, CAP));
            pawns &= pawns-1;
        }
    }

    fn add_pawn_attack_promo(&mut self, mut pawns: u64, to_diff: i32) {
        let piece = self.board.colour_to_move as u32;

        // if pawns > 0 { print_bb(pawns); }

        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            let xpiece = get_xpiece(self.board, to);
            for promo_cap in [Q_PROMO_CAP,R_PROMO_CAP,N_PROMO_CAP,B_PROMO_CAP] {
                self.add_move(Move::new(from, to, piece, xpiece, promo_cap));
            }
            pawns &= pawns-1;
        }
    }

    fn add_quiet(&mut self, mut quiet: u64, from: u32, piece: u32) {
        while quiet > 0 {
            let to = quiet.trailing_zeros();
            self.add_move(Move::new(from, to, piece, 0, QUIET));
            quiet &= quiet-1;
        }
    }

    fn add_attack(&mut self, mut attack: u64, from: u32, piece: u32) {
        while attack > 0 {
            let to = attack.trailing_zeros();
            let xpiece = get_xpiece(self.board, to);
            self.add_move(Move::new(from, to, piece, xpiece, CAP));
            attack &= attack-1;
        }
    }

    fn get_attackers(&self, sq: usize) -> u64 {
        let mut attackers = 0;
        let mut colour_to_move = self.board.colour_to_move^1;
        let pawns = self.board.pieces[PAWN + colour_to_move];
        attackers |= self.mt.pawn_attacks[colour_to_move ^ 1][sq] & pawns;

        let knights = self.board.pieces[KNIGHT + colour_to_move];
        attackers |= self.mt.knight_moves[sq] & knights;

        let king = self.board.pieces[KING + colour_to_move];
        attackers |= self.mt.king_moves[sq] & king;

        let bishop_queen = self.board.pieces[QUEEN + colour_to_move] | self.board.pieces[BISHOP + colour_to_move];
        attackers |= self.mt.get_bishop_moves(self.board.util[2], sq) & bishop_queen;

        let rook_queen = self.board.pieces[ROOK + colour_to_move] | self.board.pieces[QUEEN + colour_to_move];
        attackers |= self.mt.get_rook_moves(self.board.util[2], sq) & rook_queen;

        attackers
    }

    fn get_pinned_pieces(&self) -> u64 {
        let ksq = self.board.pieces[KING+self.board.colour_to_move].trailing_zeros() as usize;

        let mut pinned_pieces = 0;

        let mut rq = self.board.pieces[ROOK + (self.board.colour_to_move ^ 1)];
        rq |= self.board.pieces[QUEEN + (self.board.colour_to_move ^ 1)];

        let mut rq_pinners = rq & self.mt.get_rook_xray(
            self.board.util[2],
            self.board.util[self.board.colour_to_move],
            ksq
        );

        let mut bq = self.board.pieces[BISHOP + (self.board.colour_to_move ^ 1)];
        bq |= self.board.pieces[QUEEN + (self.board.colour_to_move ^ 1)];

        let mut bq_pinners = bq & self.mt.get_bishop_xray(
            self.board.util[2],
            self.board.util[self.board.colour_to_move],
            ksq
        );

        // print_bb!(rq_pinners, bq_pinners);

        while rq_pinners > 0 {
            let p_sq = rq_pinners.trailing_zeros() as usize;
            pinned_pieces |= self.mt.rays[1][ksq] & self.mt.rays[5][p_sq];
            pinned_pieces |= self.mt.rays[5][ksq] & self.mt.rays[1][p_sq];
            pinned_pieces |= self.mt.rays[3][ksq] & self.mt.rays[7][p_sq];
            pinned_pieces |= self.mt.rays[7][ksq] & self.mt.rays[3][p_sq];
            rq_pinners &= rq_pinners-1;
        }

        while bq_pinners > 0 {
            let p_sq = bq_pinners.trailing_zeros() as usize;
            pinned_pieces |= self.mt.rays[0][ksq] & self.mt.rays[4][p_sq];
            pinned_pieces |= self.mt.rays[4][ksq] & self.mt.rays[0][p_sq];
            pinned_pieces |= self.mt.rays[2][ksq] & self.mt.rays[6][p_sq];
            pinned_pieces |= self.mt.rays[6][ksq] & self.mt.rays[2][p_sq];
            bq_pinners &= bq_pinners-1;
        }

        pinned_pieces
    }

    fn get_ray_inbetween(&self, sq1: usize, sq2: usize) -> u64 {
        let (higher, lower) = if sq1 > sq2 { (sq1, sq2) } else { (sq2, sq1 ) };

        let mut dir = 12;
        for d in 0..4 {
            if SQUARES[higher] & self.mt.rays[d][lower] > 0 {
                dir = d;
                break;
            }
        }

        self.mt.rays[dir][lower] & (SQUARES[higher]-1)
    }

    #[inline]
    fn add_move(&mut self, m: Move) { self.moves.push(m); }

    fn score_moves(&mut self, km: &KillerMoves, depth: usize) {
        for m in &self.moves {
            self.scores.push(self.score_move(*m, km, depth))
        }
    }

    fn score_move(&self, m: Move, km: &KillerMoves, depth: usize) -> i32 {
        if Some(m) == self.best_move {
            return i32::MAX;
        } else if let Some(score) = km.get_move(m, depth) {
            return score
        }

        let mut score = 0;
        match m.move_type() {
            QUIET | DOUBLE | WKINGSIDE | BKINGSIDE | WQUEENSIDE | BQUEENSIDE | PROMO | EP => {
                score = m.piece() as i32;
            }
            CAP | N_PROMO_CAP | R_PROMO_CAP | B_PROMO_CAP | Q_PROMO_CAP => {
                score = self.see(m) + CAP_SCORE_OFFSET;
            }
            _ => panic!("Unknown movetype {}", m.move_type())
        }

        score
    }

    fn see_get_least_valuable(&self, attackers: u64, colour: usize) -> (usize, u64) {
        let piece_iter = self.board.pieces.iter()
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

    fn see(&self, m: Move) -> i32 {
        // trying to understand the https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
        // def using dynamic programming.

        let mut gain: [i32; 32] = [0;32];
        let mut depth = 0;

        let (from, to, mut piece, xpiece, _) = m.all();

        // froms is a bb of the next possible piece to move that attacks the to square
        let mut from_piece = SQUARES[from];
        let mut occ = self.board.util[2];
        let mut attackers = get_all_attackers(self.board, self.mt, to);

        // can_xray is just all pieces that arent knights
        // as there is no sliding piece that can be behind a knight that could attack the target
        let can_xray = occ ^ self.board.pieces[KNIGHT] ^ self.board.pieces[KNIGHT+1];

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
                attackers |= occ & get_all_attackers(self.board, self.mt, to);
            }

            (piece, from_piece) = self.see_get_least_valuable(
                attackers,
                self.board.colour_to_move ^ (depth & 1)
            );
        }

        // iterate over all the stored gain values to find the max - negamax style
        for i in (1..depth).rev() {
            gain[i-1] = -max(-gain[i-1], gain[i]);
        }

        gain[0]
    }
}

impl <'a> Iterator for MoveList<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut best_idx = None;
        let mut best_score = i32::MIN;

        for (i, score) in self.scores.iter().enumerate() {
            if score > &best_score {
                best_idx = Some(i);
                best_score = *score;
            }
        }

        if let Some(best_idx) = best_idx {
            // println!("{}\n{best_score}\n", self.moves[best_idx]);

            self.scores[best_idx] = i32::MIN;
            Some(self.moves[best_idx])
        } else {
            None
        }
    }
}

pub fn get_piece(board: &Board, sq: u32) -> u32 {
    let bb_sq = SQUARES[sq as usize];
    let s = board.colour_to_move as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces[p as usize] > 0 { return p; }
    }

    12
}

pub fn get_xpiece(board: &Board, sq: u32) -> u32 {
    let bb_sq = SQUARES[sq as usize];
    // let start = (self.board.colour_to_move^1) as u32;
    let s = (board.colour_to_move^1) as u32;

    for i in [0, 2, 4, 6, 8, 10] {
        let p = i + s;
        if bb_sq & board.pieces[p as usize] > 0 { return p; }
    }

    12
}

#[inline]
pub fn sq_attacked(board: &Board, mt: &MoveTables, sq: usize, attacker_colour: usize) -> bool {
    get_attackers(board, mt, sq, attacker_colour) > 0
}

#[inline]
fn get_all_attackers(board: &Board, mt: &MoveTables, sq: usize) -> u64 {
    get_attackers(board, mt, sq, 0) | get_attackers(board, mt, sq, 1)
}

fn get_attackers(board: &Board, mt: &MoveTables, sq: usize, attacker_colour: usize) -> u64 {
    let mut attackers = mt.pawn_attacks[attacker_colour^1][sq] & board.pieces[PAWN+attacker_colour];
    attackers |= mt.knight_moves[sq] & board.pieces[KNIGHT+attacker_colour];
    attackers |= mt.king_moves[sq] & board.pieces[KING+attacker_colour];
    let mut bq = mt.get_bishop_moves(board.util[2], sq);
    bq &= board.pieces[BISHOP+attacker_colour] | board.pieces[QUEEN+attacker_colour];
    attackers |= bq;
    let mut rq = mt.get_rook_moves(board.util[2], sq);
    rq &= board.pieces[ROOK+attacker_colour] | board.pieces[QUEEN+attacker_colour];
    attackers |= rq;

    attackers
}


pub fn moved_into_check(board: &Board, mt: &MoveTables, m: Move) -> bool {
    let ksq = board.pieces[KING + (board.colour_to_move ^ 1)].trailing_zeros() as usize;
    SQUARES[m.from() as usize] & mt.superrays[ksq] > 0
        && sq_attacked(board, mt, ksq, board.colour_to_move)
}

pub fn is_in_check(board: &Board, mt: &MoveTables) -> bool {
    sq_attacked(
        board, 
        mt, 
        board.pieces[KING+board.colour_to_move].trailing_zeros() as usize,
        board.colour_to_move^1    
    )
}

pub fn is_legal_move(board: &Board, mt: &MoveTables, m: Move, prev_moves: &PrevMoves) -> bool {
    if board.halfmove > 100 || prev_moves.get_count(board.hash) == 3 { return false; }

    match m.move_type() {
        // check castle moves to see if the king passes through an attacked square
        WKINGSIDE => !sq_attacked(board, mt, 5, 1) 
            & !sq_attacked(board, mt, 6, 1),
        WQUEENSIDE => !sq_attacked(board, mt,3, 1) 
            & !sq_attacked(board, mt, 2, 1),
        BKINGSIDE => !sq_attacked(board, mt, 61, 0) 
            & !sq_attacked(board, mt, 62, 0),
        BQUEENSIDE => !sq_attacked(board, mt, 59, 0) 
            & !sq_attacked(board, mt, 58, 0),
        _ => true,
    }
}


#[test]
fn see_evaluation() {
    let mt = MoveTables::new();

    let board1 = Board::new_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -");
    let ml = MoveList::new(&board1, &mt, 100, 100);

    let rxe5 = Move::new(4, 36, ROOK as u32, 1, CAP);
    assert_eq!(ml.see(rxe5), PIECE_VALUES[PAWN]);

    let board2 = Board::new_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
    let ml = MoveList::new(&board2, &mt, 100, 100);

    let nxe5 = Move::new(19, 36, KNIGHT as u32, 1, CAP);
    assert_eq!(ml.see(nxe5), -225)
}

#[test]
fn move_ordering() {
    let tt = SeqTT::new();
    let mt = MoveTables::new();
    let km = KillerMoves::new();


    let board = Board::new_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -");
    let ml_scored = MoveList::all_scored(&board, false, &mt, &tt, &km, 0);
    let scored: Vec<Move> = ml_scored.collect();
    let unscored: Vec<Move> = MoveList::all(&board, &mt, false).moves;

    assert_eq!(scored.len(), unscored.len())
}