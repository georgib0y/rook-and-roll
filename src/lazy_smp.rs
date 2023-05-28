use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread;
use std::thread::{Scope, ScopedJoinHandle};
use std::time::Instant;
use log::info;
use crate::board::{Board, PAWN};
use crate::moves::{Move, PrevMoves};
use crate::search::{MAX_DEPTH, MAX_TIME, MIN_SCORE, Searcher};
use crate::tt::{AtomicTTable, ORDER};

struct LazySmp {
    board: Board,
    tt: Arc<AtomicTTable>,
    prev_moves: PrevMoves,
    num_threads: usize,
    start: Instant,
    abort_flag: Arc<AtomicBool>,
    total_nodes: Arc<AtomicUsize>,
    total_hits: Arc<AtomicUsize>,
    total_misses: Arc<AtomicUsize>,
    best_move: Option<Move>,
    best_score: i32,
    alpha_window: i32,
    beta_window: i32
}

impl LazySmp {
    fn new(
        board: Board,
        tt: Arc<AtomicTTable>,
        prev_moves: PrevMoves,
        num_threads: usize
    ) -> LazySmp {
        LazySmp {
            board, tt, prev_moves, num_threads,
            start: Instant::now(),
            abort_flag: Arc::new(AtomicBool::new(false)),
            total_nodes: Arc::new(AtomicUsize::new(0)),
            total_hits: Arc::new(AtomicUsize::new(0)),
            total_misses: Arc::new(AtomicUsize::new(0)),
            best_move: None,
            best_score: MIN_SCORE,
            alpha_window: MIN_SCORE,
            beta_window: -MIN_SCORE,
        }
    }

    fn start(mut self) -> Option<Move> {
        self.start = Instant::now();
        self.abort_flag.store(false, ORDER);


        for depth in 1..=MAX_DEPTH {
            if self.start.elapsed().as_millis() > MAX_TIME { break; }
            thread::scope(|scope| {
                let threads = self.start_helpers(scope, depth);
                (self.best_score, self.best_move) = self.search(depth);
                threads.into_iter()
                    .for_each(|thread| thread.join().unwrap());
            });

            self.print_iter_info(depth);
        }

        self.print_search_info();
        self.best_move
    }

    fn start_helpers<'scope, 'env> (
        &mut self,
        scope: &'scope Scope<'scope, '_>,
        depth: usize
    ) -> Vec<ScopedJoinHandle<'scope, ()>> {
        // spawn n threads at a particular depth
        let mut threads = Vec::with_capacity(self.num_threads);
        for _ in 0..self.num_threads-1 {
            let nodes = Arc::clone(&self.total_nodes);
            let hits = Arc::clone(&self.total_hits);
            let misses = Arc::clone(&self.total_misses);
            let arc_tt = Arc::clone(&self.tt);
            let abort = Arc::clone(&self.abort_flag);
            let prev_moves_clone = self.prev_moves.clone();
            let board = self.board.clone();
            let alpha_window = self.alpha_window;
            let beta_window = self.beta_window;
            let best_score = self.best_score;

            threads.push(scope.spawn(move || {
                let mut helper = Searcher::new(
                    board,
                    &arc_tt,
                    prev_moves_clone,
                    abort
                );

                let _ = helper.root_negamax(alpha_window, beta_window, depth);
                // research with full window if aspiration search fails
                if alpha_window != MIN_SCORE && (best_score <= alpha_window || best_score >= beta_window) {
                    let _ = helper.root_negamax(MIN_SCORE, MIN_SCORE, depth);
                }

                nodes.fetch_add(helper.nodes, ORDER);
                hits.fetch_add(helper.hits, ORDER);
                misses.fetch_add(helper.misses, ORDER);
            }));
        }

        threads
    }

    fn search(&mut self, depth: usize) -> (i32, Option<Move>){
        let mut searcher: Searcher<Arc<AtomicTTable>, Arc<AtomicBool>> = Searcher::new(
            self.board.clone(),
            &self.tt,
            self.prev_moves.clone(),
            Arc::clone(&self.abort_flag)
        );

        let (best_score, best_move) = searcher.root_negamax(
            self.alpha_window,
            self.beta_window,
            depth
        );

        self.total_nodes.fetch_add(searcher.nodes, ORDER);
        self.total_hits.fetch_add(searcher.hits, ORDER);
        self.total_misses.fetch_add(searcher.misses, ORDER);

        // re-adjust aspiration window for the next iteration
        self.alpha_window = best_score - (PAWN/2) as i32;
        self.beta_window = best_score + (PAWN/2) as i32;

        (best_score, best_move)
    }

    fn print_iter_info(&self, depth: usize) {
        let elapsed = self.start.elapsed().as_millis() as f64;
        let mut nps = self.total_nodes.load(ORDER) as f64 / (elapsed / 1000f64);
        if nps.is_infinite() { nps = 0f64; } // so that there is no divide by 0 err

        let info = format!("info depth {} score cp {} nps {} pv {}",
            depth,
            self.best_score,
            (nps) as usize,
            self.best_move.unwrap().as_uci_string()
        );

        info!(target: "output", "{}", info);
        println!("{}", info);
    }

    fn print_search_info(&self) {
        let hits = self.total_hits.load(ORDER) as f32;
        let misses = self.total_misses.load(ORDER) as f32;
        let hit_rate = (hits / (hits+misses+1.0)) * 100.0;
        let info = format!("info string nodes {} hits {} misses {} hitrate {:.2}%",
                           self.total_nodes.load(ORDER), hits, misses, hit_rate);
        info!(target: "output", "{}", info);
        println!("{}", info);
    }
}

pub fn lazy_smp(
    board: Board,
    tt: Arc<AtomicTTable>,
    prev_moves: PrevMoves,
    num_threads: usize
) -> Option<Move> {
    LazySmp::new(board, tt, prev_moves, num_threads).start()
}
