/*
from 0-63,      6 bits
to 0-63,        6 bits
piece 0-11,     4 bits
xpiece 0-12,    4 bits
movetype 0-12,  4 bits
              ----------
                24 bits

ep, last castle state and last halfmove can all be stored in searchers - aha not with copy move tho
*/

use crate::board::{Board, PIECE_NAMES, WHITE};
use crate::move_info::SQ_NAMES;
use crate::movegen::{get_piece, get_xpiece};
use crate::searcher::MAX_DEPTH;
use std::fmt::{Display, Formatter};

const PREV_MOVE_SIZE: usize = 16384;
const PREV_MOVE_MASK: u64 = 0x3FFF;

pub const NULL_MOVE: Move = Move(0);

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Quiet,
    Double,
    Cap,
    WKingSide,
    BKingSide,
    WQueenSide,
    BQueenSide,
    Promo,
    NPromoCap,
    RPromoCap,
    BPromoCap,
    QPromoCap,
    Ep,
}

impl MoveType {
    #[inline]
    pub fn kingside(ctm: usize) -> MoveType {
        if ctm == 0 {
            MoveType::WKingSide
        } else {
            MoveType::BKingSide
        }
    }

    #[inline]
    pub fn queenside(ctm: usize) -> MoveType {
        if ctm == 0 {
            MoveType::WQueenSide
        } else {
            MoveType::BQueenSide
        }
    }

    pub fn is_promo(&self) -> bool {
        matches!(
            self,
            MoveType::Promo
                | MoveType::NPromoCap
                | MoveType::BPromoCap
                | MoveType::RPromoCap
                | MoveType::QPromoCap
        )
    }

    pub fn is_cap(&self) -> bool {
        matches!(
            self,
            MoveType::Cap
                | MoveType::NPromoCap
                | MoveType::BPromoCap
                | MoveType::RPromoCap
                | MoveType::QPromoCap
                | MoveType::Ep
        )
    }
}

impl From<u32> for MoveType {
    fn from(value: u32) -> Self {
        match value {
            0 => MoveType::Quiet,
            1 => MoveType::Double,
            2 => MoveType::Cap,
            3 => MoveType::WKingSide,
            4 => MoveType::BKingSide,
            5 => MoveType::WQueenSide,
            6 => MoveType::BQueenSide,
            7 => MoveType::Promo,
            8 => MoveType::NPromoCap,
            9 => MoveType::RPromoCap,
            10 => MoveType::BPromoCap,
            11 => MoveType::QPromoCap,
            12 => MoveType::Ep,
            _ => panic!("unknown movetype with value = {}", value),
        }
    }
}

impl Display for MoveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MoveType::Quiet => "Quiet",
                MoveType::Double => "Double",
                MoveType::Cap => "Cap",
                MoveType::WKingSide => "W Kingside",
                MoveType::BKingSide => "B Kingside",
                MoveType::WQueenSide => "W Queenside",
                MoveType::BQueenSide => "B Queenside",
                MoveType::Promo => "Promo",
                MoveType::NPromoCap => "N Promo Cap",
                MoveType::RPromoCap => "R Promo Cap",
                MoveType::BPromoCap => "B Promo Cap",
                MoveType::QPromoCap => "Q Promo Cap",
                MoveType::Ep => "Ep",
            }
        )
    }
}

#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq)]
pub struct Move(u32);

impl Move {
    #[inline]
    pub const fn new(from: u32, to: u32, piece: u32, xpiece: u32, move_type: MoveType) -> Move {
        //dbg!(from, to, piece, xpiece, move_type);
        Move(from << 18 | to << 12 | piece << 8 | xpiece << 4 | move_type as u32)
    }

    pub const fn empty() -> Move {
        Move(0)
    }

    pub const fn _new_from_u32(m: u32) -> Move {
        Move(m)
    }

    #[inline]
    pub const fn from(&self) -> u32 {
        self.0 >> 18
    }

    #[inline]
    pub const fn to(&self) -> u32 {
        (self.0 >> 12) & 0x3F
    }

    #[inline]
    pub const fn piece(&self) -> u32 {
        (self.0 >> 8) & 0xF
    }

    #[inline]
    pub const fn xpiece(&self) -> u32 {
        (self.0 >> 4) & 0xF
    }

    #[inline]
    pub fn move_type(&self) -> MoveType {
        MoveType::from(self.0 & 0xF)
    }

    #[inline]
    pub fn all(&self) -> (usize, usize, usize, usize, MoveType) {
        (
            self.from() as usize,
            self.to() as usize,
            self.piece() as usize,
            self.xpiece() as usize,
            self.move_type(),
        )
    }

    pub fn new_from_text(text: &str, b: &Board) -> Move {
        let from = sq_from_text(&text[0..2]) as u32;
        let to = sq_from_text(&text[2..4]) as u32;

        let promo = if text.len() == 5 {
            Some(promo_piece_from_text(&text[4..]) + b.ctm())
        } else {
            None
        };

        let promo_piece = (promo.unwrap_or(12)) as u32;

        let piece = get_piece(b, from).unwrap();

        let mut move_type = MoveType::Quiet;

        if piece < 2 && from.abs_diff(to) == 16 {
            move_type = MoveType::Double;
        } else if (piece == 10 || piece == 11) && from.abs_diff(to) == 2 {
            match from.cmp(&to) {
                std::cmp::Ordering::Less => {
                    move_type = if b.ctm() == WHITE {
                        MoveType::WKingSide
                    } else {
                        MoveType::BKingSide
                    };
                }
                std::cmp::Ordering::Greater => {
                    move_type = if b.ctm() == WHITE {
                        MoveType::WQueenSide
                    } else {
                        MoveType::BQueenSide
                    };
                }
                _ => {}
            }
        }

        let mut xpiece = get_xpiece(b, to).unwrap_or(12);
        if xpiece < 12 && promo_piece < 12 {
            match promo_piece {
                2 | 3 => move_type = MoveType::NPromoCap,
                4 | 5 => move_type = MoveType::RPromoCap,
                6 | 7 => move_type = MoveType::BPromoCap,
                8 | 9 => move_type = MoveType::QPromoCap,
                _ => panic!("promo_piece {promo_piece} not an available promo piece"),
            }
        } else if promo_piece < 12 {
            move_type = MoveType::Promo;
            xpiece = promo_piece;
        } else if piece < 2 && to == b.ep() as u32 {
            move_type = MoveType::Ep;
        } else if xpiece < 12 {
            move_type = MoveType::Cap;
        }

        Move::new(from, to, piece, xpiece, move_type)
    }

    pub fn as_uci_string(&self) -> String {
        let mut mv = String::new();

        let (f, t, _, x, m) = self.all();

        mv.push_str(SQ_NAMES[f]);
        mv.push_str(SQ_NAMES[t]);

        mv.push_str(&match m {
            MoveType::Promo => text_from_promo_piece(x as u32),
            MoveType::NPromoCap => "n".to_string(),
            MoveType::RPromoCap => "r".to_string(),
            MoveType::BPromoCap => "b".to_string(),
            MoveType::QPromoCap => "q".to_string(),
            _ => "".to_string(),
        });

        mv
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (fr, t, p, x, m) = self.all();

        write!(
            f,
            "From: {} ({})\tTo:{} ({})\tPiece: {} ({})\tXPiece: {} ({})\tMove Type: {}",
            fr, SQ_NAMES[fr], t, SQ_NAMES[t], p, PIECE_NAMES[p], x, PIECE_NAMES[x], m
        )
    }
}

impl From<Move> for u32 {
    fn from(value: Move) -> Self {
        value.0
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
    prev: Box<[u8; PREV_MOVE_SIZE]>,
}

impl Default for PrevMoves {
    fn default() -> Self {
        PrevMoves {
            prev: Box::new([0; PREV_MOVE_SIZE]),
        }
    }
}

impl PrevMoves {
    pub fn new() -> PrevMoves {
        PrevMoves::default()
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
    killer_moves: [[Option<Move>; 2]; MAX_DEPTH],
}

impl KillerMoves {
    pub fn new() -> KillerMoves {
        KillerMoves {
            killer_moves: [[None, None]; MAX_DEPTH],
        }
    }

    pub fn add(&mut self, m: Move, depth: usize) {
        if let Some(killers) = self.killer_moves.get_mut(depth) {
            // dont add the same move in twice
            if Some(m) == killers[0] {
                return;
            }

            // shuffle the killer moves upwards
            killers[1] = killers[0];
            killers[0] = Some(m);
        }
    }

    pub fn get_kms(&self, depth: usize) -> [Option<Move>; 2] {
        self.killer_moves[depth]
    }

    // returns an option containing an i32 for move scoring or none
    pub fn get_move_priority(&self, m: Move, depth: usize) -> Option<i32> {
        let [k1, k2] = self.killer_moves[depth];

        if Some(m) == k1 {
            Some(1)
        } else if Some(m) == k2 {
            Some(0)
        } else {
            None
        }
    }
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self::new()
    }
}
