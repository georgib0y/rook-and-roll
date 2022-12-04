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
use crate::Board;
use std::fmt::{Display, Formatter};
use std::sync::atomic::AtomicU32;
use crate::board::PIECE_NAMES;
// use crate::move_scorer::CAP_SCORE_OFFSET;
use crate::movegen::{CAP_SCORE_OFFSET, get_piece, get_xpiece};
use crate::search::MAX_DEPTH;
use crate::tt::ORDER;

const PREV_MOVE_SIZE: usize = 16384;
const PREV_MOVE_MASK: u64 = 0x3FFF;

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
    Ep
}

impl MoveType {
    #[inline]
    pub fn kingside(ctm: usize) -> MoveType {
        if ctm == 0 { MoveType::WKingSide } else { MoveType::BKingSide }
    }
    
    #[inline]
    pub fn queenside(ctm: usize) -> MoveType {
        if ctm == 0 { MoveType::WQueenSide } else { MoveType::BQueenSide }
    }

    pub fn is_promo(&self) -> bool {
        match self {
            MoveType::Promo |
            MoveType::NPromoCap |
            MoveType::BPromoCap |
            MoveType::RPromoCap |
            MoveType::QPromoCap => true,

            _ => false
        }
    }
}

impl Display for MoveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
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
        })
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Move(u32);

impl Move {
    #[inline]
    pub fn new(from: u32, to: u32, piece: u32, xpiece: u32, move_type: MoveType) -> Move {
        //dbg!(from, to, piece, xpiece, move_type);
        Move(from << 18 | to << 12 | piece << 8 | xpiece << 4 | move_type as u32)
    }

    pub fn _new_from_u32(m: u32) -> Move { Move(m) }

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
    pub fn move_type(&self) -> MoveType {
        // shh no unsafe here ...
        unsafe { std::mem::transmute_copy(&(self.0 & 0xF)) }
    }

    #[inline]
    pub fn all(&self) -> (usize, usize, usize, usize, MoveType) {
        (self.from() as usize, self.to() as usize, self.piece() as usize, self.xpiece() as usize, self.move_type())
    }

    pub fn new_from_text(text: &str, b: &Board) -> Move {
        let from = sq_from_text(&text[0..2]) as u32;
        let to = sq_from_text(&text[2..4]) as u32;

        let promo = if text.len() == 5 {
            Some(promo_piece_from_text(&text[4..]) + b.ctm)
        } else {
            None
        };

        let promo_piece = (promo.unwrap_or(12)) as u32;

        let piece = get_piece(b, from)
            .expect(&format!("couldnt find piece on square {} on board:\n{b}", SQ_NAMES[from as usize]));


        let mut move_type = MoveType::Quiet;

        if piece < 2 && from.abs_diff(to) == 16 {
            move_type = MoveType::Double;
        } else if (piece == 10 || piece == 11) && from.abs_diff(to) == 2 {
            if from < to {
                move_type = if b.ctm == 0 { MoveType::WKingSide } else { MoveType::BKingSide };
            } else if from > to {
                move_type = if b.ctm == 0 { MoveType::WQueenSide } else { MoveType::BQueenSide };
            }
        }

        let mut xpiece = get_xpiece(b, to).unwrap_or(12);
        if  xpiece < 12 && promo_piece < 12 {
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
        } else if piece < 2 && to == b.ep as u32 {
            move_type = MoveType::Ep;
        } else if xpiece < 12 {
            move_type = MoveType::Cap;
        }

        Move::new(from, to, piece, xpiece, move_type)
    }

    pub fn as_uci_string(&self) -> String {
        let mut mv = String::new();

        let (f,t,_,x,m) = self.all();

        mv.push_str(SQ_NAMES[f]);
        mv.push_str(SQ_NAMES[t]);
        if m as u32 > 6 && (m as u32) < 12 {
            let promo_piece = if m == MoveType::Promo {
                x as u32
            } else {
                (m as u32) - 6
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
            f, "From: {} ({})\tTo:{} ({})\tPiece: {} ({})\tXPiece: {} ({})\tMove Type: {}",
            fr, SQ_NAMES[fr], t, SQ_NAMES[t], p, PIECE_NAMES[p], x, PIECE_NAMES[x], m 
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
    pub fn get_move_score(&self, m: Move, depth: usize) -> Option<i32> {
        self.killer_moves.get(depth)
            .and_then(|(k1, k2)| {
                if &Some(m) == k1 { Some(CAP_SCORE_OFFSET+1) }
                else if &Some(m) == k2 { Some(CAP_SCORE_OFFSET) }
                else { None }
            })
    }
}


pub trait HTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32;
}

pub struct HistoryTable {
    history: Vec<[[u32; 64]; 64]>
}

impl HistoryTable {
    pub fn new() -> HistoryTable {
        let mut history = Vec::with_capacity(2 * 64 * 64);
        history.push([[0; 64]; 64]);
        history.push([[0; 64]; 64]);
        HistoryTable { history }
    }

    pub fn insert(&mut self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
        self.history[colour_to_move][from][to] += (depth * depth) as u32
    }
}

impl HTable for HistoryTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
        self.history[colour_to_move][from][to]
    }
}

pub struct AtomicHistoryTable {
    history: Vec<AtomicU32>
}

impl AtomicHistoryTable {
    pub fn new() -> AtomicHistoryTable {
        let mut history = Vec::with_capacity(2 * 64 * 64);
        (0..2 * 64 * 64).for_each(|_| history.push(AtomicU32::new(0)));
        AtomicHistoryTable { history }
    }

    pub fn insert(&self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
        self.history[colour_to_move*64*64 + from*64 + to]
            .store((depth*depth) as u32, ORDER)
    }
}

impl HTable for AtomicHistoryTable {
    fn get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
        self.history[colour_to_move*64*64 + from*64 + to].load(ORDER)
    }
}
