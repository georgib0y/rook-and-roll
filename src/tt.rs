use lazy_static::lazy_static;
use rand::distributions;
use rand::distributions::Standard;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::RwLock;

// const TTABLE_SIZE: usize = 10;
//
const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;

// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

pub const PV_TT_FLAG: u8 = 0;
pub const ALPHA_TT_FLAG: u8 = 1;
pub const BETA_TT_FLAG: u8 = 2;

pub const ORDER: Ordering = Ordering::Release;

/*
TODO
multithreaded tt table is slower becuase memory is being shared between all cores of the cpu (slow)

AN idea to minimize this (could be a bad idea but it's MY idea at least):
    what if each thread searching has its own TTable, which was some sort of slice or subset of
    a master tt

    each is likely to encounter the same positions in a localised scale, which is what the tt would
    be used to minimise
        (starting out the search, all threads would all have similar positions as well which would
        be where the master tt would come in to play)

    after a search is completed somehow the local tt are the merged into the master tt by some
    heuristic such as times accessed/age and depth or something

    though the size would be an issue, 6/12 cores/threads would be huge if the tt was copied that
    many times, so then a smaller tt would need to be copied to each thread but how?? and by what???

 */

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

    pub fn get(&mut self, hash: u64, depth: usize, alpha: i32, beta: i32) -> Option<i32> {
        if let Some(entry) = self.ttable[(hash & TT_IDX_MASK) as usize] {
            if entry.hash != hash || entry.depth < depth {
                return None;
            }

            if entry.e_type == PV_TT_FLAG {
                Some(entry.score)
            } else if entry.e_type == ALPHA_TT_FLAG && entry.score <= alpha {
                Some(alpha)
            } else if entry.e_type == BETA_TT_FLAG && entry.score >= beta {
                Some(beta)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: u64, score: i32, e_type: u8, depth: usize) {
        let entry = TTEntry {
            hash,
            score,
            e_type,
            depth,
        };
        self.ttable[(hash & TT_IDX_MASK) as usize] = Some(entry);
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

#[derive(Debug, Copy, Clone)]
struct TTEntry {
    hash: u64,
    score: i32,
    e_type: u8,
    depth: usize,
}

impl TTEntry {
    pub fn default() -> TTEntry {
        TTEntry {
            hash: 0,
            score: 0,
            e_type: PV_TT_FLAG,
            depth: 0,
        }
    }
}

struct AtomicTTEntry {
    hash: AtomicU64,
    score: AtomicI32,
    e_type: AtomicU8,
}

impl AtomicTTEntry {
    pub fn default() -> TTEntry {
        TTEntry {
            hash: 0,
            score: 0,
            e_type: PV_TT_FLAG,
            depth: 0,
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
