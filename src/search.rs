use crate::eval::{eval, CHECKMATE, QUEEN_VALUE, STALEMATE};
use crate::{Board, Move, SeqTT};
use log::info;
use std::time::Instant;
use crate::movegen::{is_in_check, is_legal_move, moved_into_check, MoveList, MoveSet};
use crate::moves::{KillerMoves, PrevMoves};
use crate::tt::EntryType;
use crate::tt::EntryType::{Alpha, Beta, PV};

pub const MAX_DEPTH: usize = 50;
// pub const MAX_DEPTH: usize = 3;
pub const MAX_TIME: u128 = 5000;
// pub const MAX_TIME: u128 = 30000;
// pub const MAX_TIME: u128 = u128::MAX;
const MAX_QUIESCE_DEPTH: usize = 10;

const MIN_SCORE: i32 = CHECKMATE * 2;
// const MAX_SCORE: i32 = -MIN_SCORE;

pub trait Searcher<'a> {
    fn add_prev_move(&mut self, hash: u64);
    fn rm_prev_move(&mut self, hash: u64);
    fn is_legal_move(&self, board: &Board, m: Move) -> bool;
    fn inc_node(&mut self);
    fn has_aborted(&self) -> bool;

    fn km(&self) -> &KillerMoves;
    fn km_mut(&mut self) -> &mut KillerMoves;
    fn prev_moves(&self) -> &PrevMoves;

    fn tt_get_score(
        &self, hash: u64, depth: usize, alpha: i32, beta: i32, ply: usize
    ) -> Option<i32>;

    fn tt_insert(
        &mut self, hash: u64, score: i32, e_type: EntryType, depth: usize, best: Option<Move>, ply: usize
    );

    fn tt_get_best_move(&self, hash: u64) -> Option<Move>;

    fn root_negamax(&mut self, board: &'a Board, depth: usize) -> (i32, Option<Move>) {
        let mut best_move = None;
        let mut alpha = MIN_SCORE;

        let p_mul = if board.colour_to_move == 0 { 1 } else { -1 };

        let move_set = MoveSet::get_move_set(board);
        let ml = MoveList::get_moves(
            board,
            move_set,
            Some((self.km(), self.tt_get_best_move(board.hash), depth))
        );


        for m in ml.moves {
            if self.has_aborted() { break; }

            let b = board.copy_make(m);

            self.add_prev_move(b.hash);

            if (move_set != MoveSet::Check && moved_into_check(&b,m))
                || !self.is_legal_move(&b, m) { continue; }

            let score = -self.negamax(&b, depth - 1, 1, MIN_SCORE, -alpha, -p_mul);

            self.rm_prev_move(b.hash);

            if score > alpha {
                best_move = Some(m);
                alpha = score;
            }
        }

        (alpha, best_move)
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
        self.inc_node();

        if let Some(score) = self.tt_get_score(board.hash, depth, alpha, beta, ply) {
            return score;
        }

        if depth == 0 {
            let eval = self.quiesce(&board, ply, ply + MAX_QUIESCE_DEPTH, alpha, beta, p_mul);
            self.tt_insert(board.hash, eval, PV, 0, None, ply);
            return eval;
        }

        let mut best_move: Option<Move> = None;
        let mut table_entry_type = Alpha;

        let move_set = MoveSet::get_move_set(&board);
        let mut not_moved = true;
        let ml = MoveList::get_moves(
            board,
            move_set,
            Some((self.km(), self.tt_get_best_move(board.hash), depth))
        );


        for m in ml {
            let b = board.copy_make(m);

            if (move_set != MoveSet::Check && moved_into_check(&b, m))
                || !is_legal_move(&b, m, self.prev_moves()) { continue }

            not_moved = false;
            self.add_prev_move(b.hash);

            let score = -self.negamax(&b, depth - 1, ply + 1, -beta, -alpha, -p_mul);

            self.rm_prev_move(b.hash);

            if score >= beta {
                self.tt_insert(board.hash, beta, Beta, depth, None, ply);
                self.km_mut().add(m, depth);
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

        self.tt_insert(board.hash, alpha, table_entry_type, depth, best_move, ply);

        alpha
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
        self.inc_node();

        // cut off at certain depth
        // if ply == max_ply { return eval(&board, p_mul); }

        if is_in_check(board) {
            return self.negamax(board, 1, ply, alpha, beta, p_mul);
        }

        let eval = eval(board, p_mul);
        if eval >= beta { return beta; }

        // delta pruning
        if eval < alpha - QUEEN_VALUE { return alpha; }
        if alpha < eval { alpha = eval; }

        let ml = MoveList::get_moves(
            board,
            MoveSet::Attacks,
            Some((self.km(), None, MAX_DEPTH+1))
        );

        for m in ml.moves {
            let b = board.copy_make(m);

            if moved_into_check(&b, m) ||
                !is_legal_move(&b, m, self.prev_moves()) { continue }

            let score = -self.quiesce(&b, ply + 1, max_ply, -beta, -alpha, -p_mul);
            if score >= beta { return beta; }
            if score > alpha { alpha = score; }
        }

        alpha
    }

}
pub fn iterative_deepening(
    board: &Board,
    tt: &mut SeqTT,
    km: &mut KillerMoves,
    mut prev_moves: PrevMoves
) -> Option<Move> {
    let mut searcher = Search::new(tt, km, &mut prev_moves);
    let mut best_move = None;
    let mut best_score: i32;

    searcher.start = Instant::now();
    searcher.nodes = 0;

    let mut nps: f64 = 0.0;
    let mut elapsed: f64 = 0.0;
    for depth in 1..=MAX_DEPTH {
        if !searcher.can_start_iter() { break; }

        let res = searcher.root_negamax(board, depth);
        best_score = res.0;
        best_move = res.1;


        elapsed = searcher.start.elapsed().as_millis() as f64; // so that there is no divide by 0 err
        nps = searcher.nodes as f64 / (elapsed / 1000f64);
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

pub struct Search <'a> {
    tt: &'a mut SeqTT,
    km: &'a mut KillerMoves,
    nodes: u64,
    start: Instant,
    prev_moves: &'a mut PrevMoves,
    abort: bool
}

impl <'a> Search <'a> {
    pub fn new(
        tt: &'a mut SeqTT,
        km: &'a mut KillerMoves,
        prev_moves: &'a mut PrevMoves
    ) -> Search <'a> {
        Search { tt, km, nodes: 0, start: Instant::now(), prev_moves, abort: false }
    }

    // fn is_timeout(&self) -> bool { self.start.elapsed().as_millis() > MAX_TIME }
    fn can_start_iter(&self) -> bool { self.start.elapsed().as_millis() < (MAX_TIME / 2) }
}

impl <'a> Searcher<'a> for Search<'a> {
    fn add_prev_move(&mut self, hash: u64) {
        self.prev_moves.add(hash)
    }

    fn rm_prev_move(&mut self, hash: u64) {
        self.prev_moves.remove(hash)
    }

    fn is_legal_move(&self, board: &Board, m: Move) -> bool {
        is_legal_move(board, m, self.prev_moves)
    }

    fn inc_node(&mut self) {
        self.nodes += 1
    }

    fn has_aborted(&self) -> bool { self.abort }

    fn km(&self) -> &KillerMoves { self.km }

    fn km_mut(&mut self) -> &mut KillerMoves { self.km }

    fn prev_moves(&self) -> &PrevMoves { self.prev_moves }

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