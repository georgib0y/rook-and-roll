use crate::board::{Board, WHITE};
use crate::movegen::moves::{KillerMoves, Move, PrevMoves, NULL_MOVE};
use crate::search::eval::PAWN_VALUE;
use crate::search::search::root_pvs;
use crate::search::searchers::{SeachResult, SearchError, Searcher, MAX_SCORE};
use crate::search::searchers::{MAX_DEPTH, MIN_SCORE};
use crate::search::tt::EntryScore;
use crate::search::tt::TTable;
use crate::search::HistoryTable;
use std::cmp::{max, min};
use std::io::Write;
use std::time::Instant;

const TIME_LIMIT_MS: u128 = 1500;

pub fn iterative_deepening(
    board: &Board,
    tt: &mut TTable,
    prev_moves: &mut PrevMoves,
    out: &mut impl Write,
) -> SeachResult {
    let mut searcher = SingleSearcher::new(tt, prev_moves);

    let start = Instant::now();

    let mut best_result: (i32, Move) = (MIN_SCORE, NULL_MOVE);

    for depth in 1..MAX_DEPTH {
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
            .fold(String::new(), |pv, m| pv.add(&m.as_uci_string()).add(" "));

        if start.elapsed().as_millis() > 0 {
            writeln!(out, "info depth {} score cp {} pv {}", depth, res.0, pv_str).unwrap();

            let hits = searcher.tt.hits();
            let collisions = searcher.tt.collisions();
            let misses = searcher.tt.misses();
            let total = hits + misses + collisions;
            let percent = (hits as f64 / total as f64) * 100.0;
            println!(
                "{} hits, {} collisions, {} misses, {} total, {}%",
                hits, collisions, misses, total, percent
            );
        }

        best_result = res;
    }

    Ok(best_result)
}

fn root_search(s: &mut SingleSearcher, b: &Board, best_score: i32, depth: usize) -> SeachResult {
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

struct SingleSearcher<'a> {
    abort: bool,
    root_depth: i32,
    ply: i32,
    colour_multiplier: i32,
    tt: &'a mut TTable,
    km: KillerMoves,
    hh: HistoryTable,
    prev_moves: &'a mut PrevMoves,
}

impl<'a> SingleSearcher<'a> {
    pub fn new(tt: &'a mut TTable, prev_moves: &'a mut PrevMoves) -> SingleSearcher<'a> {
        SingleSearcher {
            abort: false,
            root_depth: 0,
            ply: 0,
            colour_multiplier: 0,
            tt,
            km: KillerMoves::new(),
            hh: HistoryTable::new(),
            prev_moves,
        }
    }

    fn abort(&mut self) {
        self.abort = true;
    }
}

impl<'a> Searcher for SingleSearcher<'a> {
    fn init_search(&mut self, b: &Board, depth: usize) {
        self.colour_multiplier = if b.ctm() == WHITE { 1 } else { -1 };
        self.ply = 0;
        self.root_depth = depth as i32;
    }

    fn has_aborted(&self) -> bool {
        self.abort
    }

    fn probe_tt(&self, hash: u64, alpha: i32, beta: i32) -> Option<i32> {
        self.tt.get_score(hash, self.draft(), alpha, beta)
        // None
    }

    fn store_tt(&mut self, hash: u64, score: EntryScore, best_move: Option<Move>) {
        self.tt.insert(hash, score, best_move, self.draft())
    }

    fn get_tt_best_move(&self, hash: u64) -> Option<Move> {
        self.tt.get_bestmove(hash)
        // None
    }

    fn get_tt_pv_move(&mut self, hash: u64) -> Option<Move> {
        self.tt.get_bestmove(hash)
        // None
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
}
