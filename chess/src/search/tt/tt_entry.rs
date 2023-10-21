use crate::movegen::moves::{Move, NULL_MOVE};
use crate::search::eval::{CHECKMATE, MATED};
use crate::search::search::MAX_DEPTH;
use crate::search::tt::EntryType::PV;

const EMPTY_HASH: u64 = 0;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum EntryType {
    PV,
    Alpha,
    Beta,
}

#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    hash: u64,
    score: i32,
    e_type: EntryType,
    depth: u8,
    best: Move,
}

impl TTEntry {}

impl TTEntry {
    pub const fn empty() -> TTEntry {
        TTEntry {
            hash: EMPTY_HASH,
            score: 0,
            e_type: EntryType::Alpha,
            depth: 0,
            best: NULL_MOVE,
        }
    }

    pub fn get_score(
        &self,
        hash: u64,
        alpha: i32,
        beta: i32,
        depth: usize,
        ply: i32,
    ) -> Option<i32> {
        if self.hash != hash || (self.depth as usize) < depth {
            return None;
        }

        let score = adjust_score_retrieve(self.score, ply);

        match self.e_type {
            PV => Some(score),
            EntryType::Alpha => {
                if score <= alpha {
                    Some(alpha)
                } else {
                    None
                }
            }
            EntryType::Beta => {
                if score >= beta {
                    Some(beta)
                } else {
                    None
                }
            }
        }
    }

    pub fn get_bestmove(&self, hash: u64) -> Option<Move> {
        if self.hash == hash && self.best != NULL_MOVE {
            Some(self.best)
        } else {
            None
        }
    }

    pub fn get_pv(&self, hash: u64) -> Option<Move> {
        if self.hash == hash && self.best != NULL_MOVE && matches!(self.e_type, PV) {
            Some(self.best)
        } else {
            None
        }
    }

    pub fn update(
        &mut self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        if self.hash == hash && (self.depth as usize) > depth {
            return;
        }

        self.hash = hash;
        self.score = adjust_score_insert(score, ply as i32);
        self.e_type = e_type;
        self.depth = depth as u8;
        self.best = best.unwrap_or(NULL_MOVE);
    }
}

// https://www.ics.uci.edu/~eppstein/180a/990202a.html
fn adjust_score_insert(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 {
        score + ply
    } else if score < CHECKMATE - MAX_DEPTH as i32 {
        score - ply
    } else {
        score
    }
}

fn adjust_score_retrieve(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 {
        score - ply
    } else if score < CHECKMATE + MAX_DEPTH as i32 {
        score + ply
    } else {
        score
    }
}
