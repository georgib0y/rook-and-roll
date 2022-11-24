use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Instant;
use log::info;
use crate::board::Board;
use crate::movegen::{is_legal_move, MoveList};
use crate::moves::{KillerMoves, Move, PrevMoves};
use crate::search::{MAX_DEPTH, MAX_TIME, Search, Searcher};
use crate::tt::{EntryType, ORDER, ParaTT};

// use rayon::prelude::*;

///////////////////////////////////////////
// TODO heres the plan, have some sort of message channel broadcast to all the threads,
// so when the time runs out, all the threads stop running and the ones that have not stopped searching return None,
// then we can iterate though the workers thread to join the deepest thread that isnt None
/////////////////////////////////////////////

fn spawn_thread(
    board: Board,
    depth: usize,
    tx: Sender<(usize, i32, Option<Move>, usize)>,
    mut searcher: SmpSearcher
) {
    thread::spawn(move || {
        let (best_score, best_move) = searcher.root_negamax(&board, depth);
        tx.send( (depth, best_score, best_move, searcher.nodes))
    });
}

pub fn lazy_smp(board: &Board, tt: Arc<ParaTT>, prev_moves: PrevMoves, num_threads: usize) -> Option<Move> {
    let start = Instant::now();
    let abort_flag = Arc::new(AtomicBool::new(false));

    let mut best_move = None;
    let total_nodes = Arc::new(AtomicUsize::new(0));

    let mut threads = Vec::with_capacity(num_threads);

    'iter_deep: for depth in 1..=MAX_DEPTH {
        let mut best_score = i32::MIN;
        if start.elapsed().as_millis() > MAX_TIME {
        }

        // spawn n threads at a particular depth
        for _ in 0..num_threads {
            let mut searcher = SmpSearcher::new(
                Arc::clone(&tt),
                KillerMoves::new(),
                prev_moves.clone(),
                Arc::clone(&abort_flag)
            );

            let b = board.clone();
            let nodes = Arc::clone(&total_nodes);

            threads.push(thread::spawn(move || {
                let res = searcher.root_negamax(&b, depth);
                nodes.fetch_add(searcher.nodes, ORDER);
                res
            }))
        }

        // try to join threads, but if they go over the time then end early
        for thread in threads.drain(0..) {
            loop {
                if thread.is_finished() {
                    let (score, m) = thread.join().unwrap();
                    if score > best_score && m.is_some() {
                        best_score = score;
                        best_move = m;
                    }
                    break;

                } else if start.elapsed().as_millis() > MAX_TIME {
                    abort_flag.store(true, ORDER);
                    break 'iter_deep;
                }
            }
        }

        let elapsed = start.elapsed().as_millis() as f64;
        let mut nps = total_nodes.load(ORDER) as f64 / (elapsed / 1000f64);
        if nps.is_infinite() { nps = 0f64; } // so that there is no divide by 0 err

        let info = format!("info depth {depth} score cp {best_score} nps {} pv {}",
                           (nps) as usize,
                           best_move.unwrap().as_uci_string()
        );

        info!(target: "output", "{}", info);
        println!("{}", info);
    }

    best_move
}

pub fn _lazy_smp(
    board: &Board,
    tt: Arc<ParaTT>,
    km: &KillerMoves,
    prev_moves: &PrevMoves,
    num_threads: usize
) -> Option<Move> {
    let (tx, rx) = mpsc::channel();

    let start = Instant::now();
    let abort_flag = Arc::new(AtomicBool::new(false));

    let mut best_move = None;
    let mut best_depth = 0;
    let mut best_score = i32::MIN;
    let mut total_nodes = 0;

    let mut active_threads = 0;
    let mut depth = 0;

    // while we still have time:
    // spawn a thread in an increased depth if active_threads is less than num_threads
    // then see if any threads have sent any results to the receiver
    while start.elapsed().as_millis() < MAX_TIME && depth < MAX_DEPTH {
        if active_threads < num_threads {
            active_threads += 1;
            depth += 1;

            let searcher = SmpSearcher::new(
                Arc::clone(&tt),
                km.clone(),
                prev_moves.clone(),
                Arc::clone(&abort_flag)
            );
            println!("Spawning thread at depth: {depth}");
            spawn_thread(board.clone(), depth, tx.clone(), searcher);
        }



        if let Ok((iter_depth, iter_score, iter_move, nodes)) = rx.try_recv() {
            println!("\tdepth: {iter_depth}, score: {iter_score}, move: {}, nodes: {nodes}, active_threads: {active_threads}",
                iter_move.map_or("None".to_string(), |m| m.as_uci_string())
            );

            active_threads -= 1;
            total_nodes += nodes;
            if iter_move.is_none() { continue; }

            let elapsed = start.elapsed().as_millis() as f64;
            let mut nps = nodes as f64 / (elapsed / 1000f64);
            if nps.is_infinite() { nps = 0f64; } // so that there is no divide by 0 err

            let info = format!("info depth {iter_depth} score cp {iter_score} nps {} pv {}",
                               (nps) as usize,
                               iter_move.unwrap().as_uci_string()
            );

            info!(target: "output", "{}", info);
            println!("{}", info);

            if iter_depth > best_depth {
                best_depth = iter_depth;
                best_move = iter_move;
                best_score = iter_score;
            }
        }
    }

    // set the abort flag for all other threads
    abort_flag.store(true, ORDER);

    let elapsed = start.elapsed().as_millis() as f64;
    let mut nps = total_nodes as f64 / (elapsed / 1000f64);
    if nps.is_infinite() { nps = 0f64; } // so that there is no divide by 0 err

    let info = format!("info depth {best_depth} score cp {best_score} nps {} pv {}",
                       (nps) as usize,
                       best_move.unwrap().as_uci_string()
    );

    info!(target: "output", "{}", info);
    println!("{}", info);

    best_move
}

struct SmpSearcher {
    tt: Arc<ParaTT>,
    km: KillerMoves,
    prev_moves: PrevMoves,
    nodes: usize,
    abort: Arc<AtomicBool>,
}

impl SmpSearcher {
    fn new(
        tt: Arc<ParaTT>,
        km: KillerMoves,
        prev_moves: PrevMoves,
        abort: Arc<AtomicBool>
    ) -> SmpSearcher {
        SmpSearcher {
            tt, km, prev_moves, nodes: 0, abort
        }
    }
}

impl <'a> Searcher<'a> for SmpSearcher {
    fn add_prev_move(&mut self, hash: u64) {
        self.prev_moves.add(hash)
    }

    fn rm_prev_move(&mut self, hash: u64) {
        self.prev_moves.remove(hash)
    }

    fn is_legal_move(&self, board: &Board, m: Move) -> bool {
        is_legal_move(board, m, &self.prev_moves)
    }

    fn inc_node(&mut self) {
        self.nodes += 1;
    }

    fn has_aborted(&self) -> bool { self.abort.load(ORDER) }

    fn km(&self) -> &KillerMoves { &self.km }

    fn km_mut(&mut self) -> &mut KillerMoves { &mut self.km }

    fn prev_moves(&self) -> &PrevMoves { &self.prev_moves }

    fn tt_get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
        self.tt.get_score(hash, depth, alpha, beta, ply)
    }

    fn tt_insert(&mut self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
        self.tt.insert(hash, score, e_type, depth, best, ply)
    }

    fn tt_get_best_move(&self, hash: u64) -> Option<Move> {
        self.tt.get_best(hash)
    }
}
