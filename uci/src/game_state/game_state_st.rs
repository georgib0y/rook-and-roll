use crate::game_state::UciGameState;
use chess::board::board::Board;
use chess::movegen::moves::PrevMoves;
use chess::search::search::single_searcher::iterative_deepening;
use chess::search::search::SeachResult;
use chess::search::tt::tt::TTable;
use std::io::Write;
use std::{io};

pub struct GameStateST {
    tt: TTable,
    board: Board,
    prev_moves: PrevMoves,
}

impl GameStateST {
    pub fn new() -> GameStateST {
        GameStateST {
            tt: TTable::new(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
        }
    }
}

impl UciGameState for GameStateST {
    fn new_game(&mut self) {
        self.tt.clear()
    }

    fn is_ready(&self, out: &mut impl Write) -> io::Result<()> {
        writeln!(out, "readyok")
    }

    fn set_position(&mut self, b: Board, prev_moves: PrevMoves) {
        self.board = b;
        self.prev_moves = prev_moves;
    }

    fn go(&mut self, out: &mut impl Write) -> SeachResult {
        iterative_deepening(&self.board, &mut self.tt, &mut self.prev_moves, out)
    }
}
