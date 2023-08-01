use crate::eval::{CHECKMATE, MATED};
use crate::search::MAX_DEPTH;
use crate::tt_entry::{AtomicTTEntry, Entry, EntryType, NoEntry, TTEntry};
use crate::Move;
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicI32, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

const TTABLE_SIZE: usize = 1 << 20; // 2^20
const TT_IDX_MASK: u64 = 0xFFFFF;
// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

pub const ORDER: Ordering = Ordering::SeqCst;

pub trait TT {
    type TTEntryType: Entry;

    fn get_tt(&self) -> &[Self::TTEntryType];

    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.get_tt()[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply as i32)
    }

    fn get_best(&self, hash: u64) -> Option<Move> {
        self.get_tt()[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
    }

    fn insert(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        self.get_tt()[(hash & TT_IDX_MASK) as usize]
            .update_entry(hash, score, e_type, depth, best, ply);
    }

    fn clear(&self) {
        self.get_tt().iter().for_each(|entry| entry.clear())
    }
    // fn get_arc(&self) -> TTableMT;
}

pub struct TTable {
    ttable: Box<[Cell<TTEntry>]>,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            ttable: (0..TTABLE_SIZE)
                .map(|_| Cell::new(TTEntry::empty()))
                .collect(),
        }
    }
}

impl TT for TTable {
    type TTEntryType = Cell<TTEntry>;
    fn get_tt(&self) -> &[Self::TTEntryType] {
        self.ttable.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct AtomicTTable {
    ttable: Arc<Box<[AtomicTTEntry]>>,
}

impl AtomicTTable {
    pub fn new() -> Arc<AtomicTTable> {
        Arc::new(AtomicTTable {
            ttable: Arc::new((0..TTABLE_SIZE).map(|_| AtomicTTEntry::empty()).collect()),
        })
    }
}

impl TT for Arc<AtomicTTable> {
    type TTEntryType = AtomicTTEntry;
    fn get_tt(&self) -> &[Self::TTEntryType] {
        self.ttable.as_ref()
    }
}

struct NoTTable;

impl TT for NoTTable {
    type TTEntryType = NoEntry;
    fn get_tt(&self) -> &[Self::TTEntryType] {
        &[]
    }
    fn get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        None
    }
    fn get_best(&self, hash: u64) -> Option<Move> {
        None
    }
    fn insert(
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
