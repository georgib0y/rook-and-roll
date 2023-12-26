use crate::game_state::UciGameState;
use chess::board::Board;
use chess::movegen::moves::PrevMoves;
use chess::search::searchers::smp_searcher::lazy_smp;
use chess::search::searchers::SeachResult;
use chess::search::tt::SmpTTable;
use std::io::Write;
use std::sync::Arc;

pub struct GameStateMT {
    tt: Arc<SmpTTable>,
    board: Board,
    prev_moves: PrevMoves,
    num_threads: usize,
}

impl GameStateMT {
    pub fn new(num_threads: usize) -> GameStateMT {
        GameStateMT {
            tt: SmpTTable::new(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
            num_threads,
        }
    }
}

impl UciGameState for GameStateMT {
    fn new_game(&mut self) {
        self.tt.clear();
    }

    fn is_ready(&self, out: &mut impl Write) -> std::io::Result<()> {
        writeln!(out, "readyok")
    }

    fn set_position(&mut self, b: Board, prev_move: PrevMoves) {
        self.board = b;
        self.prev_moves = prev_move;
    }

    fn go(&mut self, out: &mut impl Write) -> SeachResult {
        lazy_smp(
            &self.board,
            self.tt.clone(),
            &self.prev_moves,
            self.num_threads,
            out,
        )
    }
}
