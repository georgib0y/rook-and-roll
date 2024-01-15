use crate::eval::{CHECKMATE, MATED};
use crate::moves::{Move, NULL_MOVE};
use crate::searcher::{MAX_DEPTH, MIN_SCORE};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::{Arc, RwLock};
use EntryScore::*;

pub const EMPTY_HASH: u64 = 0;

pub const TTABLE_SIZE: usize = 1 << 24; // 2^20
const TT_IDX_MASK: u64 = TTABLE_SIZE as u64 - 1;

// pub const TTABLE_SIZE: usize = 65536; // 2^16
// const TT_IDX_MASK: u64 = 0xFFFF;

#[inline]
fn tt_idx(hash: u64) -> usize {
    // ((hash >> 32) & TT_IDX_MASK) as usize
    (hash & TT_IDX_MASK) as usize
}

fn should_replace(_entry: TTEntry, _score: EntryScore) -> bool {
    true
}

pub trait TT {
    fn get_entry(&self, hash: u64) -> TTEntry;
    fn set_entry(&mut self, hash: u64, entry: TTEntry);
    fn clear(&mut self);

    fn get(&self, hash: u64) -> Option<TTEntry> {
        Some(self.get_entry(hash)).filter(|e| e.hash != EMPTY_HASH)
    }

    fn get_score(&self, hash: u64, draft: i32, ply: i32, alpha: i32, beta: i32) -> Option<i32> {
        self.get(hash)
            .filter(|entry| entry.draft >= draft as i8)
            .and_then(|entry| entry.score.get_score(alpha, beta, ply))
    }

    fn get_bestmove(&self, hash: u64) -> Option<Move> {
        self.get(hash)
            .filter(|entry| entry.best != NULL_MOVE)
            .map(|entry| entry.best)
    }

    fn insert(&mut self, hash: u64, score: EntryScore, best: Option<Move>, draft: i32) {
        match self.get(hash) {
            Some(e) if !should_replace(e, score) => return,
            _ => {}
        }

        self.set_entry(hash, TTEntry::new(hash, score, best, draft))
    }

    fn print_stats(&self) {
        println!("No TT stats to show");
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct NoTTable {
    entry: TTEntry,
}

impl TT for NoTTable {
    fn get_entry(&self, _hash: u64) -> TTEntry {
        self.entry
    }

    fn set_entry(&mut self, _hash: u64, _entry: TTEntry) {}

    fn clear(&mut self) {}
}

#[derive(Debug, Default)]
pub struct TTable {
    ttable: Box<[TTEntry]>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    collisions: AtomicUsize,
    inserts: AtomicUsize,
    new_inserts: AtomicUsize,
}

impl TTable {
    pub fn new() -> TTable {
        TTable {
            ttable: vec![TTEntry::default(); TTABLE_SIZE].into_boxed_slice(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            collisions: AtomicUsize::new(0),
            inserts: AtomicUsize::new(0),
            new_inserts: AtomicUsize::new(0),
        }
    }
}

impl TT for &mut TTable {
    fn get_entry(&self, hash: u64) -> TTEntry {
        self.ttable[tt_idx(hash)]
    }

    fn set_entry(&mut self, hash: u64, entry: TTEntry) {
        self.ttable[tt_idx(hash)] = entry;
    }

    fn clear(&mut self) {
        self.ttable
            .iter_mut()
            .for_each(|entry| *entry = TTEntry::default())
    }

    fn print_stats(&self) {
        let hits = self.hits.load(SeqCst);
        let misses = self.misses.load(SeqCst);
        let cols = self.collisions.load(SeqCst);
        let inserts = self.inserts.load(SeqCst);
        let new_inserts = self.new_inserts.load(SeqCst);
        let count = self
            .ttable
            .iter()
            .filter(|entry| entry.hash != EMPTY_HASH)
            .count();

        let total = hits + misses + cols;
        let percent = (hits as f64 / total as f64) * 100.0;
        let capacity = (count as f64 / TTABLE_SIZE as f64) * 100.0;

        let occ_inserts = (count as f64 / inserts as f64) * 100.0;
        println!(
            "{} hits, \
                {} collisions, \
                {} misses, \
                {} total gets, \
                {} inserts, \
                {} new inserts, \
                hit/miss+coll {:.2}%, \
                tt occupied {}, \
                tt capacity {:.2}%, \
                occ/inserts {:.2}%",
            hits, cols, misses, total, inserts, new_inserts, percent, count, capacity, occ_inserts
        );
    }
}

pub struct SmpTTable {
    ttable: Box<[RwLock<TTEntry>]>,
}

#[allow(unused)]
impl SmpTTable {
    pub fn new() -> Arc<SmpTTable> {
        Arc::new(SmpTTable {
            ttable: (0..TTABLE_SIZE)
                .map(|_| RwLock::new(TTEntry::default()))
                .collect(),
        })
    }
}

impl TT for Arc<SmpTTable> {
    fn get_entry(&self, hash: u64) -> TTEntry {
        *self.ttable[tt_idx(hash)].read().unwrap()
    }

    fn set_entry(&mut self, hash: u64, entry: TTEntry) {
        *self.ttable[tt_idx(hash)].write().unwrap() = entry;
    }

    fn clear(&mut self) {
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
    pub fn new_pv(score: i32, ply: i32) -> EntryScore {
        PV(adjust_insert(score, ply))
    }

    pub fn new_alpha(score: i32, ply: i32) -> EntryScore {
        Alpha(adjust_insert(score, ply))
    }

    pub fn new_beta(score: i32, ply: i32) -> EntryScore {
        Beta(adjust_insert(score, ply))
    }

    pub fn is_pv(self) -> bool {
        matches!(self, EntryScore::PV(_))
    }

    pub fn get_score(self, alpha: i32, beta: i32, ply: i32) -> Option<i32> {
        match self {
            PV(score) => Some(adjust_retrieve(score, ply)),
            Alpha(score) if adjust_retrieve(score, ply) <= alpha => Some(alpha),
            Beta(score) if adjust_retrieve(score, ply) >= beta => Some(beta),
            _ => None,
        }
    }
}

fn adjust_insert(score: i32, ply: i32) -> i32 {
    match score {
        score if score >= MATED - MAX_DEPTH as i32 => score + ply,
        score if score <= CHECKMATE + MAX_DEPTH as i32 => score - ply,
        _ => score,
    }
}

fn adjust_retrieve(score: i32, ply: i32) -> i32 {
    match score {
        score if score >= MATED - MAX_DEPTH as i32 => score - ply,
        score if score <= CHECKMATE + MAX_DEPTH as i32 => score + ply,
        _ => score,
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

pub struct PerftTT {
    ttable: Box<[PerftTTEntry]>,
}

impl Default for PerftTT {
    fn default() -> Self {
        PerftTT {
            ttable: vec![PerftTTEntry::default(); TTABLE_SIZE].into_boxed_slice(),
        }
    }
}

impl PerftTT {
    pub fn new() -> PerftTT {
        PerftTT::default()
    }

    pub fn get_count(&self, hash: u64, depth: u64) -> Option<u64> {
        let entry = self.ttable[tt_idx(hash)];

        if entry.hash == hash && entry.depth == depth {
            Some(entry.count)
        } else {
            None
        }
    }

    #[inline]
    pub fn store(&mut self, hash: u64, count: u64, depth: u64) {
        let entry = &mut self.ttable[(hash & TT_IDX_MASK) as usize];
        entry.update(hash, count, depth);
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct PerftTTEntry {
    hash: u64,
    count: u64,
    depth: u64,
}

impl PerftTTEntry {
    fn update(&mut self, hash: u64, count: u64, depth: u64) {
        self.count = count;
        self.hash = hash;
        self.depth = depth;
    }
}

#[test]
fn tt_insert_and_retrieve_is_correct() {
    crate::init();
    use EntryScore::*;

    let alpha = -50;
    let beta = 50;

    // (insert entry score, insert draft, retrieve draft, expected score)
    let inserts = [
        (PV(0), 0, 0, Some(0)),
        (PV(0), 0, 1, None),
        (Alpha(alpha - 1), 0, 0, Some(alpha)),
        (Alpha(alpha + 1), 0, 0, None),
        (Beta(beta + 1), 0, 0, Some(beta)),
        (Beta(beta - 1), 0, 0, None),
        (PV(CHECKMATE + 5), 5, 0, Some(CHECKMATE + 5)),
    ];

    let mut tt = &mut TTable::new();

    for (i, (in_score, in_draft, ret_draft, exp_score)) in inserts.into_iter().enumerate() {
        let hash = i as u64 + 1;
        tt.insert(hash, in_score, None, in_draft);
        assert_eq!(
            tt.get_score(hash, ret_draft, ret_draft, alpha, beta),
            exp_score
        );
    }
}
