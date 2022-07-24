#![allow(unused)]

use std::fmt;
use crate::move_info::SQUARES;
use crate::moves::Move;
use crate::movegen::*;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 4;
pub const BISHOP: usize = 6;
pub const QUEEN: usize = 8;
pub const KING: usize = 10;

#[derive(Copy, Clone)]
pub struct Board {
    pub pieces: [u64; 12],
    pub util: [u64; 3],

    // 0 - white to move, 1 - black to move
    pub colour_to_move: usize,

    pub castle_state: u8,
    pub ep: usize,

    pub halfmove: usize,
    // TODO i dont think fullmove is needed?
    //pub fullmove: usize,
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

        let util =  [
            pieces[0] | pieces[2] | pieces[4] | pieces[6] | pieces[8] | pieces[10], // white

            pieces[1] | pieces[3] | pieces[5] | pieces[7] | pieces[9] | pieces[11], // black

            pieces[0] | pieces[2] | pieces[4] | pieces[6] | pieces[8] | pieces[10] |  // all
                pieces[1] | pieces[3] | pieces[5] | pieces[7] | pieces[9] | pieces[11],
        ];

        Board {
            pieces,
            util,
            colour_to_move: 0,
            castle_state: 0b1111,
            ep: 64,
            halfmove: 0,
            //fullmove: 0
        }
    }

    pub fn new_fen(fen: &str) -> Board {
        let mut b = Board::new();
        // clear the board
        b.pieces = [0;12];
        b.util = [0;3];
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
            "KQkq"  => b.castle_state = 0b1111,
            "KQk"   => b.castle_state = 0b1110,
            "KQq"   => b.castle_state = 0b1101,
            "KQ"    => b.castle_state = 0b1100,
            "Kkq"   => b.castle_state = 0b1011,
            "Kk"    => b.castle_state = 0b1010,
            "Kq"    => b.castle_state = 0b1001,
            "K"     => b.castle_state = 0b1000,
            "Qkq"   => b.castle_state = 0b0111,
            "Qk"    => b.castle_state = 0b0110,
            "Qq"    => b.castle_state = 0b0101,
            "Q"     => b.castle_state = 0b0100,
            "kq"    => b.castle_state = 0b0011,
            "k"     => b.castle_state = 0b0010,
            "q"     => b.castle_state = 0b0001,
            "-"     => b.castle_state = 0b0000,
            _       => b.castle_state = 16,
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

        b.halfmove = fen[4].parse().unwrap();
        // b.fullmove = fen[5].parse().unwrap();

        b
    }

    pub fn copy_make(&self, m: &Move) -> Board {
        // copy
        let mut b = *self;

        // flip from and to bits on relevant boards
        let from = m.from();
        let to = m.to();
        let ft = SQUARES[from as usize] | SQUARES[to as usize];
        b.pieces[m.piece() as usize] ^= ft;
        b.util[b.colour_to_move] ^= ft;
        b.util[2] ^= ft;
        b.ep = 64;

        match m.move_type() {
            QUIET => {}
            DOUBLE => b.ep = (to - 8 + (b.colour_to_move as u32 * 16)) as usize,
            CAP => {
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize];
                b.util[2] ^= SQUARES[to as usize];
            }
            WKINGSIDE => {
                b.pieces[4] ^= SQUARES[7] | SQUARES[5];
                b.util[0] ^= SQUARES[7] | SQUARES[5];
                b.util[2] ^= SQUARES[7] | SQUARES[5];
            }
            BKINGSIDE => {
                b.pieces[5] ^= SQUARES[63] | SQUARES[61];
                b.util[1] ^= SQUARES[63] | SQUARES[61];
                b.util[2] ^= SQUARES[63] | SQUARES[61];
            }
            WQUEENSIDE => {
                b.pieces[4] ^= SQUARES[0] | SQUARES[3];
                b.util[0] ^= SQUARES[0] | SQUARES[3];
                b.util[2] ^= SQUARES[0] | SQUARES[3];
            }
            BQUEENSIDE => {
                b.pieces[5] ^= SQUARES[56] | SQUARES[59];
                b.util[1] ^= SQUARES[56] | SQUARES[59];
                b.util[2] ^= SQUARES[56] | SQUARES[59];
            }
            PROMO => {
                b.pieces[b.colour_to_move] ^= SQUARES[to as usize];
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
            }
            N_PROMO_CAP => {
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize];
                b.util[2] ^= SQUARES[to as usize];
                b.pieces[b.colour_to_move] ^= SQUARES[to as usize];
                b.pieces[2 + b.colour_to_move] ^= SQUARES[to as usize];
            }
            R_PROMO_CAP => {
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize];
                b.util[2] ^= SQUARES[to as usize];
                b.pieces[b.colour_to_move] ^= SQUARES[to as usize];
                b.pieces[4 + b.colour_to_move] ^= SQUARES[to as usize];
            }
            B_PROMO_CAP => {
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize];
                b.util[2] ^= SQUARES[to as usize];
                b.pieces[b.colour_to_move] ^= SQUARES[to as usize];
                b.pieces[6 + b.colour_to_move] ^= SQUARES[to as usize];
            }
            Q_PROMO_CAP => {
                b.pieces[m.xpiece() as usize] ^= SQUARES[to as usize];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize];
                b.util[2] ^= SQUARES[to as usize];
                b.pieces[b.colour_to_move] ^= SQUARES[to as usize];
                b.pieces[8 + b.colour_to_move] ^= SQUARES[to as usize];
            }
            EP => {
                b.pieces[1 - b.colour_to_move] ^= SQUARES[to as usize - 8 + (b.colour_to_move * 16)];
                b.util[1 - b.colour_to_move] ^= SQUARES[to as usize - 8 + (b.colour_to_move * 16)];
                b.util[2] ^= SQUARES[to as usize - 8 + (b.colour_to_move * 16)];
            }
            _ => panic!("Move type: {}, outside of range!", m.move_type()),
        }

        if from as usize == 7 || to as usize == 7 {
            b.castle_state &= 0b0111;
        }
        if from as usize == 0 || to as usize == 0 {
            b.castle_state &= 0b1011;
        }
        if from as usize == 63 || to as usize == 63 {
            b.castle_state &= 0b1101;
        }
        if from as usize == 56 || to as usize == 56 {
            b.castle_state &= 0b1110;
        }
        if m.piece() as usize == 10 {
            b.castle_state &= 0b11;
        }
        if m.piece() as usize == 11 {
            b.castle_state &= 0b1100;
        }

        b.colour_to_move ^= 1;

        b
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

pub fn print_bb(bb: u64) {
    let mut out = String::new();

    for i in (1..9).rev() {
        out.push_str(&i.to_string());
        out.push(' ');

        for j in i*8-8..i*8 {
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