use crate::board::Board;
use crate::movegen::moves::{KillerMoves, Move, PrevMoves};
use crate::search::eval::CHECKMATE;
use crate::search::tt::EntryScore;
use std::error::Error;
use std::fmt::{Display, Formatter};

use super::tt::TT;
use super::HistoryTable;

pub const MAX_DEPTH: usize = 500;
pub const MIN_SCORE: i32 = CHECKMATE * 2;
const MAX_SCORE: i32 = -MIN_SCORE;

pub mod single_searcher;
pub mod smp_searcher;

#[derive(Debug)]
pub enum SearchError {
    NoMove,
    FailLow,
    FailHigh,
}

impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::NoMove => writeln!(f, "No moves found"),
            SearchError::FailLow => writeln!(f, "Failed low"),
            SearchError::FailHigh => writeln!(f, "Failed high"),
        }
    }
}

impl Error for SearchError {}

pub type SeachResult = Result<(i32, Move), SearchError>;

pub trait Searcher<T: TT> {
    fn init_search(&mut self, b: &Board, depth: usize, time_limit_ms: u128);
    fn has_aborted(&self) -> bool;
    fn get_tt(&self) -> &T;
    fn get_tt_mut(&mut self) -> &mut T;
    fn get_km(&self) -> &KillerMoves;
    fn get_km_mut(&mut self) -> &mut KillerMoves;
    fn get_prev_moves(&self) -> &PrevMoves;
    fn get_prev_moves_mut(&mut self) -> &mut PrevMoves;
    fn get_hh(&self) -> &HistoryTable;
    fn get_hh_mut(&mut self) -> &mut HistoryTable;
    fn ply(&self) -> i32;
    fn draft(&self) -> i32;
    fn colour_multiplier(&self) -> i32;
    fn prev_moves(&self) -> &PrevMoves;
    fn push_ply(&mut self);
    fn push_prev_move(&mut self, hash: u64);
    fn pop_ply(&mut self);
    fn pop_prev_move(&mut self, hash: u64);
    fn get_hh_score(&self, ctm: usize, from: usize, to: usize) -> u32;
    fn store_hh_score(&mut self, ctm: usize, from: usize, to: usize, depth: usize);
    fn add_node(&mut self);

    fn probe_tt(&self, hash: u64, alpha: i32, beta: i32) -> Option<i32>;
    fn store_tt(&mut self, hash: u64, score: EntryScore, best_move: Option<Move>);
    fn get_tt_best_move(&self, hash: u64) -> Option<Move>;
    fn get_tt_pv_move(&mut self, hash: u64) -> Option<Move>;
    fn km_get(&self, depth: usize) -> [Option<Move>; 2];
    fn km_store(&mut self, km: Move, depth: usize);
}