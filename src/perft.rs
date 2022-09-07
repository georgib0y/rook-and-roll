

/*
perft/search/movegen plan

check if illegal move (if king moved into check or something a step down in recusion, in minmax can
return values that change nothing as the move shouldnt have occured?
including whether a castle was legal (saves time in movegen)

 */

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use threadpool::ThreadPool;
use crate::{Board, movegen, MoveTables};
use crate::moves::Move;
use crate::movegen::*;

pub fn perft(board: &Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut mc = 0;

    let check = is_in_check(board);
    // let check = false;
    let moves = gen_moves(board, check);
    for m in moves {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, &m)) || !is_legal_move(&b, &m) { continue; }
        mc += perft(&b, depth-1);
    }

    mc
}

pub fn perftree_root(depth: usize, fen: &str, moves_strs: Option<&String>) {
    let mut board = Board::new_fen(fen);
    if let Some(moves_str) = moves_strs {
        for m in moves_str.split(' ') {
            board = board.copy_make(&Move::new_from_text(m, &board));
        }
    }

    let mut total = 0;
    let check = is_in_check(&board);
    for m in gen_moves(&board, check) {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, &m)) || !is_legal_move(&b, &m) { continue; }
        let count = perftree(&b, depth - 1);
        println!("{} {}", m.as_uci_string(), count);
        total += count
    }

    println!("\n{total}");
}

pub fn perftree(board: &Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut move_count = 0;
    let check = is_in_check(board);
    for m in gen_moves(board, check) {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, &m)) || !is_legal_move(&b, &m) { continue; }
        move_count += perftree(&b, depth - 1);
    }

    move_count

}

pub fn perft_mt_root(board: Arc<Board>, depth: usize, workers: usize) -> usize {

    let pool = ThreadPool::new(workers);
    let move_count: Arc<Mutex<usize>> = Arc::new(Mutex::new( 0));
    let check = is_in_check(&board);
    for m in gen_moves(&board, check) {
        let mut move_count_clone = Arc::clone(&move_count);
        let board_clone = Arc::clone(&board);
        if !check && moved_into_check(&board_clone, &m) || !is_legal_move(&board_clone, &m) { continue; }
        pool.execute(move || {
            let b = board_clone.copy_make(&m);
            let mc = perft(&b, depth - 1);
            let mut total_mc = move_count_clone.lock().unwrap();
            *total_mc += mc;
        });
    }

    pool.join();
    let total = move_count.lock().unwrap();
    *total
}