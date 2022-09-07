use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use crate::{Board, is_in_check, is_legal_move, Move, moved_into_check, MoveTables};
use crate::eval::eval;
use crate::movegen::gen_moves;

const MAX_DEPTH: usize = 9;
const MAX_TIME: u64 = 5000;

// using lazy static atomic cause unsafe keyword scares me
lazy_static! {
    static ref NPS: AtomicU64 = AtomicU64::new(0);
}

// TODO impl time constraint
pub fn iterative_deepening(board: &Board) -> Option<Move> {
    let mut best_move = None;

    let start = Instant::now();
    NPS.store(0, Ordering::SeqCst); // ordering doesnt really matter
    for depth in 1..=MAX_DEPTH {
        best_move = root_pvs(board, depth);
        println!("{depth} done");
    }


    let nps = NPS.load(Ordering::SeqCst) as f64 / (start.elapsed().as_millis() as f64 / 1000f64);
    println!("Done. Total elapsed = {} NPS = {nps}", start.elapsed().as_millis());


    best_move
}

pub fn root_pvs(board: &Board, depth: usize) -> Option<Move> {
    let mut best_move = None;
    let mut best_score = i32::MIN+1;
    let p_mul = if board.colour_to_move == 0 {1} else {0};


    let check = is_in_check(board);
    for m in &gen_moves(board, check) {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) { continue; }

        // TODO could lower beta bounds by best_score but im trying it without
        let mut score = -pvs(&b, depth-1, i32::MIN+1, -best_score, -p_mul);

        if score > best_score {
            best_move = Some(*m);
            best_score = score;
        }
    }

    best_move
}

pub fn pvs(
    board: &Board,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    p_mul: i32,         // player multiplier - to be passed down to eval
) -> i32 {
    NPS.fetch_add(1, Ordering::SeqCst);

    if depth == 0 {
        return eval(board, p_mul)
    }

    let mut pv = true;

    let check = is_in_check(board);

    for m in &gen_moves(board, check) {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) { continue; }

        let mut score: i32;

        // TODO could take away this pv if statement and have it above the for to make it a *little* more branchless
        if pv {
            score = -pvs(&b, depth-1, -beta, -alpha, -p_mul);
        } else {
            score = -pvs(&b, depth-1, -alpha-1, -alpha, -p_mul);
            if score > alpha {
                score = -pvs(&b, depth-1, -beta, -alpha, -p_mul);
            }
        }

        if score >= beta { return beta; }
        if score > alpha { alpha = score; }
    }


    alpha
}