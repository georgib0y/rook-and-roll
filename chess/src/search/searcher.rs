use std::{
    error::Error,
    fmt::{Display, Formatter},
    io::Write,
    time::Instant,
};

use crate::{
    board::{Board, BLACK, KING, WHITE},
    movegen::{
        move_list::{QSearchMoveList, ScoredMoveList},
        movegen::{gen_all_attacks, gen_moves, is_in_check, is_legal_move, moved_into_check},
        moves::{KillerMoves, Move, MoveType, PrevMoves, NULL_MOVE},
    },
    search::eval::{CHECKMATE, STALEMATE},
};

use super::{
    eval::{eval, MATED, PIECE_VALUES},
    tt::{
        EntryScore::{self, *},
        TT,
    },
    HistoryTable,
};

const TIME_LIMIT_MS: u128 = 30000;
pub const MAX_DEPTH: usize = 100;
pub const MIN_SCORE: i32 = CHECKMATE * 2;
const MAX_SCORE: i32 = -MIN_SCORE;
const QSEARCH_MAX_PLY: usize = 50;

#[derive(Debug)]
pub enum SearchError {
    NoMove,
    FailLow,
    FailHigh,
}

impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::NoMove => writeln!(f, "No moves found"),
            SearchError::FailLow => writeln!(f, "Failed low"),
            SearchError::FailHigh => writeln!(f, "Failed high"),
        }
    }
}

impl Error for SearchError {}

pub type SearchResult = Result<(i32, Move), SearchError>;

pub struct Searcher<'a, T: TT> {
    aborted: bool,
    start: Instant,
    time_limit_ms: u128,
    root_depth: i32,
    ply: i32,
    c_mul: i32,
    pub tt: &'a mut T,
    pub km: KillerMoves,
    pub hh: HistoryTable,
    prev_moves: PrevMoves,
    nodes: usize,
}

impl<'a, T: TT> Searcher<'a, T> {
    fn new(tt: &'a mut T, prev_moves: PrevMoves, time_limit_ms: u128) -> Searcher<T> {
        Searcher {
            aborted: false,
            start: Instant::now(),
            time_limit_ms,
            root_depth: 0,
            ply: 0,
            c_mul: 0,
            tt,
            km: KillerMoves::new(),
            hh: HistoryTable::new(),
            prev_moves,
            nodes: 0,
        }
    }

    fn draft(&self) -> i32 {
        self.root_depth - self.ply
    }

    fn init_search(&mut self, b: &Board, depth: usize) {
        self.c_mul = if b.ctm() == WHITE { 1 } else { -1 };
        self.ply = 0;
        self.root_depth = depth as i32;
        self.nodes = 0;
    }

    fn has_aborted(&mut self) -> bool {
        // check only every few thousand nodes
        if self.nodes & 0xFFF == 0 {
            self.aborted = self.time_limit_ms < self.start.elapsed().as_millis();
        }

        self.aborted
    }

    fn push_ply(&mut self) {
        self.ply += 1;
        self.c_mul = -self.c_mul;
    }

    fn pop_ply(&mut self) {
        self.ply -= 1;
        self.c_mul = -self.c_mul;
    }

    fn store_tt(&mut self, hash: u64, score: EntryScore, bm: Option<Move>) {
        // do not store anything after abort
        if self.aborted {
            return;
        }

        self.tt.insert(hash, score, bm, self.draft());
    }

    fn try_move(
        &mut self,
        board: &Board,
        m: Move,
        alpha: i32,
        beta: i32,
        depth: usize,
    ) -> Option<i32> {
        let b = board.copy_make(m);

        if !is_legal_move(&b, m, &self.prev_moves) || moved_into_check(&b, m) {
            return None;
        }

        self.push_ply();
        self.prev_moves.add(b.hash());

        let score = -self.pvs(&b, -beta, -alpha, depth - 1);

        self.pop_ply();
        self.prev_moves.remove(b.hash());

        Some(score)
    }

    fn try_null_window_move(
        &mut self,
        b: &Board,
        m: Move,
        alpha: i32,
        depth: usize,
    ) -> Option<i32> {
        self.try_move(b, m, alpha, alpha + 1, depth)
    }

    pub fn root_pvs(
        &mut self,
        b: &Board,
        mut alpha: i32,
        beta: i32,
        depth: usize,
    ) -> Option<(i32, Move)> {
        self.init_search(b, depth);

        let mut ml = ScoredMoveList::new(b, self, depth);
        gen_moves(b, &mut ml, is_in_check(b));

        let mut best_res = None;
        let mut tt_entry_score = Alpha(alpha);

        for m in ml {
            let Some(score) = self.try_move(b, m, alpha, beta, depth) else {
                continue;
            };

            if score > alpha {
                alpha = score;
                best_res = Some((alpha, m));
                tt_entry_score = PV(alpha);
            }

            if score >= beta {
                self.tt.insert(b.hash(), Beta(beta), Some(m), self.draft());
                best_res = Some((beta, m));

                return best_res;
            }
        }

        self.store_tt(b.hash(), tt_entry_score, best_res.map(|b| b.1));

        best_res
    }

    fn pvs(&mut self, b: &Board, mut alpha: i32, beta: i32, depth: usize) -> i32 {
        if self.has_aborted() {
            return MIN_SCORE;
        }

        self.nodes += 1;

        if depth == 0 {
            let q_score = self.q_search(b, alpha, beta);
            self.tt.insert(b.hash(), PV(q_score), None, self.draft());
            return q_score;
        }

        if let Some(score) = self.tt.get_score(b.hash(), self.draft(), alpha, beta) {
            return score;
        }

        let in_check = is_in_check(b);

        let mut best_move = None;
        let mut tt_entry_score = Alpha(alpha);

        let mut ml = ScoredMoveList::new(b, self, depth);
        gen_moves(b, &mut ml, in_check);

        let mut found_pv = false;
        let mut has_moved = false;

        for m in ml {
            let score = if !found_pv {
                let Some(score) = self.try_move(b, m, alpha, beta, depth) else {
                    continue;
                };

                has_moved = true;
                score
            } else {
                let Some(mut score) = self.try_null_window_move(b, m, alpha, depth) else {
                    continue;
                };

                if score > alpha && score < beta {
                    // it is safe to unwrap here as the move is legal at this point
                    score = self.try_move(b, m, alpha, beta, depth).unwrap();
                }

                score
            };

            if score >= beta {
                self.store_tt(b.hash(), Beta(beta), Some(m));

                if m.move_type() == MoveType::Quiet {
                    self.km.add(m, depth);
                    self.hh
                        .insert(b.ctm(), m.from() as usize, m.to() as usize, depth);
                }

                return beta;
            }

            if score > alpha {
                alpha = score;
                best_move = Some(m);
                tt_entry_score = PV(alpha);
                found_pv = true;
            }
        }

        if !has_moved {
            alpha = if in_check {
                CHECKMATE + self.ply
            } else {
                STALEMATE
            };

            tt_entry_score = PV(alpha);
        }

        self.store_tt(b.hash(), tt_entry_score, best_move);
        alpha
    }

    fn try_q_move(
        &mut self,
        board: &Board,
        m: Move,
        alpha: i32,
        beta: i32,
        eval: i32,
    ) -> Option<i32> {
        let b = board.copy_make(m);

        if moved_into_check(&b, m) || delta_prune(&b, alpha, eval, m) {
            return None;
        }

        self.push_ply();
        let score = -self.q_search(&b, -beta, -alpha);
        self.pop_ply();

        Some(score)
    }

    fn q_search(&mut self, b: &Board, mut alpha: i32, beta: i32) -> i32 {
        // TODO maybe only check for check when entering q_search
        if self.draft() > -2 && is_in_check(b) {
            return self.pvs(b, alpha, beta, 1);
        }

        let eval = eval(b, self.c_mul);

        // TODO maybe return alpha instead of stand-pat?
        if self.ply > QSEARCH_MAX_PLY as i32 {
            return eval;
        }

        if eval >= beta {
            return beta;
        }

        if alpha < eval {
            alpha = eval;
        }

        let mut ml = QSearchMoveList::<'_, T, 100>::new(b, self, 0);
        gen_all_attacks(b, &mut ml);

        for m in ml {
            if m.xpiece() >= KING as u32 {
                return MATED - self.ply;
            }

            let Some(score) = self.try_q_move(b, m, alpha, beta, eval) else {
                continue;
            };

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

pub fn iterative_deepening<'a>(
    board: &'a Board,
    tt: &'a mut impl TT,
    prev_moves: PrevMoves,
    out: &mut impl Write,
) -> SearchResult {
    let mut s = Searcher::new(tt, prev_moves, TIME_LIMIT_MS);

    let mut res = None;

    for depth in 1..MAX_DEPTH {
        let alpha_window = MIN_SCORE;
        let beta_window = MAX_SCORE;

        let iter_res = s.root_pvs(board, alpha_window, beta_window, depth);

        if s.has_aborted() {
            break;
        }

        res = iter_res;

        write_info(out, &s, board, res, depth);
    }

    res.ok_or(SearchError::NoMove)
}

fn write_info<T: TT>(
    out: &mut impl Write,
    s: &Searcher<T>,
    b: &Board,
    res: Option<(i32, Move)>,
    depth: usize,
) {
    let pv_str =
        s.tt.get_full_pv(b)
            .iter()
            .fold(String::new(), |pv, m| pv + &m.as_uci_string() + " ");

    let (score, _) = res.unwrap_or((MIN_SCORE, NULL_MOVE));

    let nps = s.nodes as f64 / s.start.elapsed().as_secs_f64();
    writeln!(
        out,
        "info depth {} score cp {} nps {:.0} {}",
        depth, score, nps, pv_str
    )
    .unwrap();
}

fn delta_prune(b: &Board, alpha: i32, eval: i32, m: Move) -> bool {
    eval + PIECE_VALUES[m.xpiece() as usize] + 200 < alpha
        && !m.move_type().is_promo()
        && (b.all_occ() ^ b.pawns(WHITE) ^ b.pawns(BLACK)).count_ones() > 4
}
