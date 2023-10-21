use crate::movegen::moves::Move;
use crate::search::tt::tt_entry::TTEntry;
use crate::search::tt::{EntryType, TTABLE_SIZE, TT_IDX_MASK};
use std::sync::{Arc, RwLock};

pub struct SmpTTable {
    ttable: Box<[RwLock<TTEntry>]>,
}

impl SmpTTable {
    pub fn new() -> Arc<SmpTTable> {
        Arc::new(SmpTTable {
            ttable: (0..TTABLE_SIZE)
                .map(|_| RwLock::new(TTEntry::empty()))
                .collect(),
        })
    }

    pub fn get_score(
        &self,
        hash: u64,
        depth: usize,
        alpha: i32,
        beta: i32,
        ply: i32,
    ) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .get_score(hash, alpha, beta, depth, ply)
    }

    pub fn get_best(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .get_bestmove(hash)
    }

    pub fn get_best_pv(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .read()
            .unwrap()
            .get_pv(hash)
    }

    pub fn insert(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        self.ttable[(hash & TT_IDX_MASK) as usize]
            .write()
            .unwrap()
            .update(hash, score, e_type, depth, best, ply);
    }

    pub fn clear(&self) {
        self.ttable
            .iter()
            .map(|rw| rw.write().unwrap())
            .for_each(|mut entry| *entry = TTEntry::empty())
    }
}
