use crate::eval::{eval, CHECKMATE};
use crate::movegen::{gen_attacks, gen_moves, gen_quiet};
use crate::tt::{ALPHA_TT_FLAG, BETA_TT_FLAG, PV_TT_FLAG};
use crate::{
    gen_check_moves, is_in_check, is_legal_move, moved_into_check, tt, Board, Move, MoveTables,
    SeqTT,
};
use lazy_static::lazy_static;
use log::info;
use std::io;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const MAX_DEPTH: usize = 6;
const MAX_TIME: u64 = 5000;
const MAX_QUIESCE_DEPTH: usize = 10;

// using lazy static atomic cause unsafe keyword scares me
lazy_static! {
    static ref NODES: AtomicU64 = AtomicU64::new(0);
}

// TODO impl time constraint
pub fn iterative_deepening(board: &Board, tt: &mut SeqTT) -> Option<Move> {
    // trying buffered output
    let mut buf = BufWriter::new(io::stdout());

    let mut best_move = None;
    let mut best_score = 0;

    let start = Instant::now();
    NODES.store(0, Ordering::SeqCst); // set nodes to 0

    for depth in 1..=MAX_DEPTH {
        (best_score, best_move) = root_negamax(board, depth, tt);

        let elapsed = start.elapsed().as_millis() as f64; // so that there is no divide by 0 err
        let nodes = NODES.load(Ordering::SeqCst);
        let mut nps = nodes as f64 / (elapsed / 1000f64);
        if nps.is_infinite() {
            nps = 0f64;
        }

        let info = format!("info depth {} score cp {} nps {}", depth, best_score, (nps / 1000f64) as usize);
        info!(target: "output", "{}", info);
        println!("{}", info);
    }

    best_move
}

pub fn root_negamax(board: &Board, depth: usize, tt: &mut SeqTT) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = i32::MIN + 1;
    let p_mul = if board.colour_to_move == 0 { 1 } else { -1 };

    let check = is_in_check(board);
    for m in &gen_moves(board, check) {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) {
            continue;
        }

        // TODO changed alpha from i32::MIN+1 so that at least one move gets chosen each time, may be bad for elo dunno
        // TODO could lower beta bounds by best_score but im trying it without
        let mut score = -negamax(&b, depth - 1, 1, i32::MIN + 2, -best_score, -p_mul, tt);

        if score > best_score {
            best_move = Some(*m);
            best_score = score;
        }
    }

    (best_score, best_move)
}

pub fn negamax(
    board: &Board,
    depth: usize,
    ply: usize,
    mut alpha: i32,
    beta: i32,
    p_mul: i32, // player multiplier - to be passed down to eval
    tt: &mut SeqTT,
) -> i32 {
    NODES.fetch_add(1, Ordering::SeqCst); // count node

    if let Some(score) = tt.get(board.hash, depth, alpha, beta) {
        return score;
    }

    if depth == 0 {
        let eval = quiesce(board, ply, ply + MAX_QUIESCE_DEPTH, alpha, beta, p_mul);
        tt.insert(board.hash, eval, PV_TT_FLAG, 0);
        return eval;
    }

    let mut table_entry_type = ALPHA_TT_FLAG;

    let check = is_in_check(board);
    let mut not_moved = true;
    let mut score = i32::MIN;
    for m in &gen_moves(board, check) {
        not_moved = false;

        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) {
            continue;
        }

        score = -negamax(&b, depth - 1, ply + 1, -beta, -alpha, -p_mul, tt);

        if score > alpha {
            table_entry_type = PV_TT_FLAG;
            alpha = score;
        }

        if score >= beta {
            tt.insert(board.hash, beta, BETA_TT_FLAG, depth);
            return beta;
        }
    }

    tt.insert(board.hash, alpha, table_entry_type, depth);
    // if hasn't moved and in check -> checkmate
    if not_moved && check {
        CHECKMATE + ply as i32
    } else {
        alpha
    }
}

fn quiesce(
    board: &Board,
    ply: usize,
    max_ply: usize,
    mut alpha: i32,
    beta: i32,
    p_mul: i32,
) -> i32 {
    NODES.fetch_add(1, Ordering::SeqCst);

    // cut off at certain depth
    // if ply == max_ply {
    //     return eval(&board, p_mul);
    // }

    let check = is_in_check(board);
    let mut moves;

    if check {
        moves = gen_check_moves(board);
        if moves.is_empty() {
            return -CHECKMATE + ply as i32;
        }
    } else {
        let eval = eval(board, p_mul);

        if eval >= beta {
            return beta;
        }
        if alpha < eval {
            alpha = eval;
        }

        moves = gen_attacks(board);
    }

    for m in &moves {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) {
            continue;
        }

        let score = -quiesce(&b, ply + 1, max_ply, -beta, -alpha, -p_mul);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
