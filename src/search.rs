use crate::eval::{eval, CHECKMATE, QUEEN_VALUE};
// use crate::movegen::{gen_attacks, gen_moves, gen_quiet};
use crate::{tt, Board, Move, MoveTables,SeqTT};
use log::info;
use std::io;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use crate::movegen::{is_in_check, is_legal_move, moved_into_check, MoveList};
use crate::moves::{KillerMoves, PrevMoves};
use crate::tt::EntryType;
use crate::tt::EntryType::{Alpha, Beta, PV};

pub const MAX_DEPTH: usize = 50;
const MAX_TIME: u128 = 50000;
const MAX_QUIESCE_DEPTH: usize = 10;

pub trait Searcher {
    fn is_in_check(&self, board: &Board);
    fn get_all_scored_moves(&self, board: &Board, check: bool, depth: usize);
    fn get_in_check_moves(&self);
    fn get_attack_moves(&self);
    fn add_prev_move(&self, hash: u64);
    fn rm_prev_move(&self, hash: u64);
    fn moved_into_check(&self, board: &Board, m: Move);
    fn is_legal_move(&self, board: &Board, m: &Move);
    fn inc_node(&self);
    fn tt_get_score(&self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize);
    fn tt_insert(
        &self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize
    );

}


pub struct Search <'a> {
    mt: &'a MoveTables,
    tt: &'a mut SeqTT,
    km: &'a mut KillerMoves,
    nodes: u64,
    start: Instant,
    prev_moves: &'a mut PrevMoves
}

impl <'a> Search <'a> {
    pub fn new(
        mt: &'a MoveTables,
        tt: &'a mut SeqTT,
        km: &'a mut KillerMoves,
        prev_moves: &'a mut PrevMoves
    ) -> Search <'a> {
        Search { mt, tt, km, nodes: 0, start: Instant::now(), prev_moves }
    }

    fn is_timeout(&self) -> bool { self.start.elapsed().as_millis() > MAX_TIME }
    fn can_start_iter(&self) -> bool { self.start.elapsed().as_millis() < (MAX_TIME / 2) }

    // TODO impl time constraint
    pub fn iterative_deepening(&mut self, board: &Board) -> Option<Move> {
        let mut best_move = None;
        let mut best_score = 0;

        self.start = Instant::now();
        self.nodes = 0;

        let mut nps: f64 = 0.0;
        let mut elapsed: f64 = 0.0;
        for depth in 1..=MAX_DEPTH {
            if !self.can_start_iter() { break; }

            (best_score, best_move) = self.root_negamax(board, depth);

            elapsed = self.start.elapsed().as_millis() as f64; // so that there is no divide by 0 err
            nps = self.nodes as f64 / (elapsed / 1000f64);
            if nps.is_infinite() {
                nps = 0f64;
            }

            let info = format!("info depth {depth} score cp {best_score} nps {} pv {}", (nps) as usize, best_move.unwrap().as_uci_string());
            info!(target: "output", "{}", info);
            println!("{}", info);
        }

        println!("info string npsps {}", (nps/(elapsed/1000f64)) as usize);
        info!(target: "output", "finished last depth with {} npsps", (nps/(elapsed/1000f64)) as usize);

        best_move
    }

    pub fn root_negamax(&mut self, board: &Board, depth: usize) -> (i32, Option<Move>) {
        let mut best_move = None;
        let mut best_score = i32::MIN + 1;
        let p_mul = if board.colour_to_move == 0 { 1 } else { -1 };

        let check = is_in_check(board, self.mt);
        let ml = MoveList::all_scored(board, check, self.mt, self.tt, self.km, depth);
        for m in ml {
            let b = board.copy_make(m);

            self.prev_moves.add(b.hash);

            if (!check && moved_into_check(&b, self.mt, m))
                || !is_legal_move(&b, self.mt, m, self.prev_moves) { continue; }

            let mut score = -self.negamax(&b, depth - 1, 1, i32::MIN + 2, -best_score, -p_mul);

            self.prev_moves.remove(b.hash);

            if score > best_score {
                best_move = Some(m);
                best_score = score;
            }
        }

        (best_score, best_move)
    }

    pub fn negamax(
        &mut self,
        board: &Board,
        depth: usize,
        ply: usize,
        mut alpha: i32,
        beta: i32,
        p_mul: i32, // player multiplier - to be passed down to eval
    ) -> i32 {
        self.nodes += 1;

        if let Some(score) = self.tt.get_score(board.hash, depth, alpha, beta, ply) {
            return score;
        }

        if depth == 0 {
            let eval = self.quiesce(board, ply, ply + MAX_QUIESCE_DEPTH, alpha, beta, p_mul);
            self.tt.insert(board.hash, eval, PV, 0, None, ply);
            return eval;
        }

        let mut best_move: Option<Move> = None;
        let mut table_entry_type = Alpha;

        let check = is_in_check(board, self.mt);
        let mut not_moved = true;
        let mut score = i32::MIN;
        let ml = MoveList::all_scored(board, check, self.mt, self.tt, self.km, depth);
        for m in ml {
            not_moved = false;

            let b = board.copy_make(m);

            self.prev_moves.add(b.hash);

            if (!check && moved_into_check(&b, self.mt, m))
                || !is_legal_move(&b, self.mt, m, self.prev_moves) { continue; }

            score = -self.negamax(&b, depth - 1, ply + 1, -beta, -alpha, -p_mul);

            self.prev_moves.remove(b.hash);

            if score > alpha {
                table_entry_type = PV;
                best_move = Some(m);
                alpha = score;
            }

            if score >= beta {
                self.tt.insert(board.hash, beta, Beta, depth, None, ply);
                return beta;
            }
        }

        self.tt.insert(board.hash, alpha, table_entry_type, depth, best_move, ply);

        // hasn't moved and in check -> checkmate
        if not_moved && check {
            CHECKMATE + ply as i32
        } else {
            alpha
        }
    }

    fn quiesce(
        &mut self,
        board: &Board,
        ply: usize,
        max_ply: usize,
        mut alpha: i32,
        beta: i32,
        p_mul: i32,
    ) -> i32 {
        self.nodes += 1;

        // cut off at certain depth
        if ply == max_ply {
            return eval(&board, p_mul);
        }

        let check = is_in_check(board, self.mt);
        let mut ml;

        if check {
            ml = MoveList::checks(board, self.mt);
            if ml.moves.is_empty() {
                return CHECKMATE + ply as i32;
            }
        } else {
            let eval = eval(board, p_mul);

            if eval >= beta {
                return beta;
            }

            // delta pruning
            if eval < alpha.saturating_sub(QUEEN_VALUE) {
                return alpha
            }

            if alpha < eval {
                alpha = eval;
            }

            ml = MoveList::attacks(board, self.mt);
        }

        for m in ml.moves {
            let b = board.copy_make(m);
            if (!check && moved_into_check(&b, self.mt, m))
                || !is_legal_move(&b, self.mt, m, self.prev_moves) { continue; }

            let score = -self.quiesce(&b, ply + 1, max_ply, -beta, -alpha, -p_mul);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }
}

