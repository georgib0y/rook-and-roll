/*
from 0-63,      6 bits
to 0-63,        6 bits
piece 0-11,     4 bits
xpiece 0-12,    4 bits
movetype 0-12,  4 bits
              ----------
                24 bits

ep, last castle state and last halfmove can all be stored in search - aha not with copy move tho
*/

use crate::move_info::SQ_NAMES;
use crate::movegen::{B_PROMO_CAP, CAP, DOUBLE, EP, KINGSIDE, N_PROMO_CAP, PROMO, QUEENSIDE, QUIET, Q_PROMO_CAP, R_PROMO_CAP, get_piece, get_xpiece, CAP_SCORE_OFFSET};
use crate::{Board, MoveList, MoveTables};
use std::fmt::{write, Display, Formatter};
use crate::board::PIECE_NAMES;
use crate::movegen::MTYPE_STRS;
use crate::search::MAX_DEPTH;

const PREV_MOVE_SIZE: usize = 16384;
const PREV_MOVE_MASK: u64 = 0x3FFF;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Move(u32);

impl Move {
    #[inline]
    pub fn new(from: u32, to: u32, piece: u32, xpiece: u32, move_type: u32) -> Move {
        //dbg!(from, to, piece, xpiece, move_type);
        Move(from << 18 | to << 12 | piece << 8 | xpiece << 4 | move_type)
    }

    pub fn new_from_u32(m: u32) -> Move { Move(m) }

    #[inline]
    pub fn from(&self) -> u32 {
        self.0 >> 18
    }

    #[inline]
    pub fn to(&self) -> u32 {
        (self.0 >> 12) & 0x3F
    }

    #[inline]
    pub fn piece(&self) -> u32 {
        (self.0 >> 8) & 0xF
    }

    #[inline]
    pub fn xpiece(&self) -> u32 {
        (self.0 >> 4) & 0xF
    }

    #[inline]
    pub fn move_type(&self) -> u32 {
        self.0 & 0xF
    }

    #[inline]
    pub fn all(&self) -> (usize, usize, usize, usize, u32) {
        (self.from() as usize, self.to() as usize, self.piece() as usize, self.xpiece() as usize, self.move_type())
    }

    pub fn new_from_text(text: &str, b: &Board) -> Move {
        let from = sq_from_text(&text[0..2]) as u32;
        let to = sq_from_text(&text[2..4]) as u32;

        let promo = if text.len() == 5 {
            Some(promo_piece_from_text(&text[4..]) + b.colour_to_move)
        } else {
            None
        };

        let promo_piece = (promo.unwrap_or(12)) as u32;

        let piece = get_piece(b, from);
        let mut xpiece = get_xpiece(b, to);

        let mut move_type = QUIET;

        if piece < 2 && (from as i32 - to as i32).abs() == 16 {
            move_type = DOUBLE;
        } else if piece == 10 || piece == 11 {
            if (from as i32 - to as i32) == -2 {
                move_type = KINGSIDE + b.colour_to_move as u32;
            } else if (from as i32 - to as i32) == 2 {
                move_type = QUEENSIDE + b.colour_to_move as u32;
            }
        }

        if xpiece < 12 && promo_piece < 12 {
            match promo_piece {
                2 | 3 => move_type = N_PROMO_CAP,
                4 | 5 => move_type = R_PROMO_CAP,
                6 | 7 => move_type = B_PROMO_CAP,
                8 | 9 => move_type = Q_PROMO_CAP,
                _ => panic!("promo_piece {promo_piece} not an available promo piece"),
            }
        } else if promo_piece < 12 {
            move_type = PROMO;
            xpiece = promo_piece;
        } else if xpiece < 2 && to == b.ep as u32 {
            move_type = EP;
        } else if xpiece < 12 {
            move_type = CAP;
        }

        Move::new(from, to, piece, xpiece, move_type)
    }

    pub fn as_uci_string(&self) -> String {
        let mut mv = String::new();

        let (f,t,p,x,m) = self.all();

        mv.push_str(SQ_NAMES[f]);
        mv.push_str(SQ_NAMES[t]);
        if m > 6 && m < 12 {
            let promo_piece = if m == 7 {
                x as u32
            } else {
                m - 6
            };

            mv.push_str(&text_from_promo_piece(promo_piece));
        }
        mv
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (fr,t,p,x,m) = self.all();

        write!(
            f, "From: {} ({})\tTo:{} ({})\tPiece: {} ({})\tXPiece: {} ({})\tMove Type: {} ({})",
            fr, SQ_NAMES[fr], t, SQ_NAMES[t], p, PIECE_NAMES[p], x, PIECE_NAMES[x], m, MTYPE_STRS[m as usize]
        )
    }
}

fn sq_from_text(sq: &str) -> usize {
    let sq = sq.as_bytes();

    ((sq[0] - "a".as_bytes()[0]) + (8 * (sq[1] - "1".as_bytes()[0]))) as usize
}

fn promo_piece_from_text(p: &str) -> usize {
    match p {
        "n" => 2,
        "r" => 4,
        "b" => 6,
        "q" => 8,
        _ => 12,
    }
}

fn text_from_promo_piece(promo_piece: u32) -> String {
    match promo_piece {
        2 | 3 => String::from("n"),
        4 | 5 => String::from("r"),
        6 | 7 => String::from("b"),
        8 | 9 => String::from("q"),
        _ => String::from(""),
    }
}

#[derive(Clone)]
pub struct PrevMoves {
    prev: Box<[u8; PREV_MOVE_SIZE]>
}

impl PrevMoves {
    pub fn new() -> PrevMoves {
        PrevMoves { prev: Box::new([0;PREV_MOVE_SIZE]) }
    }

    pub fn add(&mut self, hash: u64) {
        self.prev[(hash & PREV_MOVE_MASK) as usize] += 1;
    }

    pub fn remove(&mut self, hash: u64) {
        self.prev[(hash & PREV_MOVE_MASK) as usize] -= 1;
    }

    pub fn get_count(&self, hash: u64) -> u8 {
        self.prev[(hash & PREV_MOVE_MASK) as usize]
    }
}

#[derive(Clone)]
pub struct KillerMoves {
    killer_moves: Vec<(Option<Move>, Option<Move>)>,
}

impl KillerMoves {
    pub fn new() -> KillerMoves {
        KillerMoves { killer_moves: vec![(None, None); MAX_DEPTH] }
    }

    pub fn add(&mut self, m: Move, depth: usize) {
        if let Some(killers) = self.killer_moves.get_mut(depth) {
            // dont add the same move in twice
            if Some(m) == killers.0 { return; }

            // shuffle the killer moves upwards
            killers.1 = killers.0;
            killers.0 = Some(m);
        }
    }

    // returns an option containing an i32 for move scoring or none
    pub fn get_move(&self, m: Move, depth: usize) -> Option<i32> {
        self.killer_moves.get(depth).and_then(|killers| {
            killers.0.and_then(|_| Some(2+CAP_SCORE_OFFSET))
                .or_else(|| killers.1.and_then(|_| Some(1+CAP_SCORE_OFFSET)))

        })
    }
}

// TODO Could have a root order list that uses iter to change the score of every