#![allow(unused)]

use crate::board::{Board, WHITE};
use crate::movegen::moves::{KillerMoves, Move, PrevMoves, NULL_MOVE};
use crate::search::search::root_pvs;
use crate::search::searchers::{SeachResult, SearchError, Searcher, MAX_DEPTH, MIN_SCORE};
use crate::search::tt::{EntryScore, SmpTTable};
use crate::search::HistoryTable;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::scope;
use std::time::Instant;

const TIME_LIMIT_MS: u128 = 3000;

pub fn lazy_smp(
    board: &Board,
    tt: Arc<SmpTTable>,
    prev_moves: &PrevMoves,
    num_threads: usize,
    out: &mut impl Write,
) -> SeachResult {
    let mut best_result: (i32, Move) = (MIN_SCORE, NULL_MOVE);

    let mut smp = LazySmp::new(board, tt, prev_moves, num_threads);

    let start = Instant::now();
    for depth in 1..=MAX_DEPTH {
        if start.elapsed().as_millis() > TIME_LIMIT_MS {
            smp.abort();
            break;
        }

        let res = smp.run_iteration(depth)?;

        writeln!(
            out,
            "info depth {} score cp {} pv {}",
            depth,
            res.0,
            res.1.as_uci_string()
        )
        .unwrap();

        best_result = res;
    }

    Ok(best_result)
}

struct LazySmp<'a> {
    abort: Arc<AtomicBool>,
    board: &'a Board,
    searcher: SmpSearcher,
    helpers: Vec<SmpSearcher>,
}

impl<'a> LazySmp<'a> {
    fn new(
        board: &'a Board,
        tt: Arc<SmpTTable>,
        prev_moves: &PrevMoves,
        num_threads: usize,
    ) -> LazySmp<'a> {
        let abort = Arc::new(AtomicBool::new(false));
        let searcher = SmpSearcher::new(abort.clone(), tt.clone(), prev_moves.clone());
        let helpers: Vec<_> = (0..num_threads - 1)
            .map(|_| SmpSearcher::new(abort.clone(), tt.clone(), prev_moves.clone()))
            .collect();

        LazySmp {
            abort,
            board,
            searcher,
            helpers,
        }
    }

    fn run_iteration(&mut self, depth: usize) -> SeachResult {
        let alpha_window = MIN_SCORE;
        let beta_window = -alpha_window;

        let mut res = None;
        scope(|scope| {
            let _ = self.helpers.iter_mut().map(|helper| {
                scope.spawn(|| root_pvs(helper, self.board, alpha_window, beta_window, depth))
            });

            res = Some(root_pvs(
                &mut self.searcher,
                self.board,
                alpha_window,
                beta_window,
                depth,
            ));
        });

        res.ok_or(SearchError::NoMove)?
    }

    fn abort(&self) {
        self.abort.store(true, Ordering::SeqCst)
    }
}

struct SmpSearcher {
    abort: Arc<AtomicBool>,
    root_depth: i32,
    ply: i32,
    colour_mul: i32,
    tt: Arc<SmpTTable>,
    km: KillerMoves,
    hh: HistoryTable,
    prev_moves: PrevMoves,
}

impl SmpSearcher {
    pub fn new(abort: Arc<AtomicBool>, tt: Arc<SmpTTable>, prev_moves: PrevMoves) -> SmpSearcher {
        SmpSearcher {
            abort,
            root_depth: 0,
            ply: 0,
            colour_mul: 0,
            tt,
            km: KillerMoves::new(),
            hh: HistoryTable::new(),
            prev_moves,
        }
    }
}

impl Searcher for SmpSearcher {
    fn init_search(&mut self, b: &Board, depth: usize) {
        self.colour_mul = if b.ctm() == WHITE { 1 } else { -1 };
        self.ply = 0;
        self.root_depth = depth as i32;
    }

    fn has_aborted(&self) -> bool {
        self.abort.load(Ordering::SeqCst)
    }

    fn probe_tt(&self, hash: u64, alpha: i32, beta: i32) -> Option<i32> {
        // self.tt.get_score(hash, self., alpha, beta, self.ply)
        None
    }

    fn store_tt(&mut self, hash: u64, score: EntryScore, best_move: Option<Move>) {
        // self.tt
        //     .insert(hash, score, entry_type, depth, best_move, self.ply as usize);
    }

    fn get_tt_best_move(&self, hash: u64) -> Option<Move> {
        self.tt.get_best(hash)
    }

    fn get_tt_pv_move(&mut self, hash: u64) -> Option<Move> {
        self.tt.get_best_pv(hash)
    }

    fn km_get(&self, depth: usize) -> [Option<Move>; 2] {
        self.km.get_kms(depth)
    }

    fn km_store(&mut self, km: Move, depth: usize) {
        self.km.add(km, depth)
    }

    fn ply(&self) -> i32 {
        self.ply
    }

    fn draft(&self) -> i32 {
        self.root_depth - self.ply
    }

    fn colour_multiplier(&self) -> i32 {
        self.colour_mul
    }

    fn prev_moves(&self) -> &PrevMoves {
        &self.prev_moves
    }

    fn push_ply(&mut self) {
        self.ply += 1;
        self.colour_mul = -self.colour_mul;
    }

    fn push_prev_move(&mut self, hash: u64) {
        self.prev_moves.add(hash)
    }

    fn pop_ply(&mut self) {
        self.ply -= 1;
        self.colour_mul = -self.colour_mul;
    }

    fn pop_prev_move(&mut self, hash: u64) {
        self.prev_moves.remove(hash);
    }

    fn get_hh_score(&self, ctm: usize, from: usize, to: usize) -> u32 {
        self.hh.get(ctm, from, to)
    }

    fn store_hh_score(&mut self, ctm: usize, from: usize, to: usize, depth: usize) {
        self.hh.insert(ctm, from, to, depth)
    }

    fn add_node(&mut self) {
        todo!()
    }
}
