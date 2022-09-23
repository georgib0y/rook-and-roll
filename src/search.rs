use std::io;
use std::io::{BufWriter, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use log::info;
use crate::{Board, gen_check_moves, is_in_check, is_legal_move, Move, moved_into_check, MoveTables, SeqTT, tt};
use crate::eval::{CHECKMATE, eval};
use crate::movegen::{gen_attacks, gen_moves, gen_quiet};

const MAX_DEPTH: usize = 5;
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
        (best_score, best_move) = root_pvs(board, depth, tt);

        let elapsed = start.elapsed().as_millis() as f64; // so that there is no divide by 0 err
        let nodes = NODES.load(Ordering::SeqCst);
        let mut nps = nodes as f64 / (elapsed / 1000f64);
        if nps.is_infinite() { nps = 0f64;}

        let info = format!("info depth {} score cp {}", depth, best_score);
        info!(target: "output", "{}", info);
        println!("{}", info);
        // io::stdout().flush().unwrap();

        // score cp {best_score}");
        // println!("info score cp {best_score} nps {}", (nps / 1000f64) as usize);
        // println!("info depth {}", MAX_DEPTH);
        io::stdout().flush().unwrap();
    }
    best_move
}

pub fn root_pvs(board: &Board, depth: usize, tt: &mut SeqTT) -> (i32, Option<Move>) {
    let mut best_move = None;
    let mut best_score = i32::MIN+1;
    let p_mul = if board.colour_to_move == 0 {1} else {-1};


    let check = is_in_check(board);
    for m in &gen_moves(board, check) {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) { continue; }

        // TODO changed alpha from i32::MIN+1 so that at least one move gets chosen each time, may be bad for elo dunno
        // TODO could lower beta bounds by best_score but im trying it without
        let mut score = -pvs(&b, depth-1, 1, i32::MIN+2, -best_score, -p_mul, tt);

        if score > best_score {
            best_move = Some(*m);
            best_score = score;
        }
    }


    (best_score, best_move)
}

pub fn pvs(
    board: &Board,
    depth: usize,
    ply: usize,
    mut alpha: i32,
    beta: i32,
    p_mul: i32,         // player multiplier - to be passed down to eval
    tt: &mut SeqTT,
) -> i32 {
    NODES.fetch_add(1, Ordering::SeqCst); // count node

    if depth == 0 {
        return quiesce(board, ply, ply+MAX_QUIESCE_DEPTH, alpha, beta, p_mul);
    }

    let mut pv = true;

    let check = is_in_check(board);
    let mut not_moved = true;

    for m in &gen_moves(board, check) {
        not_moved = false;

        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) { continue; }

        let mut score: i32;
        if pv {
            score = -pvs(&b, depth-1, ply+1, -beta, -alpha, -p_mul, tt);
            pv = false;
        } else {
            score = -pvs(&b, depth-1, ply+1, -alpha-1, -alpha, -p_mul, tt);
            if score > alpha {
                score = -pvs(&b, depth-1, ply+1, -beta, -alpha, -p_mul, tt);
            }
        }

        if score >= beta { return beta; }
        if score > alpha { alpha = score; }
    }

    // if hasn't moved and in check -> checkmate
    if not_moved && check {
        CHECKMATE + ply as i32
    } else {
        alpha
    }
}

fn quiesce(board: &Board, ply: usize, max_ply: usize, mut alpha: i32, beta: i32, p_mul: i32) -> i32 {
    NODES.fetch_add(1, Ordering::SeqCst);

    // cut off at certain depth
    if ply == max_ply { return eval(&board, p_mul); }

    let check = is_in_check(board);
    let mut moves;

    if check {
        moves = gen_check_moves(board);
        if moves.is_empty() { return -CHECKMATE + ply as i32; }
    } else {
        let eval = eval(board, p_mul);

        if eval >= beta { return beta; }
        if alpha < eval { alpha = eval; }

        moves = gen_attacks(board);
    }

    for m in &moves {
        let b = board.copy_make(m);
        if (!check && moved_into_check(&b, m)) || !is_legal_move(&b, m) { continue; }
        // println!("{b}in check: {check}\n{m}");
        let score = -quiesce(&b, ply+1, max_ply, -beta, -alpha, -p_mul);

        if score >= beta { return beta; }
        if alpha < score { alpha = score; }
    }

    alpha

}