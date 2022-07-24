

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

pub fn perft(
    board: &Board,
    depth: usize,
    mt: &MoveTables,
) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut mc = 0;

    // TODO moved is_in_check into copy make,

    // let check = is_in_check(board, mt);
    let check = false;
    let moves = gen_moves(board, mt, check);
    for m in moves {
        // println!("before");
        let b = board.copy_make(&m);
        // println!("after");
        // if check {
        //     println!("{board}{b}{m}");
        // }

        if (!check && moved_into_check(&b, &m, &mt)) || !is_legal_move(&b, &m, mt) { continue; }
        mc += perft(&b, depth-1, mt);
    }

    mc
}

pub fn perftree_root(depth: usize, fen: &str, moves_strs: Option<&String>) {
    let mut board = Board::new_fen(fen);
    let mt = MoveTables::new_boxed();
    if let Some(moves_str) = moves_strs {
        for m in moves_str.split(' ') {
            board = board.copy_make(&Move::new_from_text(m, &board));
        }
    }

    let mut total = 0;
    let check = is_in_check(&board, &mt);
    dbg!(check);
    for m in gen_moves(&board, &mt, check) {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, &m, &mt)) || !is_legal_move(&b, &m, &mt) { continue; }
        let count = perftree(&b, depth - 1, &mt);
        println!("{} {}", m.as_uci_string(), count);
        total += count
    }

    println!("\n{total}");
}

pub fn perftree(board: &Board, depth: usize, mt: &MoveTables) -> usize {
    if depth == 0 {
        return 1;
    }


    let mut move_count = 0;
    let check = is_in_check(board, mt);
    for m in gen_moves(board, mt, check) {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, &m, &mt)) || !is_legal_move(&b, &m, mt) { continue; }
        move_count += perftree(&b, depth - 1, mt);
    }

    move_count

}


// TODO outdated
pub fn perft_mt_root(board: Arc<Board>, depth: usize, mt: Arc<MoveTables>, workers: usize) -> usize {

    let pool = ThreadPool::new(workers);
    let move_count: Arc<Mutex<usize>> = Arc::new(Mutex::new( 0));
    for m in gen_moves(board.as_ref(), mt.as_ref(), false) {
        let mut move_count_clone = Arc::clone(&move_count);
        let mt_clone = Arc::clone(&mt);
        let board_clone = Arc::clone(&board);
        pool.execute(move || {
            let b = board_clone.copy_make(&m);
            let mc = perft_mt(&b, depth - 1, &mt_clone);
            let mut total_mc = move_count_clone.lock().unwrap();
            *total_mc += mc;
        });
    }

    pool.join();
    let total = move_count.lock().unwrap();
    *total
}

// TODO outdated
pub fn perft_mt(
    board: &Board,
    depth: usize,
    mt: &MoveTables,
) -> usize {

    if depth == 0 {
        return 1;
    }

    let mut move_count = 0;
    let check = is_in_check(board, mt);
    for m in gen_moves(board, mt, check) {
        // println!("{m}");
        let b = board.copy_make(&m);
        if !check && moved_into_check(board, &m, mt) { continue; }
        move_count += perft_mt(&b, depth-1, mt);
    }

    move_count
}

// TODO also outdated and possibly unneeded
pub fn perft_debug(
    board: &Board,
    depth: usize,
    mt: &MoveTables,
    m: Option<&Move>,
    counter: &mut Counter,
) {
    if depth == 0 {
        println!("{board}");
        counter.count_move(board, m.unwrap(), mt);
        return;
    }


    for m in gen_moves(board, mt, false) {
        // println!("{m}");
        let b = board.copy_make(&m);
        if moved_into_check(board, &m, mt) { continue; }
        perft_debug(&b, depth-1, mt, Some(&m), counter)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Counter {
    moves: usize,
    quiet: usize,
    cap: usize,
    ep_cap: usize,
    castle: usize,
    promo: usize,
    check: usize,
}

impl Counter {
    pub fn new() -> Counter {
        Counter {
            moves: 0,
            quiet: 0,
            cap: 0,
            ep_cap: 0,
            castle: 0,
            promo: 0,
            check: 0,
        }
    }

    fn count_move(&mut self, b: &Board, m: &Move, mt: &MoveTables) {
        self.moves += 1;
        match m.move_type() {
            QUIET => self.quiet += 1,
            DOUBLE => self.quiet += 1,
            CAP => self.cap += 1,
            EP => self.ep_cap += 1,
            WKINGSIDE | WQUEENSIDE | BKINGSIDE | BQUEENSIDE => self.castle += 1,
            PROMO => self.promo += 1,
            N_PROMO_CAP | R_PROMO_CAP | B_PROMO_CAP | Q_PROMO_CAP => {
                self.promo += 1;
                self.cap += 1;
            }
            _ => ()
        }

        if moved_into_check(b, &m, mt) { self.check+=1; }


        // if let MoveType::Capture = &m.move_type {
        //     // println!("{}", b);
        // }
        //
        // if m.piece == 1 {
        //     //println!("{}", b);
        // }
    }
}