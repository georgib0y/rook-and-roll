use crate::eval::{CHECKMATE, MATED};
use crate::moves::{Move, NULL_MOVE};
use crate::search::MAX_DEPTH;
use crate::tt::ORDER;
use std::cell::Cell;
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize};
use std::sync::RwLock;

const EMPTY_HASH: u64 = 0;

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum EntryType {
    PV,
    Alpha,
    Beta,
}

pub trait Entry {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32>;
    fn get_bestmove(&self, hash: u64) -> Option<Move>;
    fn update_entry(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    );
    fn clear(&self);
}

#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    hash: u64,
    score: i32,
    e_type: EntryType,
    depth: u8,
    best: Move,
}

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

    fn entry_get_score(
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
            EntryType::PV => Some(score),
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

    fn entry_get_bestmove(&self, hash: u64) -> Option<Move> {
        if self.hash == hash && self.best != NULL_MOVE {
            Some(self.best)
        } else {
            None
        }
    }

    fn entry_update(
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

impl Entry for Cell<TTEntry> {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        self.get().entry_get_score(hash, alpha, beta, depth, ply)
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.get().entry_get_bestmove(hash)
    }

    fn update_entry(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        let mut entry = self.get();
        entry.entry_update(hash, score, e_type, depth, best, ply);
        self.set(entry);
    }

    fn clear(&self) {
        self.set(TTEntry::empty())
    }
}

#[derive(Debug)]
pub struct AtomicTTEntry {
    hash: AtomicU64,
    score: AtomicI32,
    e_type: AtomicU8,
    depth: AtomicUsize,
    best: RwLock<Option<Move>>,
}

impl AtomicTTEntry {
    pub const fn empty() -> AtomicTTEntry {
        AtomicTTEntry {
            hash: AtomicU64::new(EMPTY_HASH),
            score: AtomicI32::new(0),
            e_type: AtomicU8::new(0),
            depth: AtomicUsize::new(0),
            best: RwLock::new(None),
        }
    }

    fn get_e_type(&self) -> EntryType {
        unsafe { std::mem::transmute::<u8, EntryType>(self.e_type.load(ORDER)) }
    }

    fn entry_get_score(
        &self,
        hash: u64,
        alpha: i32,
        beta: i32,
        depth: usize,
        ply: i32,
    ) -> Option<i32> {
        if self.hash.load(ORDER) != hash || self.depth.load(ORDER) < depth {
            return None;
        }

        let score = adjust_score_retrieve(self.score.load(ORDER), ply);

        match self.get_e_type() {
            EntryType::PV => Some(score),
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

    fn entry_get_bestmove(&self, hash: u64) -> Option<Move> {
        if self.hash.load(ORDER) == hash {
            *self.best.read().unwrap()
        } else {
            None
        }
    }

    fn entry_update(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        if self.hash.load(ORDER) == hash && self.depth.load(ORDER) > depth {
            return;
        }

        self.hash.store(hash, ORDER);
        self.score
            .store(adjust_score_insert(score, ply as i32), ORDER);
        self.e_type.store(e_type as u8, ORDER);
        self.depth.store(depth, ORDER);
        *self.best.write().unwrap() = best;
    }

    fn entry_clear(&self) {
        self.hash.store(EMPTY_HASH, ORDER);
        self.score.store(0, ORDER);
        self.e_type.store(0, ORDER);
        self.depth.store(0, ORDER);
        *self.best.write().unwrap() = None;
    }
}

impl Entry for AtomicTTEntry {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        self.entry_get_score(hash, alpha, beta, depth, ply)
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.entry_get_bestmove(hash)
    }

    fn update_entry(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        self.entry_update(hash, score, e_type, depth, best, ply);
    }

    fn clear(&self) {
        self.entry_clear()
    }
}

pub struct NoEntry;

impl Entry for NoEntry {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        None
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        None
    }

    fn update_entry(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
    }

    fn clear(&self) {}
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
