use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::thread;
use std::time::Instant;
use log::info;
use crate::board::Board;
use crate::movegen::{is_legal_move, MoveSet};
use crate::moves::{AtomicHistoryTable, HTable, KillerMoves, Move, PrevMoves};
use crate::search::{MAX_DEPTH, MAX_TIME, MIN_SCORE, Searcher};
use crate::tt::{EntryType, ORDER, ParaTT};

// use rayon::prelude::*;

///////////////////////////////////////////
// TODO heres the plan, have some sort of message channel broadcast to all the threads,
// so when the time runs out, all the threads stop running and the ones that have not stopped searching return None,
// then we can iterate though the workers thread to join the deepest thread that isnt None
/////////////////////////////////////////////

pub fn lazy_smp(
    board: &Board,
    tt: Arc<ParaTT>,
    prev_moves: PrevMoves,
    history_table: Arc<AtomicHistoryTable>,
    num_threads: usize
) -> Option<Move> {
    let start = Instant::now();
    let abort_flag = Arc::new(AtomicBool::new(false));

    let mut best_move: Option<Move> = None;
    let mut best_score = MIN_SCORE;

    let total_nodes = Arc::new(AtomicUsize::new(0));

    let mut threads = Vec::with_capacity(num_threads);

    'iter_deep: for depth in 1..=MAX_DEPTH {
        if start.elapsed().as_millis() > MAX_TIME { break; }

        // spawn n threads at a particular depth
        for t in 0..num_threads-1 {
            let mut helper = SmpSearcher::new(
                &tt,
                prev_moves.clone(),
                &history_table,
                &abort_flag,
                t != 0
            );

            let b = board.clone();
            let nodes = Arc::clone(&total_nodes);

            threads.push(thread::spawn(move || {
                // let res = helper.root_negamax(&b, best_score, depth);
                // nodes.fetch_add(helper.nodes, ORDER);
                // res
            }))
        }

        // try to join threads, but if they go over the time then end early
        //todo redo this may have introduced bug cant remember what this does due to lack of bugs
        for thread in threads.drain(0..) {
            loop {
                if thread.is_finished() {
                    // if let Some( (score, m) ) = thread.join().unwrap(){
                    //     if score > best_score && m.is_some() {
                    //         best_score = score;
                    //         best_move = m;
                    //     }
                    // }
                    // break;
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


struct SmpSearcher {
    tt: Arc<ParaTT>,
    km: KillerMoves,
    prev_moves: PrevMoves,
    history_table: Arc<AtomicHistoryTable>,
    nodes: usize,
    abort: Arc<AtomicBool>,
    move_set: MoveSet
}

impl SmpSearcher {
    fn new(
        tt: &Arc<ParaTT>,
        prev_moves: PrevMoves,
        history_table: &Arc<AtomicHistoryTable>,
        abort: &Arc<AtomicBool>,
        helper: bool
    ) -> SmpSearcher {
        SmpSearcher {
            tt: Arc::clone(tt),
            km: KillerMoves::new(),
            prev_moves,
            history_table: Arc::clone(history_table),
            nodes: 0,
            abort: Arc::clone(abort),
            move_set: if helper { MoveSet::Random } else { MoveSet::All }
        }
    }
}

impl <'a> Searcher<'a> for SmpSearcher {
    type History = AtomicHistoryTable;

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

    fn move_set_order(&self) -> MoveSet {
        self.move_set
    }

    fn km(&self) -> &KillerMoves { &self.km }

    fn km_mut(&mut self) -> &mut KillerMoves { &mut self.km }

    fn prev_moves(&self) -> &PrevMoves { &self.prev_moves }

    fn history_add(&mut self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
        self.history_table.insert(colour_to_move, from, to, depth)
    }

    fn history_get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
        self.history_table.get(colour_to_move, from, to)
    }

    fn history_table(&self) -> &Self::History {
        &*self.history_table
    }

    fn tt_hit(&mut self) {self.tt.hits.fetch_add(1, ORDER); }
    fn tt_miss(&mut self) {self.tt.misses.fetch_add(1, ORDER); }

    fn tt_hit_rate(&self) -> f64 {
        let hits = self.tt.hits.load(ORDER) as f64;
        let misses = self.tt.misses.load(ORDER) as f64;

        (hits / (hits + misses)) * 100.0
    }

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
