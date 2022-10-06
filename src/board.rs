#![allow(unused)]

use crate::eval::{gen_mat_value, MAT_SCORES};
use crate::move_info::SQUARES;
use crate::movegen::*;
use crate::moves::Move;
use crate::zorbist::Zorb;
use std::borrow::Cow;
use std::fmt;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 4;
pub const BISHOP: usize = 6;
pub const QUEEN: usize = 8;
pub const KING: usize = 10;

// 0 - white to move, 1 - black to move

#[derive(Copy, Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub util: [u64; 3],
    pub colour_to_move: usize,
    pub castle_state: u8,
    pub ep: usize,
    pub halfmove: usize,
    pub hash: u64,
    pub mat_value: i32,
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
            pieces,
            util,
            colour_to_move: 0,
            castle_state: 0b1111,
            ep: 64,
            halfmove: 0,
            hash: 0,
            mat_value: 0,
        };

        board.hash = gen_hash(board);
        board.mat_value = gen_mat_value(board);

        board
    }

    pub fn new_fen(fen: &str) -> Board {
        let mut b = Board::new();
        // clear the board
        b.pieces = [0; 12];
        b.util = [0; 3];
        let fen: Vec<&str> = fen.split(' ').collect();
        let mut j = 56;
        for f in fen[0].chars() {
            match f {
                'P' => {
                    b.pieces[0] ^= SQUARES[j];
                    j += 1;
                }
                'p' => {
                    b.pieces[1] ^= SQUARES[j];
                    j += 1;
                }
                'N' => {
                    b.pieces[2] ^= SQUARES[j];
                    j += 1;
                }
                'n' => {
                    b.pieces[3] ^= SQUARES[j];
                    j += 1;
                }
                'R' => {
                    b.pieces[4] ^= SQUARES[j];
                    j += 1;
                }
                'r' => {
                    b.pieces[5] ^= SQUARES[j];
                    j += 1;
                }
                'B' => {
                    b.pieces[6] ^= SQUARES[j];
                    j += 1;
                }
                'b' => {
                    b.pieces[7] ^= SQUARES[j];
                    j += 1;
                }
                'Q' => {
                    b.pieces[8] ^= SQUARES[j];
                    j += 1;
                }
                'q' => {
                    b.pieces[9] ^= SQUARES[j];
                    j += 1;
                }
                'K' => {
                    b.pieces[10] ^= SQUARES[j];
                    j += 1;
                }
                'k' => {
                    b.pieces[11] ^= SQUARES[j];
                    j += 1;
                }
                '1' => j += '1' as usize - '0' as usize,
                '2' => j += '2' as usize - '0' as usize,
                '3' => j += '3' as usize - '0' as usize,
                '4' => j += '4' as usize - '0' as usize,
                '5' => j += '5' as usize - '0' as usize,
                '6' => j += '6' as usize - '0' as usize,
                '7' => j += '7' as usize - '0' as usize,
                '8' => j += '8' as usize - '0' as usize,
                '/' => j -= 16,
                _ => {}
            }
        }

        b.util[0] =
            b.pieces[0] | b.pieces[2] | b.pieces[4] | b.pieces[6] | b.pieces[8] | b.pieces[10];
        b.util[1] =
            b.pieces[1] | b.pieces[3] | b.pieces[5] | b.pieces[7] | b.pieces[9] | b.pieces[11];
        b.util[2] = b.util[0] | b.util[1];
        b.colour_to_move = if fen[1].contains('w') { 0 } else { 1 };

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
        b.mat_value = gen_mat_value(b);

        b
    }

    // TODO debug hashing, dunno how to do that tho
    // TODO also perf seems to drop a lot for not that many lines, could just be the zorb arr tho

    pub fn copy_make(&self, m: &Move) -> Board {
        // copy board
        // let mut b = *self;
        // let mut b = self.clone();

        // get info from board
        let (from, to, piece, xpiece, move_type) = m.all();

        // flip from and to bits on relevant boards
        let ft = SQUARES[from] | SQUARES[to];

        let mut pieces = self.pieces;
        pieces[piece] ^= ft;

        let mut util = self.util;
        util[self.colour_to_move] ^= ft;
        util[2] ^= ft;

        // toggle ep file if there is one

        let mut hash = self.hash;

        if self.ep < 64 {
            hash ^= Zorb::ep_file(self.ep);
        }
        let mut ep = 64;

        hash ^= Zorb::piece(piece, from);
        hash ^= Zorb::piece(piece, to);

        let mut halfmove = self.halfmove + 1;

        let mut mat_value = self.mat_value;
        match move_type {
            // (piece > 1) as usize == 1 if piece not pawn, so halfmove*1, if pawn halfmove*0 == 0
            QUIET => halfmove *= (piece > 1) as usize,
            DOUBLE => {
                ep = to - 8 + (self.colour_to_move * 16);
                hash ^= Zorb::ep_file(ep);
                halfmove = 0;
            }
            CAP => {
                pieces[xpiece] ^= SQUARES[to];
                util[self.colour_to_move ^ 1] ^= SQUARES[to];
                util[2] ^= SQUARES[to];

                mat_value -= MAT_SCORES[xpiece];
                hash ^= Zorb::piece(xpiece, to);
                halfmove = 0;
            }
            WKINGSIDE => {
                pieces[4] ^= SQUARES[7] | SQUARES[5];
                util[0] ^= SQUARES[7] | SQUARES[5];
                util[2] ^= SQUARES[7] | SQUARES[5];

                hash ^= Zorb::piece(4, 7);
                hash ^= Zorb::piece(4, 5);
            }
            BKINGSIDE => {
                pieces[5] ^= SQUARES[63] | SQUARES[61];
                util[1] ^= SQUARES[63] | SQUARES[61];
                util[2] ^= SQUARES[63] | SQUARES[61];

                hash ^= Zorb::piece(5, 63);
                hash ^= Zorb::piece(5, 61);
            }
            WQUEENSIDE => {
                pieces[4] ^= SQUARES[0] | SQUARES[3];
                util[0] ^= SQUARES[0] | SQUARES[3];
                util[2] ^= SQUARES[0] | SQUARES[3];

                hash ^= Zorb::piece(4, 0);
                hash ^= Zorb::piece(4, 3);
            }
            BQUEENSIDE => {
                pieces[5] ^= SQUARES[56] | SQUARES[59];
                util[1] ^= SQUARES[56] | SQUARES[59];
                util[2] ^= SQUARES[56] | SQUARES[59];

                hash ^= Zorb::piece(5, 56);
                hash ^= Zorb::piece(5, 59);
            }
            PROMO => {
                pieces[self.colour_to_move] ^= SQUARES[to];
                pieces[xpiece] ^= SQUARES[to];

                hash ^= Zorb::piece(piece, to);
                hash ^= Zorb::piece(xpiece, to);

                mat_value += MAT_SCORES[xpiece];
                halfmove = 0;
            }
            N_PROMO_CAP | R_PROMO_CAP | B_PROMO_CAP | Q_PROMO_CAP => {
                // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
                // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
                let promo_piece = (move_type as usize - 7) * 2 + self.colour_to_move;

                // toggle captured piece
                pieces[xpiece] ^= SQUARES[to];
                util[self.colour_to_move ^ 1] ^= SQUARES[to];
                // retoggle piece (as its been replaced by the capture-er)
                util[2] ^= SQUARES[to];
                // toggle pawn off
                pieces[self.colour_to_move] ^= SQUARES[to];
                // toggle promo
                pieces[promo_piece] ^= SQUARES[to];

                hash ^= Zorb::piece(piece, to);
                hash ^= Zorb::piece(promo_piece, to);
                hash ^= Zorb::piece(xpiece, to);

                // update mat value (the promo piece - the captured piece and the pre-promoted piece)
                mat_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
                halfmove = 0;
            }
            EP => {
                pieces[self.colour_to_move ^ 1] ^= SQUARES[to - 8 + (self.colour_to_move * 16)];
                util[self.colour_to_move ^ 1] ^= SQUARES[to - 8 + (self.colour_to_move * 16)];
                util[2] ^= SQUARES[to - 8 + (self.colour_to_move * 16)];
                hash ^= Zorb::piece(xpiece, to - 8 + (self.colour_to_move * 16));

                mat_value -= MAT_SCORES[xpiece];
                halfmove = 0;
            }
            _ => panic!("Move type: {move_type}, outside of range!"),
        }

        let mut castle_state = self.castle_state;
        match piece {
            10 => {
                if (from == 7 || to == 7) && castle_state & 0b1000 > 0 {
                    castle_state &= 0b0111;
                    hash ^= Zorb::castle_rights(0);
                }
                if (from == 0 || to == 0) && castle_state & 0b100 > 0 {
                    castle_state &= 0b1011;
                    hash ^= Zorb::castle_rights(1);
                }
            }
            11 => {
                if (from == 63 || to == 63) && castle_state & 0b10 > 0 {
                    castle_state &= 0b1101;
                    hash ^= Zorb::castle_rights(2);
                }
                if (from == 56 || to == 56) && castle_state & 0b1 > 0 {
                    castle_state &= 0b1110;
                    hash ^= Zorb::castle_rights(3);
                }
            }
            _ => {}
        }

        hash ^= Zorb::colour();

        Board {
            pieces,
            util,
            colour_to_move: self.colour_to_move ^ 1,
            castle_state,
            ep,
            halfmove,
            hash,
            mat_value
        }
    }
}






impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        for i in (1..9).rev() {
            let s = i.to_string();
            out.push_str(&s);
            out.push_str("    ");
            for j in i * 8 - 8..i * 8 {
                if (SQUARES[j] & self.pieces[0]) > 0 {
                    out.push_str("P ");
                }
                if (SQUARES[j] & self.pieces[1]) > 0 {
                    out.push_str("p ");
                }
                if (SQUARES[j] & self.pieces[2]) > 0 {
                    out.push_str("N ");
                }
                if (SQUARES[j] & self.pieces[3]) > 0 {
                    out.push_str("n ");
                }
                if (SQUARES[j] & self.pieces[4]) > 0 {
                    out.push_str("R ");
                }
                if (SQUARES[j] & self.pieces[5]) > 0 {
                    out.push_str("r ");
                }
                if (SQUARES[j] & self.pieces[6]) > 0 {
                    out.push_str("B ");
                }
                if (SQUARES[j] & self.pieces[7]) > 0 {
                    out.push_str("b ");
                }
                if (SQUARES[j] & self.pieces[8]) > 0 {
                    out.push_str("Q ");
                }
                if (SQUARES[j] & self.pieces[9]) > 0 {
                    out.push_str("q ");
                }
                if (SQUARES[j] & self.pieces[10]) > 0 {
                    out.push_str("K ");
                }
                if (SQUARES[j] & self.pieces[11]) > 0 {
                    out.push_str("k ");
                }
                if (SQUARES[j] & self.util[2]) == 0 {
                    out.push_str("- ");
                }
            }
            out.push('\n');
        }
        out.push_str("\n     A B C D E F G H\n");
        write!(f, "{}", out)
    }
}

fn gen_hash(board: Board) -> u64 {
    let mut hash = 0;

    for piece in 0..12 {
        for sq in 0..64 {
            if (board.pieces[piece] & SQUARES[sq]) > 0 {
                hash ^= Zorb::piece(piece, sq);
            }
        }
    }

    // if black to move toggle zorb
    if board.colour_to_move == 1 {
        hash ^= Zorb::colour();
    }

    if (board.castle_state & 0b1000) == 8 {
        hash ^= Zorb::castle_rights(0);
    }
    if (board.castle_state & 0b100) == 4 {
        hash ^= Zorb::castle_rights(1);
    }
    if (board.castle_state & 0b10) == 2 {
        hash ^= Zorb::castle_rights(2);
    }
    if (board.castle_state & 0b1) == 1 {
        hash ^= Zorb::castle_rights(3);
    }

    if board.ep < 64 {
        hash ^= Zorb::ep_file(board.ep);
    }

    hash
}

pub fn print_bb(bb: u64) {
    let mut out = String::new();

    for i in (1..9).rev() {
        out.push_str(&i.to_string());
        out.push(' ');

        for j in i * 8 - 8..i * 8 {
            if SQUARES[j] & bb > 0 {
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

pub fn print_bb_over_board(m: u64, b: &Board) {
    let mut out = String::new();

    for i in (1..9).rev() {
        let s = i.to_string();
        out.push_str(&s);
        out.push_str("   ");

        for j in i * 8 - 8..i * 8 {
            if (SQUARES[j] & m) > 0 {
                out.push('(');
            } else {
                out.push(' ');
            }

            if (SQUARES[j] & b.pieces[0]) > 0 {
                out.push('P');
            } else if (SQUARES[j] & b.pieces[1]) > 0 {
                out.push('p');
            } else if (SQUARES[j] & b.pieces[2]) > 0 {
                out.push('N');
            } else if (SQUARES[j] & b.pieces[3]) > 0 {
                out.push('n');
            } else if (SQUARES[j] & b.pieces[4]) > 0 {
                out.push('R');
            } else if (SQUARES[j] & b.pieces[5]) > 0 {
                out.push('r');
            } else if (SQUARES[j] & b.pieces[6]) > 0 {
                out.push('B');
            } else if (SQUARES[j] & b.pieces[7]) > 0 {
                out.push('b');
            } else if (SQUARES[j] & b.pieces[8]) > 0 {
                out.push('Q');
            } else if (SQUARES[j] & b.pieces[9]) > 0 {
                out.push('q');
            } else if (SQUARES[j] & b.pieces[10]) > 0 {
                out.push('K');
            } else if (SQUARES[j] & b.pieces[11]) > 0 {
                out.push('k');
            } else {
                out.push('-');
            }

            if (SQUARES[j] & m) > 0 {
                out.push(')');
            } else {
                out.push(' ');
            }
        }
        out.push('\n');
    }
    out.push_str("\n     A  B  C  D  E  F  G  H\n");

    println!("{}", out);
}
