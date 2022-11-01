use rand::distributions;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};
use crate::eval::{CHECKMATE, MATED};
use crate::Move;
use crate::tt::EntryType::PV;

// const TTABLE_SIZE: usize = 10;
//
const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;
// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

pub const ORDER: Ordering = Ordering::Release;

// pub const PV_TT_FLAG: u8 = 0;
// pub const ALPHA_TT_FLAG: u8 = 1;
// pub const BETA_TT_FLAG: u8 = 2;

#[derive(Debug, Copy, Clone)]
pub enum EntryType {
    PV,
    Alpha,
    Beta,
}

#[derive(Debug, Copy, Clone)]
struct TTEntry {
    hash: u64,
    score: i32,
    e_type: EntryType,
    depth: usize,
    best: Option<Move>,
}

impl TTEntry {
    pub fn default() -> TTEntry {
        TTEntry {
            hash: 0,
            score: 0,
            e_type: EntryType::PV,
            depth: 0,
            best: None,
        }
    }
}


// sequential transposition table
pub struct SeqTT {
    ttable: Box<[Option<TTEntry>]>,
    hit_count: usize,
    miss_count: usize,
}

impl SeqTT {
    pub fn new() -> SeqTT {
        SeqTT {
            ttable: (0..TTABLE_SIZE).map(|_| None).collect(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get_score(
        &mut self,
        hash: u64,
        depth: usize,
        alpha: i32,
        beta: i32,
        ply: usize
    ) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize].and_then(|entry| {
            if entry.hash != hash || entry.depth < depth {
                return None;
            }

            // if checkmate adjust the score so that it includes the amoiunt of plies up until
            // this point, the checkmate score should be stored so that it reflects the distance
            // between the mated node and this current one (not all the way up to the root)
            let score = if entry.score >= MATED {
                entry.score - ply as i32
            } else if entry.score <= CHECKMATE {
                entry.score + ply as i32
            } else {
                entry.score
            };

            match entry.e_type {
                EntryType::PV => Some(score),
                EntryType::Alpha => if score <= alpha { Some(score) } else { None },
                EntryType::Beta => if score >= beta { Some(score) } else { None }
            }
        })
    }

    #[inline]
    pub fn get_best(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize].and_then(|entry| entry.best)
    }

    pub fn insert(
        &mut self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize
    ) {
        // if checkmate make the score only the distance between this node and the checkmate
        // as opposed to the checkmate from the root
        let adjusted_score = if score >= MATED { score + ply as i32 }
            else if score <= CHECKMATE { score - ply as i32 }
            else { score };

        self.ttable[(hash & TT_IDX_MASK) as usize] = Some(TTEntry {
            hash,
            score: adjusted_score,
            e_type,
            depth,
            best
        });
    }
}




// parallel transposition table
pub struct ParaTT {
    ttable: Box<[RwLock<Option<TTEntry>>]>,
    hit_count: AtomicUsize,
    miss_count: AtomicUsize,
}

impl ParaTT {
    pub fn new() -> ParaTT {
        ParaTT {
            ttable: (0..TTABLE_SIZE).map(|_| RwLock::new(None)).collect(),
            hit_count: AtomicUsize::new(0),
            miss_count: AtomicUsize::new(0),
        }
    }

    pub fn get_score(
        &self,
        hash: u64,
        depth: usize,
        alpha: i32,
        beta: i32,
        ply: usize
    ) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .and_then(|entry| {
            if entry.hash != hash || entry.depth < depth {
                return None;
            }

            // if checkmate adjust the score so that it includes the amoiunt of plies up until
            // this point, the checkmate score should be stored so that it reflects the distance
            // between the mated node and this current one (not all the way up to the root)
            let score = if entry.score >= MATED {
                entry.score - ply as i32
            } else if entry.score <= CHECKMATE {
                entry.score + ply as i32
            } else {
                entry.score
            };

            match entry.e_type {
                EntryType::PV => Some(score),
                EntryType::Alpha => if score <= alpha { Some(score) } else { None },
                EntryType::Beta => if score >= beta { Some(score) } else { None }
            }
        })
    }

    #[inline]
    pub fn get_best(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .and_then(|entry| entry.best)
    }

    pub fn insert(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize
    ) {
        // if checkmate make the score only the distance between this node and the checkmate
        // as opposed to the checkmate from the root
        let adjusted_score = if score >= MATED { score + ply as i32 }
        else if score <= CHECKMATE { score - ply as i32 }
        else { score };

        *self.ttable[(hash & TT_IDX_MASK) as usize].write().unwrap() = Some(TTEntry {
            hash,
            score: adjusted_score,
            e_type,
            depth,
            best
        });
    }
}