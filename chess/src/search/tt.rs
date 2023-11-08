use crate::board::Board;
use crate::movegen::moves::{Move, NULL_MOVE};
use crate::search::eval::{CHECKMATE, MATED};
use crate::search::searchers::{MAX_DEPTH, MIN_SCORE};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, RwLock};
use EntryScore::*;

pub const EMPTY_HASH: u64 = 0;
const TTABLE_SIZE: usize = 1 << 26; // 2^20
const TT_IDX_MASK: u64 = TTABLE_SIZE as u64 - 1;

// const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64  = 0xFFFF;

#[inline]
fn tt_idx(hash: u64) -> usize {
    (hash & TT_IDX_MASK) as usize
}

#[derive(Debug, Default)]
pub struct TTable {
    ttable: Box<[TTEntry]>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    collisions: AtomicUsize,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            ttable: (0..TTABLE_SIZE).map(|_| TTEntry::default()).collect(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            collisions: AtomicUsize::new(0),
        }
    }

    pub fn hits(&self) -> usize {
        self.hits.load(SeqCst)
    }

    pub fn collisions(&self) -> usize {
        self.collisions.load(SeqCst)
    }

    pub fn misses(&self) -> usize {
        self.misses.load(SeqCst)
    }

    pub fn get(&self, hash: u64) -> Option<&TTEntry> {
        let entry = &self.ttable[tt_idx(hash)];
        if entry.hash == hash {
            // dbg!(entry);
            self.hits.fetch_add(1, SeqCst);
            Some(entry)
        } else {
            if entry.hash != EMPTY_HASH {
                self.collisions.fetch_add(1, SeqCst);
            } else {
                self.misses.fetch_add(1, SeqCst);
            }
            None
        }
    }

    pub fn get_entry_at_hash_mut(&mut self, hash: u64) -> &mut TTEntry {
        &mut self.ttable[tt_idx(hash)]
    }

    pub fn get_score(&self, hash: u64, draft: i32, alpha: i32, beta: i32) -> Option<i32> {
        self.get(hash)
            .filter(|entry| entry.draft >= draft as i8)
            .and_then(|entry| entry.score.get_score(alpha, beta, draft))
    }

    pub fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.get(hash)
            .filter(|entry| entry.best != NULL_MOVE)
            .map(|entry| entry.best)
    }

    pub fn get_pv(&self, hash: u64) -> Option<Move> {
        self.get(hash)
            .filter(|entry| entry.best != NULL_MOVE)
            .filter(|entry| entry.score.is_pv())
            .map(|entry| entry.best)
        // None
    }

    pub fn insert(&mut self, hash: u64, score: EntryScore, best: Option<Move>, draft: i32) {
        assert!(draft >= 0);
        let entry = self.get_entry_at_hash_mut(hash);
        if score.is_pv() || !entry.score.is_pv() {
            *entry = TTEntry::new(hash, score.adjust_insert(draft), best, draft)
        }
    }

    pub fn clear(&mut self) {
        self.ttable
            .iter_mut()
            .for_each(|entry| *entry = TTEntry::default())
    }

    pub fn get_full_pv(&self, board: &Board) -> Vec<Move> {
        let mut pv = vec![];
        let mut b = *board;
        while let Some(best) = self.get_pv(b.hash()) {
            b = b.copy_make(best);
            pv.push(best);
        }

        pv
    }
}

pub struct SmpTTable {
    ttable: Box<[RwLock<TTEntry>]>,
}

impl SmpTTable {
    pub fn new() -> Arc<SmpTTable> {
        Arc::new(SmpTTable {
            ttable: (0..TTABLE_SIZE)
                .map(|_| RwLock::new(TTEntry::default()))
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
        // self.ttable[tt_idx(hash)]
        //     .read()
        //     .unwrap()
        //     .get_score(hash, alpha, beta, ply as usize)
        None
    }

    pub fn get_best(&self, hash: u64) -> Option<Move> {
        // self.ttable[tt_idx(hash)].read().unwrap().get_bestmove(hash)
        None
    }

    pub fn get_best_pv(&self, hash: u64) -> Option<Move> {
        // self.ttable[tt_idx(hash)].read().unwrap().get_pv(hash)
        None
    }

    pub fn insert(
        &self,
        hash: u64,
        score: i32,
        e_type: EntryScore,
        depth: usize,
        best: Option<Move>,
        ply: usize,
    ) {
        // self.ttable[(hash & TT_IDX_MASK) as usize]
        //     .write()
        //     .unwrap()
        //     .insert(hash, score, e_type, best, ply);
    }

    pub fn clear(&self) {
        self.ttable
            .iter()
            .map(|rw| rw.write().unwrap())
            .for_each(|mut entry| *entry = TTEntry::default())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EntryScore {
    PV(i32),
    Alpha(i32),
    Beta(i32),
}

impl Default for EntryScore {
    fn default() -> Self {
        Alpha(MIN_SCORE)
    }
}

impl EntryScore {
    pub fn is_pv(self) -> bool {
        matches!(self, EntryScore::PV(_))
    }

    pub fn adjust_insert(self, ply: i32) -> EntryScore {
        match self {
            PV(score) if score > MATED - MAX_DEPTH as i32 => PV(score + ply),
            Alpha(score) if score > MATED - MAX_DEPTH as i32 => Alpha(score + ply),
            Beta(score) if score > MATED - MAX_DEPTH as i32 => Beta(score + ply),

            PV(score) if score < CHECKMATE + MAX_DEPTH as i32 => PV(score - ply),
            Alpha(score) if score < CHECKMATE + MAX_DEPTH as i32 => Alpha(score - ply),
            Beta(score) if score < CHECKMATE + MAX_DEPTH as i32 => Beta(score - ply),

            _ => self,
        }
    }

    pub fn get_score(self, alpha: i32, beta: i32, ply: i32) -> Option<i32> {
        let score = match self {
            PV(score) => score,
            Alpha(score) if score <= alpha => alpha,
            Beta(score) if score >= beta => beta,
            _ => return None,
        };

        // adjust for checkmates
        match score {
            score if score > MATED - MAX_DEPTH as i32 => Some(score - ply),
            score if score < CHECKMATE + MAX_DEPTH as i32 => Some(score + ply),
            _ => Some(score),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TTEntry {
    pub hash: u64,
    pub score: EntryScore,
    pub draft: i8,
    pub best: Move,
}

impl TTEntry {
    pub fn new(hash: u64, entry: EntryScore, best: Option<Move>, draft: i32) -> TTEntry {
        TTEntry {
            hash,
            score: entry,
            draft: draft as i8,
            best: best.unwrap_or(NULL_MOVE),
        }
    }
}
