use crate::eval::{gen_mat_value, gen_pst_value, MAT_SCORES};
use crate::move_info::{PST, SQUARES};
use crate::moves::{Move, MoveType};
use crate::zorbist::Zorb;
use std::collections::HashMap;
use std::fmt;
use crate::movegen::{get_piece, get_xpiece};

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 4;
pub const BISHOP: usize = 6;
pub const QUEEN: usize = 8;
pub const KING: usize = 10;

pub const PIECE_NAMES: [&str; 12] = ["P", "p", "N", "n", "R", "r", "B", "b", "Q", "q", "K", "k"];

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
        let pieces = [
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

        let util = [
            pieces[0] | pieces[2] | pieces[4] | pieces[6] | pieces[8] | pieces[10], // white
            pieces[1] | pieces[3] | pieces[5] | pieces[7] | pieces[9] | pieces[11], // black
            pieces[0] | pieces[2] | pieces[4] | pieces[6] | pieces[8] | pieces[10] |  // all
                pieces[1] | pieces[3] | pieces[5] | pieces[7] | pieces[9] | pieces[11],
        ];

        let mut board = Board {
            pieces, util, ctm: 0, castle_state: 0b1111, ep: 64, halfmove: 0, hash: 0, mg_value: 0, eg_value: 0,
        };

        board.hash = gen_hash(board);
        let mat = gen_mat_value(&board);
        let (mg, eg) = gen_pst_value(&board);
        board.mg_value = mat + mg;
        board.eg_value = mat + eg;

        board
    }

    pub fn new_fen(fen: &str) -> Board {
        let mut b = Board::new();
        // clear the board
        b.pieces = [0; 12];
        b.util = [0; 3];
        let fen: Vec<&str> = fen.split(' ').collect();

        let name_piece: HashMap<char, usize> = HashMap::from([
            ('P', 0), ('p', 1), ('N', 2), ('n', 3), ('R', 4), ('r', 5), ('B', 6), ('b', 7),
            ('Q', 8), ('q', 9), ('K', 10), ('k', 11),
        ]);

        let fen_board: Vec<&str> = fen[0].split("/").collect();
        fen_board.iter().rev().enumerate().for_each(|(i, row)| {
            let mut idx = i*8;
            for sq in row.chars() {
                if let Some(piece) = name_piece.get(&sq) {
                    b.pieces[*piece] ^= SQUARES[idx];
                    idx += 1;
                } else if '1' <= sq && sq <= '8' {
                    idx += sq as usize - '0' as usize;
                }

            }
        });

        b.util[0] =
            b.pieces[0] | b.pieces[2] | b.pieces[4] | b.pieces[6] | b.pieces[8] | b.pieces[10];
        b.util[1] =
            b.pieces[1] | b.pieces[3] | b.pieces[5] | b.pieces[7] | b.pieces[9] | b.pieces[11];
        b.util[2] = b.util[0] | b.util[1];
        b.ctm = if fen[1].contains('w') { 0 } else { 1 };

        match fen[2] {
            "KQkq" => b.castle_state = 0b1111,
            "KQk" => b.castle_state = 0b1110,
            "KQq" => b.castle_state = 0b1101,
            "KQ" => b.castle_state = 0b1100,
            "Kkq" => b.castle_state = 0b1011,
            "Kk" => b.castle_state = 0b1010,
            "Kq" => b.castle_state = 0b1001,
            "K" => b.castle_state = 0b1000,
            "Qkq" => b.castle_state = 0b0111,
            "Qk" => b.castle_state = 0b0110,
            "Qq" => b.castle_state = 0b0101,
            "Q" => b.castle_state = 0b0100,
            "kq" => b.castle_state = 0b0011,
            "k" => b.castle_state = 0b0010,
            "q" => b.castle_state = 0b0001,
            "-" => b.castle_state = 0b0000,
            _ => b.castle_state = 16,
        }

        if fen[3].contains('-') {
            b.ep = 64;
        } else {
            // convert file letter to 0-7 value
            let file = fen[3].chars().next().unwrap() as usize - 'a' as usize;
            // convert rank to 0-7 value
            let rank = fen[3].chars().nth(1).unwrap() as usize - '1' as usize;
            b.ep = rank * 8 + file;
        }

        b.halfmove = fen.get(4).unwrap_or(&"0").parse().unwrap();

        // regen the hash after everything is finished
        b.hash = gen_hash(b);
        let mat = gen_mat_value(&b);
        let (mg, eg) = gen_pst_value(&b);
        b.mg_value = mat + mg;
        b.eg_value = mat + eg;


        b
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
//
#[test]
fn inc_value_update() {
    crate::init();
    let board = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");

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
    if (board.castle_state & 0b1000) == 8 { hash ^= Zorb::castle_rights(0); }
    if (board.castle_state & 0b100) == 4 { hash ^= Zorb::castle_rights(1); }
    if (board.castle_state & 0b10) == 2 { hash ^= Zorb::castle_rights(2); }
    if (board.castle_state & 0b1) == 1 { hash ^= Zorb::castle_rights(3); }
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