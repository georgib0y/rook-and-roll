use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::RwLock;
use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use rand::distributions;
use rand::distributions::Standard;

// const TTABLE_SIZE: usize = 10;
//
const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;

// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

pub const SET_TT_FLAG: u8 = 0;
pub const UNSET_TT_FLAG: u8 = 0;

pub const ORDER: Ordering = Ordering::Release;

lazy_static!{
    pub static ref ZORB: Box<[u64]> = init_zorbist_array();
}


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
    ttable: Box<[TTEntry]>,
    hit_count: usize,
    miss_count: usize
}

impl SeqTT {
    pub fn new() -> SeqTT {
        SeqTT {
            ttable: (0..TTABLE_SIZE)
                .map(|_| TTEntry::random_entry())
                .collect(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    pub fn get(&mut self, hash: u64, alpha: i32, beta: i32) -> Option<i32> {
        let entry = self.ttable[(hash & TT_IDX_MASK) as usize];
        if entry.e_type == UNSET_TT_FLAG {
            None
        } else {
            Some(entry.score)
        }
    }

    pub fn insert(&mut self, hash: u64, score: i32, e_type: u8) {
        let entry = TTEntry { hash, score, e_type };
        self.ttable[(hash & TT_IDX_MASK) as usize] = entry;
    }

}

// parallel transposition table
pub struct AtomicTT {
    ttable: Box<[AtomicTTEntry]>,
    hit_count: usize,
    miss_count: usize
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
        let entry = &self.ttable[(hash & TT_IDX_MASK) as usize];

        if entry.e_type.load(ORDER) == UNSET_TT_FLAG {
            None
        } else {
            Some(entry.score.load(ORDER))
        }
    }

    pub fn insert(&self, hash: u64, score: i32, e_type: u8) {
        self.ttable[(hash & TT_IDX_MASK) as usize].hash.store(hash, ORDER);
        self.ttable[(hash & TT_IDX_MASK) as usize].score.store(score, ORDER);
        self.ttable[(hash & TT_IDX_MASK) as usize].e_type.store(e_type, ORDER);
    }

}

#[derive(Debug, Copy, Clone)]
pub enum EntryType {
    Set,
    Unset,
}

impl Distribution<EntryType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EntryType {
        match rng.gen_range(0..=2) {
            0 => EntryType::Set,
            _ => EntryType::Unset,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct TTEntry {
    hash: u64,
    score: i32,
    e_type: u8,
}

impl TTEntry {
    pub fn default() -> TTEntry {
        TTEntry { hash: 0, score: 0, e_type: UNSET_TT_FLAG }
    }

    pub fn random_entry() -> TTEntry {
        TTEntry {
            hash: rand::thread_rng().gen(),
            score: rand::thread_rng().gen(),
            e_type: rand::thread_rng().gen_range(0..2)
        }
    }
}

struct AtomicTTEntry {
    hash: AtomicU64,
    score: AtomicI32,
    e_type: AtomicU8
}

impl AtomicTTEntry {
    pub fn default() -> TTEntry {
        TTEntry { hash: 0, score: 0, e_type: UNSET_TT_FLAG }
    }

    pub fn random_entry() -> AtomicTTEntry {
        AtomicTTEntry {
            hash: AtomicU64::new(rand::thread_rng().gen()),
            score: AtomicI32::new(rand::thread_rng().gen()),
            e_type: AtomicU8::new(rand::thread_rng().gen_range(0..2))
        }
    }
}

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
fn init_zorbist_array() -> Box<[u64]> {
    let mut zorbist_array: [u64; 781] = [0; 781];

    let mut z_hash = 0u64;
    // may be a good seed or may not be (could try flipping the reverse around if not)
    let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);

    for z in &mut zorbist_array  {
        *z = prng.gen::<u64>();
        z_hash ^= *z;
    }


    Box::new(zorbist_array)
}
