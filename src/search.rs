use std::sync::atomic::{AtomicBool};
use std::sync::{Arc};
use crate::eval::{eval, CHECKMATE, STALEMATE, PIECE_VALUES, MATED};
use crate::{Board, Move};
use log::info;
use std::time::Instant;
use crate::board::{KING, PAWN};
use crate::movegen::{is_in_check, is_legal_move, moved_into_check, MoveList, MoveSet};
use crate::moves::{KillerMoves, MoveType, PrevMoves};
use crate::tt::{AtomicTTable, ORDER, TT, TTable};
use crate::tt_entry::EntryType::{Alpha, Beta, PV};

pub const MAX_DEPTH: usize = 50;
// pub const MAX_DEPTH: usize = 4;
pub const MAX_TIME: u128 = 5000;
// pub const MAX_TIME: u128 = 30000;
// pub const MAX_TIME: u128 = u128::MAX;
// const MAX_QUIESCE_DEPTH: usize = 10;

pub const MIN_SCORE: i32 = CHECKMATE * 2;
const MAX_SCORE: i32 = -MIN_SCORE;


pub struct AbortFlag(Arc<AtomicBool>);

impl AbortFlag {
    pub fn new() -> AbortFlag { AbortFlag(Arc::new(AtomicBool::new(false))) }
    pub fn has_aborted(&self) -> bool { self.0.load(ORDER) }
    pub fn set_abort(&mut self, flag: bool) { self.0.store(flag, ORDER) }
}

impl Clone for AbortFlag {
    fn clone(&self) -> Self { AbortFlag(Arc::clone(&self.0)) }
}



pub fn iterative_deepening(
    board: Board,
    tt: &TTable,
    prev_moves: PrevMoves,
) -> Option<Move> {
    let mut searcher  = Searcher::<TTable>::new(board, tt, prev_moves);

    let mut best_score = MIN_SCORE;
    // let mut best_score: i32 = 0;
    let mut alpha_window = MIN_SCORE;
    let mut beta_window = -alpha_window;
    let mut best_move = None;

    for depth in 1..=MAX_DEPTH {
        if !searcher.can_start_iter() {
            searcher.abort.set_abort(true);
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
    best_move
}

pub struct Searcher<'a, T: TT> {
    board: Board,
    pub tt: &'a T,
    km: KillerMoves,
    pub nodes: usize,
    pub hits: usize,
    pub misses: usize,
    start: Instant,
    pub prev_moves: PrevMoves,
    abort: AbortFlag,
}

impl <'a> Searcher<'a, Arc<AtomicTTable>> {
    pub fn new(
        board: Board, 
        tt: &'a Arc<AtomicTTable>, 
        prev_moves: PrevMoves, 
        abort: AbortFlag
    ) -> Searcher<'a, Arc<AtomicTTable>> {
        Searcher {
            board,
            tt,
            km: KillerMoves::new(),
            nodes: 0,
            hits: 0,
            misses: 0,
            start: Instant::now(),
            prev_moves,
            abort,
        }
    }
}

impl <'a> Searcher<'a, TTable> {
    pub fn new(board: Board, tt: &'a TTable, prev_moves: PrevMoves) -> Searcher<'a, TTable> {
        Searcher {
            board,
            tt,
            km: KillerMoves::new(),
            nodes: 0,
            hits: 0,
            misses: 0,
            start: Instant::now(),
            prev_moves,
            abort: AbortFlag::new()
        }
    }
}

impl<'a, T: TT> Searcher<'a, T> {

    fn can_start_iter(&self) -> bool {
        self.start.elapsed().as_millis() < MAX_TIME
    }

    pub fn root_negamax(
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
}