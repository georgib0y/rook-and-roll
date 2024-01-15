use std::sync::Arc;

use crate::{
    board::Board,
    moves::PrevMoves,
    searcher::{iterative_deepening, lazy_smp, SearchResult},
    tt::{NoTTable, SmpTTable, TTable, TT},
};

pub trait CanSearch {
    fn new_game(&mut self);
    fn go(&mut self) -> SearchResult;
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

impl Default for GameState<TTable> {
    fn default() -> Self {
        Self::new()
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
    pub fn is_ready(&self) -> bool {
        true
    }

    pub fn set_position(&mut self, board: Board, prev_moves: PrevMoves) {
        self.board = board;
        self.prev_moves = prev_moves;
    }
}

impl CanSearch for GameState<TTable> {
    fn new_game(&mut self) {
        (&mut self.tt).clear()
    }

    fn go(&mut self) -> SearchResult {
        iterative_deepening(&self.board, &mut self.tt, self.prev_moves.clone())
    }
}

impl CanSearch for GameState<NoTTable> {
    fn new_game(&mut self) {
        self.tt.clear()
    }

    fn go(&mut self) -> SearchResult {
        iterative_deepening(&self.board, self.tt, self.prev_moves.clone())
    }
}

impl CanSearch for GameState<Arc<SmpTTable>> {
    fn new_game(&mut self) {
        self.tt.clear()
    }

    fn go(&mut self) -> SearchResult {
        lazy_smp(
            &self.board,
            self.tt.clone(),
            self.prev_moves.clone(),
            self.num_threads,
        )
    }
}
