use crate::board::{Board, WHITE};
use crate::movegen::moves::{KillerMoves, Move, PrevMoves, NULL_MOVE};
use crate::search::eval::PAWN_VALUE;
use crate::search::search::root_pvs;
use crate::search::searchers::{SeachResult, SearchError, Searcher, MAX_SCORE};
use crate::search::searchers::{MAX_DEPTH, MIN_SCORE};
use crate::search::tt::EntryScore;
use crate::search::tt::TT;
use crate::search::HistoryTable;
use std::cmp::{max, min};
use std::io::Write;
use std::time::Instant;

// const TIME_LIMIT_MS: u128 = 100000;
const TIME_LIMIT_MS: u128 = 1500;

pub fn iterative_deepening(
    board: &Board,
    tt: &mut impl TT,
    prev_moves: &mut PrevMoves,
    out: &mut impl Write,
) -> SeachResult {
    let mut searcher = SingleSearcher::new(tt, prev_moves);

    let start = Instant::now();

    let mut best_result: (i32, Move) = (MIN_SCORE, NULL_MOVE);

    for depth in 1..MAX_DEPTH {
        // for depth in 1..8 {
        if start.elapsed().as_millis() > TIME_LIMIT_MS {
            searcher.abort();
            break;
        }

        let alpha_window = MIN_SCORE;
        let beta_window = -alpha_window;

        // let res = if best_result.0 == MIN_SCORE {
        //     root_pvs(&mut searcher, board, alpha_window, beta_window, depth)?
        // } else {
        //     root_search(&mut searcher, board, best_result.0, depth)?
        // };

        let res = root_pvs(&mut searcher, board, alpha_window, beta_window, depth)?;
        let pv_str = searcher
            .tt
            .get_full_pv(board)
            .iter()
            .fold(String::new(), |pv, m| pv + &m.as_uci_string() + " ");

        // if start.elapsed().as_millis() > 0 {
        if true {
            let nps = searcher.nodes as f64 / start.elapsed().as_secs_f64();
            writeln!(
                out,
                "info depth {} score cp {} nps {:.0} pv {}",
                depth, res.0, nps, pv_str
            )
            .unwrap();

            // print_stats(&searcher)
        }

        best_result = res;
    }

    Ok(best_result)
}

#[allow(unused)]
fn root_search<T: TT>(
    s: &mut SingleSearcher<T>,
    b: &Board,
    best_score: i32,
    depth: usize,
) -> SeachResult {
    let mut alpha_window_width = 4;
    let mut beta_window_width = 4;
    let mut alpha_window = max(best_score - (PAWN_VALUE / alpha_window_width), MIN_SCORE);
    let mut beta_window = min(best_score + (PAWN_VALUE / beta_window_width), MAX_SCORE);

    dbg!(best_score);
    loop {
        dbg!(alpha_window, beta_window);
        match root_pvs(s, b, alpha_window, beta_window, depth) {
            Ok(res) => return Ok(res),
            Err(SearchError::FailHigh) => {
                if beta_window == MAX_SCORE {
                    Err(SearchError::FailHigh)?
                }

                beta_window_width /= 2;
                if beta_window_width < 2 {
                    beta_window = MAX_SCORE;
                } else {
                    beta_window = min(best_score + (PAWN_VALUE / beta_window_width), MAX_SCORE);
                }
            }
            Err(SearchError::FailLow) => {
                if alpha_window == MIN_SCORE {
                    Err(SearchError::FailLow)?
                }

                alpha_window_width /= 2;
                if alpha_window_width < 2 {
                    alpha_window = MIN_SCORE;
                } else {
                    alpha_window = max(best_score - (PAWN_VALUE / alpha_window_width), MIN_SCORE);
                }
            }
            _ => panic!("Unknown error finding move in root searchers"),
        }
    }
}

struct SingleSearcher<'a, T: TT> {
    abort: bool,
    root_depth: i32,
    ply: i32,
    colour_multiplier: i32,
    tt: &'a mut T,
    km: KillerMoves,
    hh: HistoryTable,
    prev_moves: &'a mut PrevMoves,
    nodes: usize,
}

impl<'a, T: TT> SingleSearcher<'a, T> {
    pub fn new(tt: &'a mut T, prev_moves: &'a mut PrevMoves) -> SingleSearcher<'a, T> {
        SingleSearcher {
            abort: false,
            root_depth: 0,
            ply: 0,
            colour_multiplier: 0,
            tt,
            km: KillerMoves::new(),
            hh: HistoryTable::new(),
            prev_moves,
            nodes: 0,
        }
    }

    fn abort(&mut self) {
        self.abort = true;
    }
}

impl<'a, T: TT> Searcher for SingleSearcher<'a, T> {
    fn init_search(&mut self, b: &Board, depth: usize) {
        self.colour_multiplier = if b.ctm() == WHITE { 1 } else { -1 };
        self.ply = 0;
        self.root_depth = depth as i32;
        self.nodes = 0;
    }

    fn has_aborted(&self) -> bool {
        self.abort
    }

    fn probe_tt(&self, hash: u64, alpha: i32, beta: i32) -> Option<i32> {
        self.tt.get_score(hash, self.draft(), alpha, beta)
    }

    fn store_tt(&mut self, hash: u64, score: EntryScore, best_move: Option<Move>) {
        self.tt.insert(hash, score, best_move, self.draft())
    }

    fn get_tt_best_move(&self, hash: u64) -> Option<Move> {
        self.tt.get_bestmove(hash)
    }

    fn get_tt_pv_move(&mut self, hash: u64) -> Option<Move> {
        self.tt.get_pv(hash)
    }

    fn km_get(&self, depth: usize) -> [Option<Move>; 2] {
        self.km.get_kms(depth)
    }

    fn km_store(&mut self, km: Move, depth: usize) {
        self.km.add(km, depth);
    }

    fn ply(&self) -> i32 {
        self.ply
    }

    fn draft(&self) -> i32 {
        self.root_depth - self.ply
    }

    fn colour_multiplier(&self) -> i32 {
        self.colour_multiplier
    }

    fn prev_moves(&self) -> &PrevMoves {
        self.prev_moves
    }

    fn push_ply(&mut self) {
        self.ply += 1;
        self.colour_multiplier = -self.colour_multiplier;
    }

    fn push_prev_move(&mut self, hash: u64) {
        self.prev_moves.add(hash);
    }

    fn pop_ply(&mut self) {
        self.ply -= 1;
        self.colour_multiplier *= -1;
    }

    fn pop_prev_move(&mut self, hash: u64) {
        self.prev_moves.remove(hash);
    }

    fn get_hh_score(&self, ctm: usize, from: usize, to: usize) -> u32 {
        self.hh.get(ctm, from, to)
    }

    fn store_hh_score(&mut self, ctm: usize, from: usize, to: usize, depth: usize) {
        self.hh.insert(ctm, from, to, depth)
    }

    fn add_node(&mut self) {
        self.nodes += 1;
    }
}
