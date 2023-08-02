use crate::board::Board;
use crate::movegen::*;
use crate::moves::{Move, PrevMoves};

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

    #[allow(unused)]
    pub fn perft(&mut self, board: &Board, depth: usize) {
        if depth == 0 {
            self.mc += 1;
            return;
        }

        let move_set = MoveSet::get_move_set(MoveSet::All, board);
        let move_list = MoveList::get_moves_unscored(board, move_set);

        for m in move_list.moves {
            let b = board.copy_make(m);
            if move_set != MoveSet::Check && (moved_into_check(&b, m))
                || !is_legal_move(&b, m, &self.prev_moves)
            {
                continue;
            }
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
        let move_set = MoveSet::get_move_set(MoveSet::All, &board);
        let move_list = MoveList::get_moves_unscored(&board, move_set);

        for m in move_list.moves {
            let b = board.copy_make(m);
            if move_set != MoveSet::Check && (moved_into_check(&b, m))
                || !is_legal_move(&b, m, &self.prev_moves)
            {
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
        let move_set = MoveSet::get_move_set(MoveSet::All, board);
        let move_list = MoveList::get_moves_unscored(board, move_set);

        for m in move_list.moves {
            let b = board.copy_make(m);
            if move_set != MoveSet::Check && (moved_into_check(&b, m))
                || !is_legal_move(&b, m, &self.prev_moves)
            {
                continue;
            }
            move_count += self.perftree(&b, depth - 1);
        }

        move_count
    }
}
