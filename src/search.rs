use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc};
use std::thread;
use crate::eval::{eval, CHECKMATE, STALEMATE, PIECE_VALUES, MATED};
use crate::{Board, Move};
use log::info;
use std::time::Instant;
use crate::board::{KING, PAWN};
use crate::movegen::{is_in_check, is_legal_move, moved_into_check, MoveList, MoveSet};
use crate::moves::{KillerMoves, MoveType, PrevMoves};
use crate::tt::{Entry, ORDER, TT, TTable, TTableMT, TTableST};
use crate::tt::EntryType::{Alpha, Beta, PV};

pub const MAX_DEPTH: usize = 50;
// pub const MAX_DEPTH: usize = 4;
pub const MAX_TIME: u128 = 5000;
// pub const MAX_TIME: u128 = 30000;
// pub const MAX_TIME: u128 = u128::MAX;
// const MAX_QUIESCE_DEPTH: usize = 10;

pub const MIN_SCORE: i32 = CHECKMATE * 2;
const MAX_SCORE: i32 = -MIN_SCORE;

pub trait AbortFlag {
    fn has_aborted(&self) -> bool;
    fn set_abort(&mut self);
}

impl AbortFlag for bool {
    fn has_aborted(&self) -> bool { *self }
    fn set_abort(&mut self) { *self = true }
}

impl AbortFlag for Arc<AtomicBool> {
    fn has_aborted(&self) -> bool { self.load(ORDER) }
    fn set_abort(&mut self) { self.store(true, ORDER) }
}

pub fn lazy_smp<T: TT>(
    board: Board,
    tt: T,
    prev_moves: PrevMoves,
    // history_table: Arc<AtomicHistoryTable>,
    num_threads: usize
) -> (T, Option<Move>) {
    let start = Instant::now();
    let abort_flag = Arc::new(AtomicBool::new(false));

    let mut best_move: Option<Move> = None;
    let mut best_score = MIN_SCORE;
    let mut alpha_window = MIN_SCORE;
    let mut beta_window = -alpha_window;

    let total_nodes = Arc::new(AtomicUsize::new(0));
    let total_hits = Arc::new(AtomicUsize::new(0));
    let total_misses = Arc::new(AtomicUsize::new(0));

    let mut threads = Vec::with_capacity(num_threads);

    for depth in 1..=MAX_DEPTH {
        if start.elapsed().as_millis() > MAX_TIME { break; }

        // spawn n threads at a particular depth
        for _ in 0..num_threads-1 {
            let mut helper: Searcher<TTableMT, Arc<AtomicBool>> = Searcher::new(
                board.clone(),
                tt.get_arc(),
                prev_moves.clone(),
                Arc::clone(&abort_flag)
            );

            let nodes = Arc::clone(&total_nodes);
            let hits = Arc::clone(&total_hits);
            let misses = Arc::clone(&total_misses);

            threads.push(thread::spawn(move || {
                let (mut _best_score, mut _best_move) = helper.root_negamax(alpha_window, beta_window, depth);
                // research with full window if aspiration search fails
                if alpha_window != MIN_SCORE && (best_score <= alpha_window || best_score >= beta_window) {
                    (_best_score, _best_move) = helper.root_negamax(MIN_SCORE, MAX_SCORE, depth);
                }

                nodes.fetch_add(helper.nodes, ORDER);
                hits.fetch_add(helper.hits, ORDER);
                misses.fetch_add(helper.misses, ORDER);
            }))
        }

        let mut searcher: Searcher<TTableMT, Arc<AtomicBool>> = Searcher::new(
            board.clone(),
            tt.get_arc(),
            prev_moves.clone(),
            Arc::clone(&abort_flag)
        );

        (best_score, best_move) = searcher.root_negamax(alpha_window, beta_window, depth);

        total_nodes.fetch_add(searcher.nodes, ORDER);
        total_hits.fetch_add(searcher.hits, ORDER);
        total_misses.fetch_add(searcher.misses, ORDER);


        // re-adjust aspiration window for the next iteration
        alpha_window = best_score - (PAWN/2) as i32;
        beta_window = best_score + (PAWN/2) as i32;

        threads.drain(0..).for_each(|thread| thread.join().unwrap());

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

    let hits = total_hits.load(ORDER) as f32;
    let misses = total_misses.load(ORDER) as f32;
    let hit_rate = (hits / (hits+misses+1.0)) * 100.0;
    let info = format!("info string nodes {} hits {} misses {} hitrate {:.2}%",
                       total_nodes.load(ORDER), hits, misses, hit_rate);
    info!(target: "output", "{}", info);
    println!("{}", info);

    (tt, best_move)
}

pub fn iterative_deepening<T: TT>(
    board: Board,
    tt: T,
    prev_moves: PrevMoves,
) -> (T, Option<Move>) {
    let mut searcher: Searcher<T, bool> = Searcher::new(board, tt, prev_moves, false);
    searcher.start = Instant::now();
    searcher.nodes = 0;

    let mut best_score = MIN_SCORE;
    let mut alpha_window = MIN_SCORE;
    let mut beta_window = -alpha_window;
    let mut best_move = None;

    for depth in 1..=MAX_DEPTH {
        if !searcher.can_start_iter() {
            searcher.abort.set_abort();
            break;
        }

        (best_score, best_move) = searcher.root_negamax(alpha_window, beta_window, depth);
        // research with full window if aspiration search fails
        if alpha_window != MIN_SCORE && (best_score <= alpha_window || best_score >= beta_window) {
            (best_score, best_move) = searcher.root_negamax(MIN_SCORE, MAX_SCORE, depth);
        }

        // re-adjust aspiration window for the next iteration
        alpha_window = best_score - (PAWN/2) as i32;
        beta_window = best_score + (PAWN/2) as i32;

        let nps = searcher.nodes / (searcher.start.elapsed().as_secs()+1) as usize;

        // let info = format!("info depth {depth} score cp {best_score} nps {} tbhits {:.0} pv {}", nps, searcher.tt_hit_rate(), best_move.unwrap().as_uci_string() );
        let info = format!("info depth {depth} score cp {best_score} nps {} pv {}", nps, best_move.unwrap().as_uci_string() );

        info!(target: "output", "{}", info);
        println!("{}", info);
    }

    let hit_rate = (searcher.hits as f32 / (searcher.hits+searcher.misses+1) as f32) * 100.0;
    println!("info string nodes {} hitrate {:.2}%", searcher.nodes, hit_rate);
    // return the seacher to take back ownership of the tt
    (searcher.tt, best_move)
}

pub struct Searcher<T: TT, U: AbortFlag> {
    board: Board,
    pub tt: T,
    km: KillerMoves,
    nodes: usize,
    hits: usize,
    misses: usize,
    start: Instant,
    pub prev_moves: PrevMoves,
    abort: U
}

impl<T: TT, U: AbortFlag> Searcher<T, U> {
    pub fn new(board: Board, tt: T, prev_moves: PrevMoves, abort: U) -> Searcher<T, U> {
        Searcher {
            board,
            tt,
            km: KillerMoves::new(),
            nodes: 0,
            hits: 0,
            misses: 0,
            start: Instant::now(),
            prev_moves,
            abort
        }
    }

    fn can_start_iter(&self) -> bool {
        self.start.elapsed().as_millis() < MAX_TIME
    }

    fn root_negamax(
        &mut self,
        alpha_window: i32,
        beta_window: i32,
        depth: usize
    ) -> (i32, Option<Move>) {
        let p_mul = if self.board.ctm == 0 { 1 } else { -1 };

        let move_set = MoveSet::get_move_set(MoveSet::All, &self.board);
        let root_moves: Vec<Move> = MoveList::get_moves(
            &self.board, move_set, &self.km, self.tt.get_best(self.board.hash), depth,
        ).collect();

        let mut best_move = None;
        let mut best_score = MIN_SCORE;

        for m in root_moves {
            // limit the amount of times has aborted is checked
            if self.abort.has_aborted() { break; }

            let board = self.board.copy_make(m);
            if (move_set != MoveSet::Check && moved_into_check(&board, m))
                || !is_legal_move(&board, m, &self.prev_moves) { continue; }

            self.prev_moves.add(board.hash);
            let score = -self.negamax(&board, depth-1, 0, alpha_window, beta_window, -p_mul);
            self.prev_moves.remove(board.hash);

            if score > best_score {
                best_score = score;
                best_move = Some(m);
            }
        }

        (best_score, best_move)
    }

    fn negamax(
        &mut self,
        board: &Board,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        p_mul: i32, // player multiplier - to be passed down to eval
    ) -> i32 {
        // if self.has_aborted() { return MIN_SCORE; }
        self.nodes += 1;

        if let Some(score) = self.tt.get_score(board.hash, depth, alpha, beta, ply) {
            self.hits += 1;
            return score;
        }

        self.misses += 1;

        if depth == 0 {
            let eval = self.quiesce(&board, ply, alpha, beta, p_mul);
            self.tt.insert(board.hash, eval, PV, 0, None, ply);
            return eval;
        }

        let mut best_move: Option<Move> = None;
        let mut table_entry_type = Alpha;

        let move_set = MoveSet::get_move_set(MoveSet::All, &board);
        let mut not_moved = true;
        let ml = MoveList::get_moves(
            board,
            move_set,
            &self.km,
            self.tt.get_best(board.hash),
            depth,
            // self.history_table()
        );

        for m in ml {
            let b = board.copy_make(m);

            if (move_set != MoveSet::Check && moved_into_check(&b, m))
                || !is_legal_move(&b, m, &self.prev_moves) { continue }

            not_moved = false;
            self.prev_moves.add(b.hash);

            let score = -self.negamax(&b, depth - 1, ply + 1, -beta, -alpha, -p_mul);

            self.prev_moves.remove(b.hash);

            if score >= beta {
                self.tt.insert(board.hash, beta, Beta, depth, None, ply);
                if m.move_type() == MoveType::Quiet {
                    self.km.add(m, depth);
                    // self.history_add(board.ctm, m.from() as usize, m.to() as usize, depth);
                }
                return beta;
            }

            if score > alpha {
                table_entry_type = PV;
                best_move = Some(m);
                alpha = score;
            }
        }

        // hasn't moved and in check -> checkmate
        alpha = if not_moved && move_set == MoveSet::Check {
            CHECKMATE + ply as i32
        } else if not_moved {
            STALEMATE
        } else {
            alpha
        };

        self.tt.insert(board.hash, alpha, table_entry_type, depth, best_move, ply);

        alpha
    }

    fn quiesce(
        &mut self,
        board: &Board,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        p_mul: i32,
    ) -> i32 {
        self.nodes += 1;

        // if ply > 5 { return eval(board, p_mul); }

        if is_in_check(board) {
            return self.negamax(board, 1, ply, alpha, beta, p_mul);
        }

        let eval = eval(board, p_mul);
        if eval >= beta { return beta; }
        if alpha < eval { alpha = eval; }

        let ml = MoveList::get_moves(
            board,
            MoveSet::Attacks,
            &self.km,
            self.tt.get_best(board.hash),
            MAX_DEPTH+1,
            // self.history_table()
        );
        // let ml = MoveList::get_moves_unscored(board, MoveSet::Attacks);

        for m in ml {
            if m.xpiece() >= KING as u32 { return MATED - ply as i32; }

            let b = board.copy_make(m);

            if moved_into_check(&b, m) { continue; }

            // delta pruning
            if eval + PIECE_VALUES[m.xpiece() as usize] + 200 < alpha
                && !m.move_type().is_promo()
                && (b.util[2] ^ b.pieces[0] ^ b.pieces[1]).count_ones() > 4  { continue; }

            let score = -self.quiesce(&b, ply + 1, -beta, -alpha, -p_mul);

            if score >= beta { return beta; }
            if score > alpha { alpha = score; }
        }

        alpha
    }

    // type History: HTable;
    // fn add_prev_move(&mut self, hash: u64);
    // fn rm_prev_move(&mut self, hash: u64);
    // fn is_legal_move(&self, board: &Board, m: Move) -> bool;
    // fn inc_node(&mut self);
    // fn has_aborted(&self) -> bool;
    // fn move_set_order(&self) -> MoveSet;
    //
    // fn km(&self) -> &KillerMoves;
    // fn km_mut(&mut self) -> &mut KillerMoves;
    // fn prev_moves(&self) -> &PrevMoves;
    //
    // fn history_add(&mut self, colour_to_move: usize, from: usize, to: usize, depth: usize);
    // fn history_get(&self, colour_to_move: usize, from: usize, to: usize) -> u32;
    //
    // fn history_table(&self) -> &Self::History;
    //
    // fn tt_hit(&mut self);
    // fn tt_miss(&mut self);
    // fn tt_hit_rate(&self) -> f64;
    //
    // fn tt_get_score(
    //     &self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize
    // ) -> Option<i32>;
    //
    // fn tt_insert(
    //     &mut self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize
    // );
    //
    // fn tt_get_best_move(&self, hash: u64) -> Option<Move>;
}


//
// pub struct _Searcher<'a> {
//     tt: &'a mut SeqTT,
//     km: &'a mut KillerMoves,
//     nodes: u64,
//     start: Instant,
//     prev_moves: &'a mut PrevMoves,
//     history_table: &'a mut HistoryTable,
//     abort: bool
// }
//
// impl <'a> _Searcher<'a> {
//     pub fn new(
//         tt: &'a mut SeqTT,
//         km: &'a mut KillerMoves,
//         prev_moves: &'a mut PrevMoves,
//         history_table: &'a mut HistoryTable
//     ) -> _Searcher<'a> {
//         Searcher { tt, km, nodes: 0, start: Instant::now(), prev_moves, history_table, abort: false }
//     }
//
//     // fn is_timeout(&self) -> bool { self.start.elapsed().as_millis() > MAX_TIME }
//     fn can_start_iter(&self) -> bool { self.start.elapsed().as_millis() < (MAX_TIME / 2) }
// }
//
// impl <'a> Searches<'a> for Searcher<'a> {
//     type History = HistoryTable;
//
//     fn add_prev_move(&mut self, hash: u64) {
//         self.prev_moves.add(hash)
//     }
//
//     fn rm_prev_move(&mut self, hash: u64) {
//         self.prev_moves.remove(hash)
//     }
//
//     fn is_legal_move(&self, board: &Board, m: Move) -> bool {
//         is_legal_move(board, m, self.prev_moves)
//     }
//
//     fn inc_node(&mut self) {
//         self.nodes += 1
//     }
//
//     fn has_aborted(&self) -> bool { self.abort }
//
//     fn move_set_order(&self) -> MoveSet {
//         MoveSet::All
//     }
//
//     fn km(&self) -> &KillerMoves { self.km }
//
//     fn km_mut(&mut self) -> &mut KillerMoves { self.km }
//
//     fn prev_moves(&self) -> &PrevMoves { self.prev_moves }
//
//     fn history_add(&mut self, colour_to_move: usize, from: usize, to: usize, depth: usize) {
//         self.history_table.insert(colour_to_move, from, to, depth)
//     }
//
//     fn history_get(&self, colour_to_move: usize, from: usize, to: usize) -> u32 {
//         self.history_table.get(colour_to_move, from, to)
//     }
//
//     fn history_table(&self) -> &Self::History {
//         self.history_table
//     }
//
//     fn tt_hit(&mut self) { self.tt.hits += 1; }
//     fn tt_miss(&mut self) { self.tt.misses += 1; }
//
//     fn tt_hit_rate(&self) -> f64 {
//         (self.tt.hits as f64 / (self.tt.hits + self.tt.misses) as f64) * 100.0
//     }
//
//     fn tt_get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize) -> Option<i32> {
//         self.tt.get_score(hash, depth, alpha, beta, ply)
//     }
//
//     fn tt_insert(&mut self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize) {
//         self.tt.insert(hash, score, e_type, depth, best, ply)
//     }
//
//     fn tt_get_best_move(&self, hash: u64) -> Option<Move> {
//         self.tt.get_best(hash)
//     }
// }