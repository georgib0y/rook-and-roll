use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{RwLock};
use crate::eval::{CHECKMATE, MATED};
use crate::Move;
use crate::search::MAX_DEPTH;

const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;
// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;


pub const ORDER: Ordering = Ordering::SeqCst;

#[repr(u8)]
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
    pub fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        if self.hash != hash || self.depth < depth { return None; }

        // if checkmate adjust the score so that it includes the amount of plies up until
        // this point, the checkmate score should be stored so that it reflects the distance
        // between the mated node and this current one (not all the way up to the root)
        let score = adjust_score_retrieve(self.score, ply);

        match self.e_type {
            EntryType::PV => Some(score),
            EntryType::Alpha => if score <= alpha { Some(alpha) } else { None },
            EntryType::Beta => if score >= beta { Some(beta) } else { None }
        }
    }
}
// https://www.ics.uci.edu/~eppstein/180a/990202a.html
fn adjust_score_insert(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 { score + ply }
    else if score < CHECKMATE - MAX_DEPTH as i32 { score - ply }
    else { score }
}

// https://www.ics.uci.edu/~eppstein/180a/990202a.html
fn adjust_score_retrieve(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 { score - ply }
    else if score < CHECKMATE + MAX_DEPTH as i32 { score + ply }
    else { score }
}

// sequential transposition table
pub struct SeqTT {
    ttable: Box<[Option<TTEntry>]>,
    pub hits: usize,
    pub misses: usize,
}

impl SeqTT {
    pub fn new() -> SeqTT {
        SeqTT {
            ttable: (0..TTABLE_SIZE).map(|_| None).collect(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .and_then(|entry| entry.get_score(hash, alpha, beta, depth, ply as i32))
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
        let score = adjust_score_insert(score, ply as i32);

        // self.ttable[(hash & TT_IDX_MASK) as usize] = Some(TTEntry {
        //     hash, score, e_type, depth, best
        // });

        if let Some(entry) = self.ttable[(hash & TT_IDX_MASK) as usize]{
            if entry.hash == hash && entry.depth > depth { return; }
            self.ttable[(hash & TT_IDX_MASK) as usize] = Some(TTEntry {
                hash, score, e_type, depth, best
            });
        } else {
            self.ttable[(hash & TT_IDX_MASK) as usize] = Some(TTEntry {
                hash, score, e_type, depth, best
            });
        }
    }

    pub fn clear(&mut self) {
        self.ttable.iter_mut().for_each(|entry| *entry = None)
    }
}

// parallel transposition table
pub struct ParaTT {
    ttable: Box<[RwLock<Option<TTEntry>>]>,
    pub hits: AtomicUsize,
    pub misses: AtomicUsize,
}

impl ParaTT {
    pub fn new() -> ParaTT {
        ParaTT {
            ttable: (0..TTABLE_SIZE).map(|_| RwLock::new(None)).collect(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    pub fn get_score( &self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize ) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .and_then(|entry| entry.get_score(hash, alpha, beta, depth, ply as i32))
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

        let score = adjust_score_insert(score, ply as i32);
        let mut lock = self.ttable[(hash & TT_IDX_MASK) as usize]
            .write()
            .unwrap();

        if let Some(entry) = lock.as_mut() {
            if entry.depth > depth { return; }
            *entry = TTEntry { hash, score, e_type, depth, best }
        } else {
            *lock = Some(TTEntry { hash, score, e_type, depth, best });
        }

    }

    pub fn clear(&self) {
        self.ttable.iter()
            .map(|entry| entry.write().unwrap())
            .for_each(|mut entry| *entry = None)
    }
}
