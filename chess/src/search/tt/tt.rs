use crate::movegen::moves::Move;
use crate::search::tt::tt_entry::{EntryType, TTEntry};
use crate::search::tt::{TTABLE_SIZE, TT_IDX_MASK};

pub struct TTable {
    ttable: Box<[TTEntry]>,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            ttable: (0..TTABLE_SIZE).map(|_| TTEntry::empty()).collect(),
        }
    }

    pub fn get_score(
        &self,
        hash: u64,
        depth: usize,
        alpha: i32,
        beta: i32,
        ply: i32,
    ) -> Option<i32> {
        self.ttable[(hash & TT_IDX_MASK) as usize].get_score(hash, alpha, beta, depth, ply)
    }

    pub fn get_best(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize].get_bestmove(hash)
    }

    pub fn get_best_pv(&self, hash: u64) -> Option<Move> {
        self.ttable[(hash & TT_IDX_MASK) as usize].get_pv(hash)
    }

    pub fn insert(
        &mut self,
        hash: u64,
        score: i32,
        e_type: EntryType,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        self.ttable[(hash & TT_IDX_MASK) as usize].update(hash, score, e_type, depth, best, ply);
    }

    pub fn clear(&mut self) {
        self.ttable
            .iter_mut()
            .for_each(|entry| *entry = TTEntry::empty())
    }
}
