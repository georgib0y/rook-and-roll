use crate::eval::{gen_mat_value, gen_pst_value, MAT_SCORES};
use crate::move_info::{PST, SQUARES};
use crate::moves::{Move, MoveType};
use crate::zorbist::Zorb;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use crate::movegen::{get_piece, get_xpiece};

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 4;
pub const BISHOP: usize = 6;
pub const QUEEN: usize = 8;
pub const KING: usize = 10;

pub const PIECE_NAMES: [&str; 12] = ["P", "p", "N", "n", "R", "r", "B", "b", "Q", "q", "K", "k"];

const DEFAULT_PIECES: [u64; 12] = [
    0x000000000000FF00, //wp 0
    0x00FF000000000000, //bp 1
    0x0000000000000042, //wn 2
    0x4200000000000000, //bn 3
    0x0000000000000081, //wr 4
    0x8100000000000000, //br 5
    0x0000000000000024, //wb 6
    0x2400000000000000, //bb 7
    0x0000000000000008, //wq 8
    0x0800000000000000, //bq 9
    0x0000000000000010, //wk 10
    0x1000000000000000, //bk 11
];

const DEFAULT_UTIL: [u64; 3] = [
    DEFAULT_PIECES[0] | DEFAULT_PIECES[2] | DEFAULT_PIECES[4] |
        DEFAULT_PIECES[6] | DEFAULT_PIECES[8] | DEFAULT_PIECES[10], // white

    DEFAULT_PIECES[1] | DEFAULT_PIECES[3] | DEFAULT_PIECES[5] |
        DEFAULT_PIECES[7] | DEFAULT_PIECES[9] | DEFAULT_PIECES[11], // black

    DEFAULT_PIECES[0] | DEFAULT_PIECES[2] | DEFAULT_PIECES[4] |
        DEFAULT_PIECES[6] | DEFAULT_PIECES[8] | DEFAULT_PIECES[10] |
        DEFAULT_PIECES[1] | DEFAULT_PIECES[3] | DEFAULT_PIECES[5] |
        DEFAULT_PIECES[7] | DEFAULT_PIECES[9] | DEFAULT_PIECES[11], // all
];

const WKS_STATE: usize = 0;
const WQS_STATE: usize = 1;
const BKS_STATE: usize = 2;
const BQS_STATE: usize = 3;



// 0 - white to move, 1 - black to move
#[derive(Copy, Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub util: [u64; 3],
    pub ctm: usize,
    pub castle_state: u8,
    pub ep: usize,
    pub halfmove: usize,
    pub hash: u64,
    pub mg_value: i32,
    pub eg_value: i32,
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            pieces: DEFAULT_PIECES,
            util: DEFAULT_UTIL,
            ctm: 0,
            castle_state: 0b1111,
            ep: 64,
            halfmove: 0,
            hash: 0,
            mg_value: 0,
            eg_value: 0,
        };

        board.hash = gen_hash(board);
        let mat = gen_mat_value(&board);
        let (mg, eg) = gen_pst_value(&board);
        board.mg_value = mat + mg;
        board.eg_value = mat + eg;

        board
    }


    pub fn copy_make(&self, m: Move) -> Board {
        // println!("{}", self);
        // get info from board
        let (from, to, piece, xpiece, move_type) = m.all();
        let ft = SQUARES[from] | SQUARES[to];

        let mut hash = self.hash;
        hash ^= Zorb::piece(piece, from);
        hash ^= Zorb::piece(piece, to);

        let mut pieces = self.pieces;
        pieces[piece] ^= ft;

        let mut util = self.util;
        util[self.ctm] ^= ft;
        util[2] ^= ft;

        // toggle ep file if there is one
        if self.ep < 64 { hash ^= Zorb::ep_file(self.ep); }
        let mut ep = 64;

        let mut halfmove = self.halfmove + 1;

        let mut mg_value = self.mg_value-PST::mid_pst(piece,from)+PST::mid_pst(piece, to);
        let mut eg_value = self.eg_value-PST::end_pst(piece,from)+PST::end_pst(piece, to);

        match move_type {
            // (piece > 1) as usize == 1 if piece not pawn, so halfmove*1, if pawn halfmove*0 == 0
            MoveType::Quiet => halfmove *= (piece > 1) as usize,

            MoveType::Double => {
                ep = to - 8 + (self.ctm * 16);
                hash ^= Zorb::ep_file(ep);
                halfmove = 0;
            }

            MoveType::Cap => {
                pieces[xpiece] ^= SQUARES[to];
                util[self.ctm ^ 1] ^= SQUARES[to];
                util[2] ^= SQUARES[to];

                mg_value -= MAT_SCORES[xpiece] + PST::mid_pst(xpiece, to);
                eg_value -= MAT_SCORES[xpiece] + PST::end_pst(xpiece, to);

                hash ^= Zorb::piece(xpiece, to);
                halfmove = 0;
            }

            MoveType::WKingSide =>
                update_castling(&mut pieces, &mut util, 0, 7, 5, &mut hash, &mut mg_value, &mut eg_value),
            MoveType::BKingSide =>
                update_castling(&mut pieces, &mut util, 1, 63, 61, &mut hash, &mut mg_value, &mut eg_value),
            MoveType::WQueenSide =>
                update_castling(&mut pieces, &mut util, 0, 0, 3, &mut hash, &mut mg_value, &mut eg_value),
            MoveType::BQueenSide =>
                update_castling(&mut pieces, &mut util, 1, 56, 59, &mut hash, &mut mg_value, &mut eg_value),

            MoveType::Promo => {
                // toggle the pawn off and the toggled piece on
                pieces[self.ctm] ^= SQUARES[to];
                pieces[xpiece] ^= SQUARES[to];

                hash ^= Zorb::piece(piece, to);
                hash ^= Zorb::piece(xpiece, to);

                mg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
                mg_value += -PST::mid_pst(piece, to) + PST::mid_pst(xpiece, to);
                eg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
                eg_value += -PST::end_pst(piece, to) + PST::end_pst(xpiece, to);
                halfmove = 0;
            }
            MoveType::NPromoCap | MoveType::RPromoCap | MoveType::BPromoCap | MoveType::QPromoCap => {
                // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
                // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
                let promo_piece = (move_type as usize - 7) * 2 + self.ctm;

                // toggle captured piece
                pieces[xpiece] ^= SQUARES[to];
                util[self.ctm ^ 1] ^= SQUARES[to];
                // retoggle piece (as its been replaced by the capture-er)
                util[2] ^= SQUARES[to];
                // toggle pawn off
                pieces[self.ctm] ^= SQUARES[to];
                // toggle promo
                pieces[promo_piece] ^= SQUARES[to];

                hash ^= Zorb::piece(piece, to);
                hash ^= Zorb::piece(promo_piece, to);
                hash ^= Zorb::piece(xpiece, to);

                // update mat value (the promo piece - the captured piece and the pre-promoted piece)
                mg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
                mg_value += -PST::mid_pst(xpiece, to) - PST::mid_pst(piece, to) + PST::mid_pst(promo_piece, to);
                eg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
                eg_value += -PST::end_pst(xpiece, to) - PST::end_pst(piece, to) + PST::end_pst(promo_piece, to);
                halfmove = 0;
            }
            MoveType::Ep => {
                let ep_sq = to - 8 + (self.ctm * 16);
                pieces[self.ctm ^ 1] ^= SQUARES[ep_sq]; // toggle capture pawn off
                util[self.ctm ^ 1] ^= SQUARES[ep_sq];
                util[2] ^= SQUARES[ep_sq];
                // dbg!("ep");
                hash ^= Zorb::piece(self.ctm^1, ep_sq);

                mg_value -= MAT_SCORES[self.ctm^1] + PST::mid_pst(self.ctm^1, ep_sq);
                eg_value -= MAT_SCORES[self.ctm^1] + PST::end_pst(self.ctm^1, ep_sq);
                halfmove = 0;
            }
        }

        let castle_state = update_castle_state(from, to, piece, self.castle_state, &mut hash);

        hash ^= Zorb::colour();

        Board {
            pieces, util, ctm: self.ctm ^ 1, castle_state, ep, halfmove, hash, mg_value, eg_value
        }
    }
}

fn update_castling(
    pieces: &mut [u64;12],
    util: &mut [u64;3],
    colour: usize,
    from: usize,
    to: usize,
    hash: &mut u64,
    mg_value: &mut i32,
    eg_value: &mut i32,
) {
    let sqs = SQUARES[from] | SQUARES[to];
    pieces[ROOK + colour] ^= sqs;
    util[colour] ^= sqs;
    util[2] ^= sqs;

    *mg_value += -PST::mid_pst(ROOK + colour, from) + PST::mid_pst(ROOK + colour, to);
    *eg_value += -PST::end_pst(ROOK + colour, from) + PST::end_pst(ROOK + colour, to);

    *hash ^= Zorb::piece(ROOK + colour, from);
    *hash ^= Zorb::piece(ROOK + colour, to);
}

fn update_castle_state(
    from: usize,
    to: usize,
    piece: usize,
    mut castle_state: u8,
    hash: &mut u64
) -> u8 {
    // stop thinking you can optimise this you have the ifs for the hash
    if (piece == 10 || from == 7 || to == 7) && castle_state & 0b1000 > 0 {
        castle_state &= 0b0111;
        *hash ^= Zorb::castle_rights(WKS_STATE);
    }

    if (piece == 10 || from == 0 || to == 0) && castle_state & 0b100 > 0 {
        castle_state &= 0b1011;
        *hash ^= Zorb::castle_rights(WQS_STATE);
    }

    if (piece == 11 || from == 63 || to == 63) && castle_state & 0b10 > 0 {
        castle_state &= 0b1101;
        *hash ^= Zorb::castle_rights(BKS_STATE);
    }

    if (piece == 11 || from == 56 || to == 56) && castle_state & 0b1 > 0 {
        castle_state &= 0b1110;
        *hash ^= Zorb::castle_rights(BQS_STATE);
    }

    castle_state
}

#[test]
fn inc_value_update() {
    crate::init();
    let board = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();

    let quiet_move = Move::new(0,1,ROOK as u32,0,MoveType::Quiet);
    let quiet_board = board.copy_make(quiet_move);

    let mat = gen_mat_value(&quiet_board);
    let (mg, eg) = gen_pst_value(&quiet_board);
    let mg_quiet = mat + mg;
    let eg_quiet = mat + eg;

    assert_eq!(quiet_board.mg_value, mg_quiet);
    assert_eq!(quiet_board.eg_value, eg_quiet);

    let cap_move = Move::new(25,32,BISHOP as u32, KNIGHT as u32 +1, MoveType::Cap);
    let cap_board = board.copy_make(cap_move);

    let mat = gen_mat_value(&cap_board);
    let (mg, eg) = gen_pst_value(&cap_board);
    let mg_cap = mat + mg;
    let eg_cap = mat + eg;

    assert_eq!(cap_board.mg_value, mg_cap);
    assert_eq!(cap_board.eg_value, eg_cap);
}



impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SQ_PIECES: [&str;12] = ["P ", "p ", "N ", "n ", "R ", "r ", "B ", "b ", "Q ", "q ", "K ", "k "];

        let add_sq = |s, sq| format!("{s}{}", get_piece(self, sq)
            .or(get_xpiece(self, sq))
            .map_or("- ", |piece| SQ_PIECES[piece as usize])
        );

        // iterate over every row (56-63, 48-55, ... 0-7) and concat the pieces of that row to the out string
        let mut out = (0..8).rev().map(|i| (i+1, (i*8..i*8+8)))
            .fold(String::new(), |out, (row_num, row)|
                format!("{out}\n{row_num}   {}", row.fold(String::new(), add_sq))
            );

        out.push_str("\n\n    A B C D E F G H\n");
        write!(f, "{}", out)
    }
}

pub fn gen_hash(board: Board) -> u64 {
    let mut hash = 0;

    for piece in 0..12 {
        for (i, sq) in SQUARES.iter().enumerate().take(64) {
            if (board.pieces[piece] & sq) > 0 {
                hash ^= Zorb::piece(piece, i);
            }
        }
    }

    // if black to move toggle zorb
    if board.ctm == 1 { hash ^= Zorb::colour(); }
    if (board.castle_state & 0b1000) == 8 { hash ^= Zorb::castle_rights(WKS_STATE); }
    if (board.castle_state & 0b100) == 4 { hash ^= Zorb::castle_rights(WQS_STATE); }
    if (board.castle_state & 0b10) == 2 { hash ^= Zorb::castle_rights(BKS_STATE); }
    if (board.castle_state & 0b1) == 1 { hash ^= Zorb::castle_rights(BQS_STATE); }
    if board.ep < 64 { hash ^= Zorb::ep_file(board.ep); }

    hash
}

// macro to print a list of bitboards (u64s) one after each other, v similar to dbg!() but only for bbs
#[macro_export]
macro_rules! print_bb {
    ( $( $args:expr ),* ) => {
        {
            $( print_bb($args); )*
        }
    };
}

pub fn _print_bb(bb: u64) {
    let mut out = String::new();

    for i in (1..9).rev() {
        out.push_str(&i.to_string());
        out.push(' ');

        for sq in SQUARES.iter().skip(i * 8 - 8).take(8) {
            if sq & bb > 0 {
                out.push_str(" X ");
            } else {
                out.push_str(" - ");
            }
        }
        out.push('\n');
    }
    out.push_str("   A  B  C  D  E  F  G  H\n");

    println!("{}", out);
}


