use lazy_static::lazy_static;
use rand::distributions;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::RwLock;
use crate::eval::CHECKMATE;
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
            let score = if entry.score >= -CHECKMATE {
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
    pub fn get_best(&mut self, hash: u64) -> Option<Move> {
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
        let score = if score >= -CHECKMATE { score + ply as i32 }
            else if score <= CHECKMATE { score - ply as i32 }
            else { score };

        self.ttable[(hash & TT_IDX_MASK) as usize] = Some(TTEntry {
            hash,
            score,
            e_type,
            depth,
            best
        });
    }
}




// parallel transposition table
pub struct AtomicTT {
    ttable: Box<[AtomicTTEntry]>,
    hit_count: usize,
    miss_count: usize,
}

impl AtomicTT {
    pub fn new() -> AtomicTT {
        AtomicTT {
            ttable: (0..TTABLE_SIZE)
                .map(|_| AtomicTTEntry::random_entry())
                .collect(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&self, hash: u64, alpha: i32, beta: i32) -> Option<i32> {
        todo!()
    }

    pub fn insert(&self, hash: u64, score: i32, e_type: u8) {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .hash
            .store(hash, ORDER);
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .score
            .store(score, ORDER);
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .e_type
            .store(e_type, ORDER);
    }
}



struct AtomicTTEntry {
    hash: AtomicU64,
    score: AtomicI32,
    e_type: AtomicU8,
}

impl AtomicTTEntry {
    pub fn default() -> AtomicTTEntry {
        AtomicTTEntry {
            hash: AtomicU64::new(0),
            score: AtomicI32::new(0),
            e_type: AtomicU8::new(0),

        }
    }

    pub fn random_entry() -> AtomicTTEntry {
        AtomicTTEntry {
            hash: AtomicU64::new(rand::thread_rng().gen()),
            score: AtomicI32::new(rand::thread_rng().gen()),
            e_type: AtomicU8::new(rand::thread_rng().gen_range(0..2)),
        }
    }
}
