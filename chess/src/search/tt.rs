use crate::movegen::moves::Move;
use crate::search::tt_entry::{AtomicTTEntry, Entry, EntryType, NoEntry, TTEntry};
use std::cell::Cell;
use std::sync::atomic::Ordering;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct AtomicTTable {
    ttable: Box<[AtomicTTEntry]>,
}

impl AtomicTTable {
    pub fn new() -> Arc<AtomicTTable> {
        Arc::new(AtomicTTable {
            ttable: (0..TTABLE_SIZE).map(|_| AtomicTTEntry::empty()).collect(),
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
    fn get_score(
        &self,
        _hash: u64,
        _depth: usize,
        _alpha: i32,
        _beta: i32,
        _ply: usize,
    ) -> Option<i32> {
        None
    }

    fn get_best(&self, _hash: u64) -> Option<Move> {
        None
    }
    fn insert(
        &self,
        _hash: u64,
        _score: i32,
        _e_type: EntryType,
        _depth: usize,
        _best: Option<Move>,
        _ply: usize,
    ) {
    }
    fn clear(&self) {}
}
