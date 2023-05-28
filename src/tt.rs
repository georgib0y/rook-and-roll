use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use crate::eval::{CHECKMATE, MATED};
use crate::Move;
use crate::search::MAX_DEPTH;
use crate::tt_entry::{AtomicTTEntry, Entry, EntryType, NoEntry, TTEntry};

const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;
// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;


pub const ORDER: Ordering = Ordering::SeqCst;

// pub type TTableST = TTable<Cell<TTEntry>>;
// pub type TTableMT = Arc<TTable<AtomicTTEntry>>;

pub trait TT {
    type TTEntryType: Entry;

    fn get_tt(&self) -> &[Self::TTEntryType];

    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.get_tt()[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply as i32)
    }

    fn get_best(&self, hash: u64) -> Option<Move> {
        self.get_tt()[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
    }

    fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.get_tt()[(hash & TT_IDX_MASK) as usize].update_entry(hash, score, e_type, depth, best, ply);
    }

    fn clear(&self) {
        self.get_tt().iter().for_each(|entry| entry.clear())
    }
    // fn get_arc(&self) -> TTableMT;
}

pub struct TTable {
    ttable: Box<[Cell<TTEntry>]>
}

impl TTable {
    pub fn new() -> TTable {
        TTable { ttable: (0..TTABLE_SIZE).map(|_| Cell::new(TTEntry::empty())).collect() }
    }
}

impl TT for TTable {
    type TTEntryType = Cell<TTEntry>;
    fn get_tt(&self) -> &[Self::TTEntryType] { self.ttable.as_ref() }
}

#[derive(Debug, Clone)]
pub struct AtomicTTable {
    ttable: Arc<Box<[AtomicTTEntry]>>
}

impl AtomicTTable {
    pub fn new() -> Arc<AtomicTTable> {
        Arc::new(AtomicTTable {
            ttable: Arc::new((0..TTABLE_SIZE).map(|_| AtomicTTEntry::empty()).collect())
        })
    }
}

impl TT for Arc<AtomicTTable> {
    type TTEntryType = AtomicTTEntry;
    fn get_tt(&self) -> &[Self::TTEntryType] { self.ttable.as_ref() }
}

struct NoTTable;

impl TT for NoTTable {
    type TTEntryType = NoEntry;
    fn get_tt(&self) -> &[Self::TTEntryType] { &[] }
    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> { None }
    fn get_best(&self, hash: u64) -> Option<Move> { None }
    fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {}
    fn clear(&self) {}
}

// pub struct TTable<T: Entry> {
//     ttable: Box<[T]>,
// }
//
// impl<T: Entry> TTable<T> {
//     pub fn new_single_threaded() -> TTableST {
//         TTable { ttable: (0..TTABLE_SIZE).map(|_| Cell::new(TTEntry::empty())).collect() }
//     }
//
//     pub fn new_multi_threaded() -> TTableMT {
//         Arc::new(TTable { ttable: (0..TTABLE_SIZE).map(|_| AtomicTTEntry::empty()).collect() })
//     }
//
//     pub fn tt_get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         self.ttable[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply as i32)
//     }
//
//     pub fn tt_get_best(&self, hash: u64) -> Option<Move> {
//         self.ttable[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
//     }
//
//     pub fn tt_insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
//         self.ttable[(hash & TT_IDX_MASK) as usize].update_entry(hash, score, e_type, depth, best, ply);
//     }
//
//     pub fn tt_clear(&self) {
//         self.ttable.iter().for_each(|entry| entry.clear())
//     }
// }
//
//
// impl TT for TTableST {
//     fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         self.tt_get_score(hash, depth, alpha, beta, ply)
//     }
//
//     fn get_best(&self, hash: u64) -> Option<Move> {
//         self.tt_get_best(hash)
//     }
//
//     fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
//         self.tt_insert(hash, score, e_type, depth, best, ply)
//     }
//
//     fn clear(&self) {
//         self.tt_clear()
//     }
//
//     fn get_arc(&self) -> TTableMT { panic!("Tried to make arc of single threaded TT!") }
// }
//
// impl TT for TTableMT {
//     fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         self.tt_get_score(hash, depth, alpha, beta, ply)
//     }
//
//     fn get_best(&self, hash: u64) -> Option<Move> {
//         self.tt_get_best(hash)
//     }
//
//     fn insert(&self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
//         self.tt_insert(hash, score, e_type, depth, best, ply)
//     }
//
//     fn clear(&self) {
//         self.tt_clear()
//     }
//
//     fn get_arc(&self) -> TTableMT { Arc::clone(self) }
// }


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
// pub struct SeqTT {
//     ttable: Box<[TTEntry]>,
//     pub hits: usize,
//     pub misses: usize,
// }
//
// impl SeqTT {
//     pub fn new() -> SeqTT {
//         SeqTT {
//             ttable: (0..TTABLE_SIZE).map(|_| TTEntry::empty()).collect(),
//             hits: 0,
//             misses: 0,
//         }
//     }
//
//     pub fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         let entry = self.ttable[(hash & TT_IDX_MASK) as usize];
//         if entry.hash == EMPTY_HASH {
//             return None;
//         }
//         entry.entry_get_score(hash, alpha, beta, depth, ply as i32)
//     }
//
//     #[inline]
//     pub fn get_best(&self, hash: u64) -> Option<Move> {
//         let entry = self.ttable[(hash & TT_IDX_MASK) as usize];
//         if entry.hash == hash {
//             entry.best
//         } else {
//             None
//         }
//     }
//
//     pub fn insert(
//         &mut self,
//         hash: u64,
//         score: i32,
//         e_type: EntryType,
//         depth: usize,
//         best: Option<Move>,
//         ply: usize
//     ) {
//         // if checkmate make the score only the distance between this node and the checkmate
//         // as opposed to the checkmate from the root
//         let entry = &mut self.ttable[(hash & TT_IDX_MASK) as usize];
//
//         if entry.hash == hash && entry.depth > depth { return; }
//
//         entry.hash = hash;
//         entry.score = adjust_score_insert(score, ply as i32);
//         entry.e_type = e_type;
//         entry.depth = depth;
//         entry.best = best;
//     }
//
//     pub fn clear(&mut self) {
//         self.ttable.iter_mut().for_each(|entry| *entry = TTEntry::empty())
//     }
// }
//
// // parallel transposition table
// pub struct ParaTT {
//     ttable: Box<[RwLock<Option<TTEntry>>]>,
//     pub hits: AtomicUsize,
//     pub misses: AtomicUsize,
// }
//
// impl ParaTT {
//     pub fn new() -> ParaTT {
//         ParaTT {
//             ttable: (0..TTABLE_SIZE).map(|_| RwLock::new(None)).collect(),
//             hits: AtomicUsize::new(0),
//             misses: AtomicUsize::new(0),
//         }
//     }
//
//     pub fn get_score( &self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize ) -> Option<i32> {
//         self.ttable[(hash & TT_IDX_MASK) as usize]
//             .read()
//             .unwrap()
//             .and_then(|entry| entry.entry_get_score(hash, alpha, beta, depth, ply as i32))
//     }
//
//     #[inline]
//     pub fn get_best(&self, hash: u64) -> Option<Move> {
//         self.ttable[(hash & TT_IDX_MASK) as usize]
//             .read()
//             .unwrap()
//             .and_then(|entry| entry.best)
//     }
//
//     pub fn insert(
//         &self,
//         hash: u64,
//         score: i32,
//         e_type: EntryType,
//         depth: usize,
//         best: Option<Move>,
//         ply: usize
//     ) {
//         // if checkmate make the score only the distance between this node and the checkmate
//         // as opposed to the checkmate from the root
//
//         let score = adjust_score_insert(score, ply as i32);
//         let mut lock = self.ttable[(hash & TT_IDX_MASK) as usize]
//             .write()
//             .unwrap();
//
//         if let Some(entry) = lock.as_mut() {
//             if entry.depth > depth { return; }
//             *entry = TTEntry { hash, score, e_type, depth, best }
//         } else {
//             *lock = Some(TTEntry { hash, score, e_type, depth, best });
//         }
//
//     }
//
//     pub fn clear(&self) {
//         self.ttable.iter()
//             .map(|entry| entry.write().unwrap())
//             .for_each(|mut entry| *entry = None)
//     }
// }

