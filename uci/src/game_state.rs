use std::{io::Write, sync::Arc};

use chess::{
    board::Board,
    movegen::moves::PrevMoves,
    search::{
        searcher::{iterative_deepening, lazy_smp, SearchResult},
        tt::{NoTTable, SmpTTable, TTable, TT},
    },
};

pub trait CanSearch {
    fn go(&mut self, out: &mut impl Write) -> SearchResult;
}

trait HasTT<T: TT> {
    fn tt(&mut self) -> T;
}

pub struct GameState<T> {
    tt: T,
    board: Board,
    prev_moves: PrevMoves,
    num_threads: usize,
}

impl GameState<TTable> {
    pub fn new() -> GameState<TTable> {
        GameState {
            tt: TTable::new(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
            num_threads: 1,
        }
    }
}

impl GameState<NoTTable> {
    pub fn new_no_tt() -> GameState<NoTTable> {
        GameState {
            tt: NoTTable::default(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
            num_threads: 1,
        }
    }
}

impl GameState<Arc<SmpTTable>> {
    pub fn new_smp(num_threads: usize) -> GameState<Arc<SmpTTable>> {
        GameState {
            tt: SmpTTable::new(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
            num_threads,
        }
    }
}

impl<T> GameState<T> {
    pub fn new_game(&mut self) {
        // self.tt.clear()
    }

    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn set_position(&mut self, board: Board, prev_moves: PrevMoves) {
        self.board = board;
        self.prev_moves = prev_moves;
    }
}

impl CanSearch for GameState<TTable> {
    fn go(&mut self, out: &mut impl Write) -> SearchResult {
        iterative_deepening(&self.board, &mut self.tt, self.prev_moves.clone(), out)
    }
}

impl CanSearch for GameState<NoTTable> {
    fn go(&mut self, out: &mut impl Write) -> SearchResult {
        iterative_deepening(&self.board, self.tt, self.prev_moves.clone(), out)
    }
}

impl CanSearch for GameState<Arc<SmpTTable>> {
    fn go(&mut self, out: &mut impl Write) -> SearchResult {
        lazy_smp(
            &self.board,
            self.tt.clone(),
            self.prev_moves.clone(),
            self.num_threads,
            out,
        )
    }
}
