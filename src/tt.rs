use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use log::info;
use crate::eval::{CHECKMATE, MATED};
use crate::Move;
use crate::search::MAX_DEPTH;

const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;
// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

const EMPTY_HASH: u64 = 0;

pub const ORDER: Ordering = Ordering::SeqCst;

pub type TTableST = TTable<RefCell<TTEntry>>;
pub type TTableMT = Arc<TTable<AtomicTTEntry>>;

pub trait Entry {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32>;
    fn get_bestmove(&self, hash: u64) -> Option<Move>;
    fn update_entry(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize);
    fn clear(&self);
}

impl Entry for RefCell<TTEntry> {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        self.borrow().entry_get_score(hash, alpha, beta, depth, ply)
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.borrow().entry_get_bestmove(hash)
    }

    fn update_entry(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.borrow_mut().entry_update(hash, score, e_type, depth, best, ply)
    }

    fn clear(&self) { self.borrow_mut().entry_clear() }
}

impl Entry for AtomicTTEntry {
    fn get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        self.entry_get_score(hash, alpha, beta, depth, ply)
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.entry_get_bestmove(hash)
    }

    fn update_entry(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.entry_update(hash, score, e_type, depth, best, ply);
    }

    fn clear(&self) {
        self.entry_clear()
    }
}


#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum EntryType { PV, Alpha, Beta }


#[derive(Debug, Copy, Clone)]
pub struct TTEntry {
    hash: u64,
    score: i32,
    e_type: EntryType,
    depth: usize,
    best: Option<Move>,
}

impl TTEntry {
    const fn empty() -> TTEntry {
        TTEntry {hash: EMPTY_HASH, score: 0, e_type: EntryType::Alpha, depth: 0, best: None}
    }

    fn entry_get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        if self.hash != hash || self.depth < depth { return None; }

        let score = adjust_score_retrieve(self.score, ply);

        match self.e_type {
            EntryType::PV => Some(score),
            EntryType::Alpha => if score <= alpha { Some(alpha) } else { None },
            EntryType::Beta => if score >= beta { Some(beta) } else { None }
        }
    }

    fn entry_get_bestmove(&self, hash: u64) -> Option<Move> {
        if self.hash == hash { self.best } else { None }
    }

    fn entry_update (
        &mut self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize
    ) {
        if self.hash == hash && self.depth > depth { return; }

        self.hash = hash;
        self.score = adjust_score_insert(score, ply as i32);
        self.e_type = e_type;
        self.depth = depth;
        self.best = best;
    }

    fn entry_clear(&mut self) { *self = TTEntry::empty() }
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
    const fn empty() -> AtomicTTEntry {
        AtomicTTEntry {
            hash: AtomicU64::new(EMPTY_HASH),
            score: AtomicI32::new(0),
            e_type: AtomicU8::new(0),
            depth: AtomicUsize::new(0),
            best: RwLock::new(None)
        }
    }

    fn entry_get_score(&self, hash: u64, alpha: i32, beta: i32, depth: usize, ply: i32) -> Option<i32> {
        if self.hash.load(ORDER) != hash || self.depth.load(ORDER) < depth { return None; }

        let score = adjust_score_retrieve(self.score.load(ORDER), ply);

        // TODO FIX if actually works
        match unsafe { std::mem::transmute::<u8, EntryType>(self.e_type.load(ORDER)) } {
            EntryType::PV  => Some(score),
            EntryType::Alpha => if score <= alpha { Some(alpha) } else { None },
            EntryType::Beta => if score >= beta { Some(beta) } else { None }
        }
    }

    fn entry_get_bestmove(&self, hash: u64) -> Option<Move> {
        if self.hash.load(ORDER) == hash { *self.best.read().unwrap() } else { None }
    }

    fn entry_update (
        &self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize
    ) {
        if self.hash.load(ORDER) == hash && self.depth.load(ORDER) > depth { return; }

        self.hash.store(hash, ORDER);
        self.score.store(adjust_score_insert(score, ply as i32), ORDER);
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


// https://www.ics.uci.edu/~eppstein/180a/990202a.html
fn adjust_score_insert(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 { score + ply }
    else if score < CHECKMATE - MAX_DEPTH as i32 { score - ply }
    else { score }
}

fn adjust_score_retrieve(score: i32, ply: i32) -> i32 {
    if score > MATED - MAX_DEPTH as i32 { score - ply }
    else if score < CHECKMATE + MAX_DEPTH as i32 { score + ply }
    else { score }
}

pub struct TTable<T: Entry> {
    ttable: Box<[T]>,
}

impl<T: Entry> TTable<T> {
    pub fn new_single_threaded() -> TTableST {
        TTable { ttable: (0..TTABLE_SIZE).map(|_| RefCell::new(TTEntry::empty())).collect() }
    }

    pub fn new_multi_threaded() -> TTableMT {
        Arc::new(TTable { ttable: (0..TTABLE_SIZE).map(|_| AtomicTTEntry::empty()).collect() })
    }

    pub fn tt_get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply as i32)
    }

    pub fn tt_get_best(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
    }

    pub fn tt_insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.ttable[(hash & TT_IDX_MASK) as usize].update_entry(hash, score, e_type, depth, best, ply);
    }

    pub fn tt_clear(&self) {
        self.ttable.iter().for_each(|entry| entry.clear())
    }
}

pub trait TT {
    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32>;
    fn get_best(&self, hash: u64) -> Option<Move>;
    fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize);
    fn clear(&self);
    fn get_arc(&self) -> TTableMT;
}

impl TT for TTableST {
    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.tt_get_score(hash, depth, alpha, beta, ply)
    }

    fn get_best(&self, hash: u64) -> Option<Move> {
        self.tt_get_best(hash)
    }

    fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.tt_insert(hash, score, e_type, depth, best, ply)
    }

    fn clear(&self) {
        self.tt_clear()
    }

    fn get_arc(&self) -> TTableMT { panic!("Tried to make arc of single threaded TT!") }
}

impl TT for TTableMT {
    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.tt_get_score(hash, depth, alpha, beta, ply)
    }

    fn get_best(&self, hash: u64) -> Option<Move> {
        self.tt_get_best(hash)
    }

    fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.tt_insert(hash, score, e_type, depth, best, ply)
    }

    fn clear(&self) {
        self.tt_clear()
    }

    fn get_arc(&self) -> TTableMT { Arc::clone(self) }
}


//
// impl TT for TTableST {
//
//
//     fn get_arc(&self) -> TTableMT {
//         panic!("Tried to make arc of single threaded ttable!")
//     }
// }
//
// impl TT for TTableMT {
//     fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         self.ttable[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply as i32)
//     }
//
//     fn get_best(&self, hash: u64) -> Option<Move> {
//         self.ttable[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
//     }
//
//     fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
//         self.ttable[(hash & TT_IDX_MASK) as usize].update_entry(hash, score, e_type, depth, best, ply);
//     }
//
//     fn clear(&self) { self.ttable.iter().for_each(|entry| entry.clear()) }
//
//     fn get_arc(&self) -> TTableMT { Arc::clone(self) }
// }


// sequential transposition table
pub struct SeqTT {
    ttable: Box<[TTEntry]>,
    pub hits: usize,
    pub misses: usize,
}

impl SeqTT {
    pub fn new() -> SeqTT {
        SeqTT {
            ttable: (0..TTABLE_SIZE).map(|_| TTEntry::empty()).collect(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        let entry = self.ttable[(hash & TT_IDX_MASK) as usize];
        if entry.hash == EMPTY_HASH {
            return None;
        }
        entry.entry_get_score(hash, alpha, beta, depth, ply as i32)
    }

    #[inline]
    pub fn get_best(&self, hash: u64) -> Option<Move> {
        let entry = self.ttable[(hash & TT_IDX_MASK) as usize];
        if entry.hash == hash {
            entry.best
        } else {
            None
        }
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
        let entry = &mut self.ttable[(hash & TT_IDX_MASK) as usize];

        if entry.hash == hash && entry.depth > depth { return; }

        entry.hash = hash;
        entry.score = adjust_score_insert(score, ply as i32);
        entry.e_type = e_type;
        entry.depth = depth;
        entry.best = best;
    }

    pub fn clear(&mut self) {
        self.ttable.iter_mut().for_each(|entry| *entry = TTEntry::empty())
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
            .and_then(|entry| entry.entry_get_score(hash, alpha, beta, depth, ply as i32))
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

