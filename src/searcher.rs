use std::{sync::Arc, time::Instant};

use crate::{
    board::{Board, BLACK, KING, WHITE},
    error::SearchError,
    eval::{eval, MATED, PIECE_VALUES},
    eval::{CHECKMATE, STALEMATE},
    hh::HistoryTable,
    move_list::{QSearchMoveList, ScoredMoveList},
    movegen::{gen_all_attacks, gen_moves, is_in_check, is_legal_move, moved_into_check},
    moves::{KillerMoves, Move, MoveType, PrevMoves, NULL_MOVE},
    tt::{
        EntryScore::{self},
        SmpTTable, TT,
    },
};

const TIME_LIMIT_MS: u128 = 5000;
pub const MAX_DEPTH: usize = 100;
pub const MIN_SCORE: i32 = CHECKMATE * 2;
const MAX_SCORE: i32 = -MIN_SCORE;
const QSEARCH_MAX_PLY: usize = 50;

pub type SearchResult = Result<(i32, Move), SearchError>;

pub struct PVTable {
    table: Box<[Move]>,
}

impl Default for PVTable {
    fn default() -> Self {
        let size = MAX_DEPTH * (MAX_DEPTH + 1) / 2;
        PVTable {
            table: vec![Move::default(); size].into_boxed_slice(),
        }
    }
}

impl PVTable {
    fn idx_from_ply(ply: usize) -> usize {
        ply * (2 * MAX_DEPTH + 1 - ply) / 2
    }

    pub fn get(&self, ply: usize) -> Move {
        self.table[PVTable::idx_from_ply(ply)]
    }

    fn set(&mut self, ply: usize, m: Move) {
        let ply_idx = PVTable::idx_from_ply(ply);
        self.table[ply_idx] = m;

        let next_ply_idx = PVTable::idx_from_ply(ply + 1);
        let end = next_ply_idx + MAX_DEPTH - ply - 1;

        self.table.copy_within(next_ply_idx..end, ply_idx + 1)
    }

    fn get_pv_line(&self) -> Vec<Move> {
        self.table
            .iter()
            .copied()
            .take_while(|m| *m != NULL_MOVE)
            .collect()
    }
}

pub struct Searcher<T: TT> {
    aborted: bool,
    start: Instant,
    time_limit_ms: u128,
    root_depth: i32,
    pub ply: i32,
    c_mul: i32,
    pub tt: T,
    pub pv_table: PVTable,
    pub km: KillerMoves,
    pub hh: HistoryTable,
    prev_moves: PrevMoves,
    nodes: usize,
}

impl<T: TT> Searcher<T> {
    fn new(tt: T, prev_moves: PrevMoves, time_limit_ms: u128) -> Searcher<T> {
        Searcher {
            aborted: false,
            start: Instant::now(),
            time_limit_ms,
            root_depth: 0,
            ply: 0,
            c_mul: 0,
            tt,
            pv_table: PVTable::default(),
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

        // TODO test to see order of these checks performance
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
        let mut tt_entry_score = EntryScore::new_alpha(alpha, self.ply);

        for m in ml {
            let Some(score) = self.try_move(b, m, alpha, beta, depth) else {
                continue;
            };

            if score > alpha {
                alpha = score;
                best_res = Some((alpha, m));
                tt_entry_score = EntryScore::new_pv(alpha, self.ply);
                self.pv_table.set(self.ply as usize, m);
            }

            if score >= beta {
                self.tt.insert(
                    b.hash(),
                    EntryScore::new_beta(beta, self.ply),
                    Some(m),
                    self.draft(),
                );
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
            self.tt.insert(
                b.hash(),
                EntryScore::new_pv(q_score, self.ply),
                None,
                self.draft(),
            );
            return q_score;
        }

        if let Some(score) = self
            .tt
            .get_score(b.hash(), self.draft(), self.ply, alpha, beta)
        {
            return score;
        }

        // TODO is this needed?
        self.pv_table.table[PVTable::idx_from_ply(self.ply as usize)] = NULL_MOVE;

        let in_check = is_in_check(b);

        let mut best_move = None;
        let mut tt_entry_score = EntryScore::new_alpha(alpha, self.ply);

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

                if score > alpha {
                    // it is safe to unwrap here as the move is legal at this point
                    score = self.try_move(b, m, alpha, beta, depth).unwrap();
                }

                score
            };

            if score >= beta {
                self.store_tt(b.hash(), EntryScore::new_beta(beta, self.ply), Some(m));

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
                tt_entry_score = EntryScore::new_pv(alpha, self.ply);
                found_pv = true;
                self.pv_table.set(self.ply as usize, m);
            }
        }

        if !has_moved {
            alpha = if in_check {
                CHECKMATE + self.ply
            } else {
                STALEMATE
            };

            tt_entry_score = EntryScore::new_pv(alpha, self.ply);
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

    // fn q_search(&mut self, b: &Board, mut alpha: i32, beta: i32) -> i32 {
    //     let eval = eval(b, self.c_mul);

    //     if self.ply > QSEARCH_MAX_PLY as i32 {
    //         return eval;
    //     }

    //     if eval >= beta {
    //         return beta;
    //     }

    //     if alpha < eval {
    //         alpha = eval;
    //     }

    //     alpha
    // }

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

fn delta_prune(b: &Board, alpha: i32, eval: i32, m: Move) -> bool {
    eval + PIECE_VALUES[m.xpiece() as usize] + 200 < alpha
        && !m.move_type().is_promo()
        && (b.all_occ() ^ b.pawns(WHITE) ^ b.pawns(BLACK)).count_ones() > 4
}

pub fn iterative_deepening(board: &Board, tt: impl TT, prev_moves: PrevMoves) -> SearchResult {
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

        write_info(&s, board, res, depth);
    }

    res.ok_or(SearchError::NoMove)
}

pub fn lazy_smp(
    board: &Board,
    tt: Arc<SmpTTable>,
    prev_moves: PrevMoves,
    num_threads: usize,
) -> SearchResult {
    let mut res = None;

    let mut smp = LazySmp::new(tt, prev_moves, TIME_LIMIT_MS, num_threads);

    for depth in 1..MAX_DEPTH {
        let iter_res = smp.run_iter(board, depth);

        if iter_res.is_none() {
            break;
        }

        res = iter_res;

        write_info(&smp.main, board, res, depth);
    }

    res.ok_or(SearchError::NoMove)
}

struct LazySmp {
    main: Searcher<Arc<SmpTTable>>,
    helpers: Vec<Searcher<Arc<SmpTTable>>>,
}

impl LazySmp {
    fn new(
        tt: Arc<SmpTTable>,
        prev_moves: PrevMoves,
        time_limit_ms: u128,
        num_threads: usize,
    ) -> LazySmp {
        LazySmp {
            main: Searcher::new(tt.clone(), prev_moves.clone(), time_limit_ms),
            helpers: (1..num_threads)
                .map(|_| Searcher::new(tt.clone(), prev_moves.clone(), time_limit_ms))
                .collect(),
        }
    }

    fn run_iter(&mut self, board: &Board, depth: usize) -> Option<(i32, Move)> {
        let alpha_window = MIN_SCORE;
        let beta_window = MAX_SCORE;

        let mut iter_res = None;
        std::thread::scope(|scope| {
            for h in self.helpers.iter_mut() {
                scope.spawn(|| h.root_pvs(board, alpha_window, beta_window, depth));
            }

            let res = self.main.root_pvs(board, alpha_window, beta_window, depth);

            if !self.main.has_aborted() {
                iter_res = res
            }
        });

        iter_res
    }
}

fn write_info<T: TT>(s: &Searcher<T>, _b: &Board, res: Option<(i32, Move)>, depth: usize) {
    let pv_str =
        // s.tt.get_full_pv(b)
        s.pv_table.get_pv_line()
            .iter()
            .fold(String::new(), |pv, m| pv + &m.as_uci_string() + " ");

    // dbg!(&s.pv_table.table[..10]);

    let (score, _) = res.unwrap_or((MIN_SCORE, NULL_MOVE));

    let nps = s.nodes as f64 / s.start.elapsed().as_secs_f64();
    println!(
        "info depth {} score cp {} nps {:.0} pv {}",
        depth, score, nps, pv_str
    )
}
#[test]
fn pv_table_sets_pv_line() {
    crate::init();

    let pv_line: Vec<_> = (1..10).map(Move::_new_from_u32).collect();

    let mut pv_table = PVTable::default();

    for (ply, m) in pv_line.iter().enumerate().rev() {
        pv_table.set(ply, *m);
    }

    assert_eq!(pv_table.get_pv_line(), pv_line);

    for (ply, m) in pv_line.iter().enumerate().rev() {
        assert_eq!(pv_table.get(ply), *m);
    }
}
