/*
perft/search/movegen plan

check if illegal move (if king moved into check or something a step down in recusion, in minmax can
return values that change nothing as the move shouldnt have occured?
including whether a castle was legal (saves time in movegen)

 */

// use crate::movegen::*;
use crate::movegen::*;
use crate::moves::{Move, PrevMoves};
use crate::{movegen, Board, MoveTables};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::eval::{eval, gen_mat_value, gen_pst_value};
// use rayon::prelude::*;
// use rayon::ThreadPoolBuilder;
use crate::tt::ORDER;

pub struct Perft {
    pub mt: MoveTables,
    pub mc: usize,
    prev_moves: PrevMoves
}

impl Perft {
    pub fn new() -> Perft {
        Perft {
            mt: MoveTables::new(),
            mc: 0,
            prev_moves: PrevMoves::new()
        }
    }

    pub fn perft(&mut self, board: &Board, depth: usize) {
        if depth == 0 {
            self.mc += 1;
            return;
        }
        let check = is_in_check(board, &self.mt);
        let move_list = MoveList::all(board, &self.mt, check);

        for m in move_list.moves {
            let b = board.copy_make(m);
            if !check && (moved_into_check(&b, &self.mt, m))
                || !is_legal_move(&b, &self.mt, m, &self.prev_moves) { continue; }
            self.perft(&b, depth - 1);
        }
    }

    pub fn perftree_root(&self, depth: usize, fen: &str, moves_strs: Option<&String>) {
        let mut board = Board::new_fen(fen);
        if let Some(moves_str) = moves_strs {
            for m in moves_str.split(' ') {
                board = board.copy_make(Move::new_from_text(m, &board));
            }
        }

        let mut total = 0;
        let check = is_in_check(&board, &self.mt);
        let move_list = MoveList::all(&board, &self.mt, check);
        for m in move_list.moves { //gen_moves(&board, check) {
            let b = board.copy_make(m);
            if !check && (moved_into_check(&b, &self.mt, m))
                || !is_legal_move(&b, &self.mt, m, &self.prev_moves) { continue; }
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
        let check = is_in_check(board, &self.mt, );
        let move_list = MoveList::all(board, &self.mt, check);
        for m in move_list.moves { //gen_moves(board, check) {
            let b = board.copy_make(m);
            if !check && (moved_into_check(&b, &self.mt, m))
                || !is_legal_move(&b, &self.mt, m, &self.prev_moves) { continue; }
            move_count += self.perftree(&b, depth - 1);
        }

        move_count
    }
}