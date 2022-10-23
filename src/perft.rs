/*
perft/search/movegen plan

check if illegal move (if king moved into check or something a step down in recusion, in minmax can
return values that change nothing as the move shouldnt have occured?
including whether a castle was legal (saves time in movegen)

 */

// use crate::movegen::*;
use crate::movegen::*;
use crate::moves::Move;
use crate::{movegen, Board, MoveTables};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
// use rayon::prelude::*;
// use rayon::ThreadPoolBuilder;
use crate::tt::ORDER;

pub struct Perft {
    pub mt: MoveTables,
    pub mc: usize
}

impl Perft {
    pub fn new() -> Perft {
        Perft { mt: MoveTables::new(), mc: 0 }
    }

    pub fn perft_new_movegen(&mut self, board: &Board, depth: usize) {
        if depth == 0 {
            self.mc += 1;
            return;
        }
        let check = is_in_check(board, &self.mt);
        let move_list = MoveList::all(board, &self.mt, check);

        for m in move_list.moves {
            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, &self.mt, &m)) || !is_legal_move(&b, &self.mt, &m) {
                continue;
            }
            self.perft_new_movegen(&b, depth - 1);
        }
    }

    pub fn perftree_root(&self, depth: usize, fen: &str, moves_strs: Option<&String>) {
        let mut board = Board::new_fen(fen);
        if let Some(moves_str) = moves_strs {
            for m in moves_str.split(' ') {
                board = board.copy_make(&Move::new_from_text(m, &board));
            }
        }

        let mut total = 0;
        let check = is_in_check(&board, &self.mt);
        let move_list = MoveList::all(&board, &self.mt, check);
        for m in move_list.moves { //gen_moves(&board, check) {
            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, &self.mt,  &m)) || !is_legal_move(&b, &self.mt, &m) {
                continue;
            }
            let count = self.perftree(&b, depth - 1);
            println!("{} {}", m.as_uci_string(), count);
            total += count
        }

        println!("\n{total}");
    }

    pub fn perftree(&self, board: &Board, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut move_count = 0;
        let check = is_in_check(board, &self.mt,);
        let move_list = MoveList::all(board, &self.mt, check);
        for m in move_list.moves  { //gen_moves(board, check) {
            let b = board.copy_make(&m);
            if (!check && moved_into_check(&b, &self.mt, &m)) || !is_legal_move(&b, &self.mt, &m) {
                continue;
            }
            move_count += self.perftree(&b, depth - 1);
        }

        move_count
    }

    pub fn perft_mt_root(&mut self, board: Board, depth: usize, workers: usize) {
        let move_count_arc = Arc::new(AtomicUsize::new(0));

        let check = is_in_check(&board, &self.mt);
        let move_list = MoveList::all(&board, &self.mt, check);

        let pool = ThreadPool::new(workers);

        for m in move_list.moves {
            let b = board.copy_make(&m);

            if !check && moved_into_check(&b, &self.mt, &m)
                || !is_legal_move(&b, &self.mt, &m) { continue; }

            let move_count = Arc::clone(&move_count_arc);
            let mt = self.mt.clone();
            pool.execute(move || {
                let mut mc = perft_mt(&b, &mt,depth - 1);
                // println!("{mc}");
                move_count.fetch_add(mc, Ordering::SeqCst);

            });
        }


        pool.join();

        self.mc = move_count_arc.load(Ordering::SeqCst);
    }
}

fn perft_mt(board: &Board, mt: &MoveTables, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }

    let mut mc = 0;
    let check = is_in_check(board, mt);

    let move_list = MoveList::all(board, mt, check);
    for m in move_list.moves {
        let b = board.copy_make(&m);
        if (!check && moved_into_check(&b, mt, &m)) || !is_legal_move(&b, mt, &m) {
            continue;
        }

        mc += perft_mt(&b, mt, depth - 1);
    }

    mc
}


// pub fn perft(board: &Board, depth: usize) -> usize {
//     if depth == 0 {
//         return 1;
//     }
//
//     let mut mc = 0;
//
//     let check = is_in_check(board);
//     // let check = false;
//     let moves = gen_moves(board, check);
//     for m in moves {
//         let b = board.copy_make(&m);
//         if (!check && moved_into_check(&b, &m)) || !is_legal_move(&b, &m) {
//             continue;
//         }
//         mc += perft(&b, depth - 1);
//     }
//
//     mc
// }
//
// pub fn perft_mt_root(board: Arc<Board>, depth: usize, workers: usize) -> usize {
//     let pool = ThreadPool::new(workers);
//     let move_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
//     let check = is_in_check(&board);
//     for m in gen_moves(&board, check) {
//         let mut move_count_clone = Arc::clone(&move_count);
//         let board_clone = Arc::clone(&board);
//         if !check && moved_into_check(&board_clone, &m) || !is_legal_move(&board_clone, &m) {
//             continue;
//         }
//         pool.execute(move || {
//             let b = board_clone.copy_make(&m);
//             let mc = perft(&b, depth - 1);
//             let mut total_mc = move_count_clone.lock().unwrap();
//             *total_mc += mc;
//         });
//     }
//
//     pool.join();
//     let total = move_count.lock().unwrap();
//     *total
// }
//
//
// #[test]
// fn get_moves() {
//     let mut board = Board::new();
//
//     for m in "c2c3 d7d5 d1a4".split(' ') {
//         board = board.copy_make(&Move::new_from_text(m, &board));
//         println!("{board}");
//     }
//
//     let mt = MoveTables::new();
//
//     let check = is_in_check(&board);
//     println!("{check}");
//
//     let move_list = MoveList::all(&board, &mt, check);
//
//     for m in move_list.moves {
//         println!("{m}");
//     }
//
// }