use crate::board::{Board, BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use crate::move_info::{FA, FH, R1, R2, R3, R6, R7, R8, SQUARES};
use crate::move_tables::MT;
use crate::moves::Move;
use crate::{print_bb, MoveTables};
use std::cmp::{max, min};
use crate::movegen::{gen_attacks, get_xpiece};

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

const ALL_SQUARES: u64 = u64::MAX;
const NO_SQUARES: u64 = 0;

pub struct MoveList <'a> {
    moves: Vec<Move>,
    board: &'a Board,
    mt: &'a MoveTables,
    idx: usize,
}

impl <'a> MoveList <'a> {
    pub fn new(board: &'a Board, mt: &MoveTables, capacity: usize) -> MoveList<'a> {
        MoveList { board, mt, moves: Vec::with_capacity(capacity), idx: 0 }
    }

    pub fn all(board: &'a Board, mt: &MoveTables, check: bool) -> MoveList<'a> {
        let mut move_list;
        if check {
            move_list = MoveList::new(board, mt, 75);
            move_list.gen_check()
        } else {
            move_list = MoveList::new(board, mt, 220);
            move_list.gen_attacks();
            move_list.gen_quiet();
        };

        move_list
    }

    pub fn attacks(board: &'a Board, mt: &MoveTables) -> MoveList<'a> {
        let mut move_list = MoveList::new(board, mt, 100);
        move_list.gen_attacks();

        move_list
    }

    fn gen_attacks(&mut self) {
        if self.board.colour_to_move == 0 {
            self.white_pawn_attack(NO_SQUARES, ALL_SQUARES);
        } else {
            self.black_pawn_attack(NO_SQUARES, ALL_SQUARES);
        }
        self.knight_attack(NO_SQUARES, ALL_SQUARES);
        self.rook_attack(NO_SQUARES, ALL_SQUARES);
        self.bishop_attack(NO_SQUARES, ALL_SQUARES);
        self.queen_attack(NO_SQUARES, ALL_SQUARES);
        self.king_attack();
    }

    fn gen_quiet(&mut self) {
        if self.board.colour_to_move == 0 {
            self.white_pawn_quiet(NO_SQUARES, ALL_SQUARES);
        } else {
            self.black_pawn_quiet(NO_SQUARES, ALL_SQUARES);
        }
        self.knight_quiet(NO_SQUARES, ALL_SQUARES);
        self.rook_quiet(NO_SQUARES, ALL_SQUARES);
        self.bishop_quiet(NO_SQUARES, ALL_SQUARES);
        self.queen_quiet(NO_SQUARES, ALL_SQUARES);
        self.king_quiet();
    }

    fn gen_check(&mut self) {

    }

    // pinned: all the pieces that are pinned by an attacker
    // target: all the squares that the pawns could move to
    fn white_pawn_quiet(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[0] & !pinned;
        let occ = self.board.util[2] & target;
        let push = pawns & !(occ >> 8);
        let double = (pawns & R2) & !(occ >> 16) & !(self.board.util[2] >> 8);

        self.add_pawn_quiet(push & !R7, 8, QUIET);
        self.add_pawn_quiet(push, 16, DOUBLE);
        self.add_pawn_quiet_promo(push & R7, 8);
    }

    fn white_pawn_attack(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[0] & !pinned;
        let opp = self.board.util[1] & target;
        let up_lefts = (pawns & !FA) & (opp >> 7);
        let up_rights = (pawns & !FH) & (opp >> 9);

        self.add_pawn_attack(up_lefts & !R7, 7);
        self.add_pawn_attack_promo(up_lefts & R7, 7);
        self.add_pawn_attack(up_rights & !R7, 9);
        self.add_pawn_attack_promo(up_rights & R7, 9);

        let ep = self.board.ep as u32;
        if ep < 64 && SQUARES[ep as usize - 8] & self.board.pieces[1] & (pawns & !FA) << 7 > 0 {
            self.moves.push(Move::new(ep - 7, ep, 0, 1, EP));
        } else if ep < 64 &&  SQUARES[ep as usize - 8] & self.board.pieces[1] & (pawns & !FH) << 9 > 0 {
            self.moves.push(Move::new(ep - 9, ep, 0, 1, EP));
        }
    }

    fn black_pawn_quiet(&mut self, pinned: u64, target: u64) {
        let pawns = self.board.pieces[1] & !pinned;
        let occ = self.board.util[2] & target;
        let push = pawns & !(occ << 8);
        let double = (pawns & R7) & !(occ << 16) & !(self.board.util[2] << 8);

        self.add_pawn_quiet(push & !R2, -8, QUIET);
        self.add_pawn_quiet(push, -16, DOUBLE);
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
        if ep < 64 && SQUARES[self.board.ep + 8] & self.board.pieces[0] & (pawns & !FH) >> 7 > 0 {
            self.moves.push(Move::new(ep + 7, ep, 1, 0, EP));
        } else if ep < 64 && SQUARES[self.board.ep + 8] & self.board.pieces[0] & (pawns & !FA) >> 9 > 0 {
            self.moves.push(Move::new(ep + 9, ep, 1, 0, EP));
        }
    }

    fn knight_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 2+self.board.colour_to_move;
        let mut knights = self.board.pieces[piece] & !pinned;
        while knights > 0 {
            let from = knights.trailing_zeros();
            let quiet = MT.knight_moves[from as usize] & !self.board.util[2] & target;
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
            let attacks = MT.knight_moves[from as usize] & opp;
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
            let mut quiet = MT.get_rook_moves(self.board.util[2], from as usize);
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
            let attack = MT.get_rook_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            rooks &= rooks-1;
        }
    }

    fn bishop_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 6+self.board.colour_to_move;
        let mut bishops = self.board.pieces[piece] & !pinned;
        while bishops > 0 {
            let from = bishops.trailing_zeros();
            let mut quiet = MT.get_bishop_moves(self.board.util[2], from as usize);
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
            let attack = MT.get_bishop_moves(self.board.util[2], from as usize) & opp;
            self.add_attack(attack, from, piece as u32);
            bishops &= bishops -1;
        }
    }

    fn queen_quiet(&mut self, pinned: u64, target: u64) {
        let piece = 8+self.board.colour_to_move;
        let mut queens = self.board.pieces[piece] & !pinned;
        while queens > 0 {
            let from = queens.trailing_zeros();
            let mut quiet = MT.get_bishop_moves(self.board.util[2], from as usize);
            quiet |= MT.get_rook_moves(self.board.util[2], from as usize);
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
            let mut attack = MT.get_bishop_moves(self.board.util[2], from as usize);
            attack |= MT.get_rook_moves(self.board.util[2], from as usize);
            attack &= opp;
            self.add_attack(attack, from, piece as u32);
            queens &= queens -1;
        }
    }

    fn king_quiet(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let quiet = MT.king_moves[from as usize] & !self.board.util[2];
        self.add_quiet(quiet, from, piece as u32);
    }

    fn king_attack(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let attack = MT.king_moves[from as usize] & self.board.util[self.board.colour_to_move^1];
        self.add_attack(attack, from, piece as u32);
    }

    fn king_castle(&mut self) {
        let piece = 10+self.board.colour_to_move;
        let from = self.board.pieces[piece].trailing_zeros();
        let rights = self.board.castle_state >> 2*(self.board.colour_to_move^1);
        let kingside = rights & 0b10;
        let queenside = rights & 1;

        if kingside > 0 && self.board.util[2] & (0x60<<self.board.colour_to_move*56) == 0 {
            let move_type = KINGSIDE + self.board.colour_to_move as u32;
            self.moves.push(Move::new(from,from+2,piece as u32,0,move_type))
        }

        if queenside > 0 && self.board.util[2] & (0xE<<self.board.colour_to_move*56) == 0 {
            let move_type = QUEENSIDE + self.board.colour_to_move as u32;
            self.moves.push(Move::new(from,from-2,piece as u32,0,move_type))
        }
    }

    fn add_pawn_quiet(&mut self, mut pawns: u64, to_diff: i32, move_type: u32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            self.moves.push(Move::new(from, to,piece,0,move_type));
            pawns &= pawns-1;
        }
    }

    fn add_pawn_quiet_promo(&mut self, mut pawns: u64, to_diff: i32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            for xpiece in [8,4,2,6] {
                self.moves.push(Move::new(from, to, piece, xpiece, PROMO));
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
            self.moves.push(Move::new(from, to, piece, xpiece, CAP));
            pawns &= pawns-1;
        }
    }

    fn add_pawn_attack_promo(&mut self, mut pawns: u64, to_diff: i32) {
        let piece = self.board.colour_to_move as u32;
        while pawns > 0 {
            let from = pawns.trailing_zeros();
            let to = (from as i32 + to_diff) as u32;
            let xpiece = get_xpiece(self.board, to);
            for promo_cap in [Q_PROMO_CAP,R_PROMO_CAP,N_PROMO_CAP,B_PROMO_CAP] {
                self.moves.push(Move::new(from, to, piece, xpiece, promo_cap));
            }
            pawns &= pawns-1;
        }
    }

    fn add_quiet(&mut self, mut quiet: u64, from: u32, piece: u32) {
        while quiet > 0 {
            let to = quiet.trailing_zeros();
            self.moves.push(Move::new(from, to, piece, 0, QUIET));
            quiet &= quiet-1;
        }
    }

    fn add_attack(&mut self, mut attack: u64, from: u32, piece: u32) {
        while attack > 0 {
            let to = attack.trailing_zeros();
            let xpiece = get_xpiece(self.board, to);
            self.moves.push(Move::new(from, to, piece, xpiece, CAP));
            attack &= attack-1;
        }
    }
}

impl <'a> Iterator for MoveList <'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_move = None;

        if self.idx < self.moves.len() {
            next_move = Some(self.moves[self.idx]);
            self.idx += 1;
        }

        next_move
    }
}