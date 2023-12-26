#![allow(unused)]

use crate::board::Board;
use crate::movegen::move_list::StackMoveList;
use crate::movegen::movegen::*;
use crate::movegen::moves::{Move, PrevMoves};
use crate::search::tt::PerftTT;

pub struct Perft {
    pub mc: usize,
    prev_moves: PrevMoves,
}

impl Perft {
    pub fn new() -> Perft {
        Perft {
            mc: 0,
            prev_moves: PrevMoves::new(),
        }
    }

    pub fn perft(&mut self, board: &Board, depth: usize) {
        if depth == 0 {
            self.mc += 1;
            return;
        }

        let mut ml = StackMoveList::default();

        if is_in_check(board) {
            gen_check_moves(board, &mut ml);
        } else {
            gen_all_moves(board, &mut ml);
        }

        for m in ml {
            let b = board.copy_make(m);

            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
                continue;
            }

            // println!("{b}\n{m}: {}", m.as_uci_string());

            self.perft(&b, depth - 1);
        }
    }

    pub fn perftree_root(&self, depth: usize, fen: &str, moves_strs: Option<&String>) {
        let mut board = Board::new_fen(fen).unwrap();
        if let Some(moves_str) = moves_strs {
            for m in moves_str.split(' ') {
                board = board.copy_make(Move::new_from_text(m, &board));
            }
        }

        let mut total = 0;

        let mut ml = StackMoveList::default();
        gen_moves(&board, &mut ml, is_in_check(&board));

        for m in ml {
            let b = board.copy_make(m);
            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
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

        let mut ml = StackMoveList::default();
        gen_moves(board, &mut ml, is_in_check(board));

        let mut move_count = 0;

        for m in ml {
            let b = board.copy_make(m);
            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
                continue;
            }

            move_count += self.perftree(&b, depth - 1);
        }

        move_count
    }
}

#[derive(Default)]
pub struct HashPerft {
    prev_moves: PrevMoves,
    tt: PerftTT,
}

impl HashPerft {
    pub fn new() -> HashPerft {
        HashPerft::default()
    }

    pub fn perft(&mut self, board: &Board, depth: u64) -> u64 {
        if depth == 0 {
            self.tt.store(board.hash(), 1, 0);
            return 1;
        }

        if let Some(count) = self.tt.get_count(board.hash(), depth) {
            return count;
        }

        let mut mc = 0;

        let mut ml = StackMoveList::default();
        gen_moves(board, &mut ml, is_in_check(board));

        for m in ml {
            let b = board.copy_make(m);

            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
                continue;
            }

            mc += self.perft(&b, depth - 1);
        }

        self.tt.store(board.hash(), mc, depth);
        mc
    }

    pub fn perftree_root(&mut self, depth: usize, fen: &str, moves_strs: Option<&String>) {
        let mut board = Board::new_fen(fen).unwrap();
        if let Some(moves_str) = moves_strs {
            for m in moves_str.split(' ') {
                board = board.copy_make(Move::new_from_text(m, &board));
            }
        }

        let mut total = 0;

        let mut ml = StackMoveList::default();
        gen_moves(&board, &mut ml, is_in_check(&board));

        for m in ml {
            let b = board.copy_make(m);
            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
                continue;
            }

            let count = self.perftree(&b, depth - 1);
            println!("{} {}", m.as_uci_string(), count);
            total += count
        }

        println!("\n{total}");
    }

    pub fn perftree(&mut self, board: &Board, depth: usize) -> usize {
        if let Some(count) = self.tt.get_count(board.hash(), depth as u64) {
            return count as usize;
        }

        if depth == 0 {
            self.tt.store(board.hash(), 1, 0);
            return 1;
        }

        let mut ml = StackMoveList::default();
        gen_moves(board, &mut ml, is_in_check(board));

        let mut move_count = 0;

        for m in ml {
            let b = board.copy_make(m);
            if moved_into_check(&b, m) || !is_legal_move(&b, m, &self.prev_moves) {
                continue;
            }

            move_count += self.perftree(&b, depth - 1);
        }

        self.tt.store(board.hash(), move_count as u64, depth as u64);
        move_count
    }
}
