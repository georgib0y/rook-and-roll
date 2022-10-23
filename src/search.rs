use crate::eval::{eval, CHECKMATE, QUEEN_VALUE};
// use crate::movegen::{gen_attacks, gen_moves, gen_quiet};
use crate::{tt, Board, Move, MoveTables,SeqTT};
use log::info;
use std::io;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use crate::movegen::{is_in_check, is_legal_move, moved_into_check, MoveList};
use crate::tt::EntryType::{Alpha, Beta, PV};

const MAX_DEPTH: usize = 7;
const MAX_TIME: u64 = 5000;
const MAX_QUIESCE_DEPTH: usize = 10;

// using lazy static atomic cause unsafe keyword scares me

pub struct Search <'a> {
    mt: &'a MoveTables,
    tt: &'a mut SeqTT,
    nodes: u64,
}

impl <'a> Search <'a> {
    pub fn new(mt: &'a MoveTables, tt: &'a mut SeqTT) -> Search <'a> {
        Search { mt, tt, nodes: 0 }
    }

    // TODO impl time constraint
    pub fn iterative_deepening(&mut self, board: &Board) -> Option<Move> {
        let mut best_move = None;
        let mut best_score = 0;

        let start = Instant::now();
        self.nodes = 0;

        let mut nps: f64 = 0.0;
        let mut elapsed: f64 = 0.0;
        for depth in 1..=MAX_DEPTH {
            (best_score, best_move) = self.root_negamax(board, depth);

            elapsed = start.elapsed().as_millis() as f64; // so that there is no divide by 0 err
            nps = self.nodes as f64 / (elapsed / 1000f64);
            if nps.is_infinite() {
                nps = 0f64;
            }

            let info = format!("info depth {} score cp {} nps {}", depth, best_score, (nps) as usize);
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
        let ml = MoveList::all(board, self.mt, check);
        for m in ml.moves {
            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, self.mt, &m))
                || !is_legal_move(&b, self.mt, &m) { continue; }

            // TODO changed alpha from i32::MIN+1 so that at least one move gets chosen each time, may be bad for elo dunno
            // TODO could lower beta bounds by best_score but im trying it without
            let mut score = -self.negamax(&b, depth - 1, 1, i32::MIN + 2, -best_score, -p_mul);

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
        let ml = MoveList::all(board, self.mt, check);
        for m in ml.moves {
            not_moved = false;

            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, self.mt, &m))
                || !is_legal_move(&b, self.mt, &m) { continue; }

            score = -self.negamax(&b, depth - 1, ply + 1, -beta, -alpha, -p_mul);

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
            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, self.mt, &m))
                || !is_legal_move(&b, self.mt, &m) { continue; }

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

