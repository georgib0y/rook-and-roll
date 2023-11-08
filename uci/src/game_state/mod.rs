pub mod game_state_mt;
pub mod game_state_st;

use chess::board::board::Board;
use chess::movegen::moves::PrevMoves;
use chess::search::searchers::SeachResult;
use std::io;
use std::io::Write;

pub trait UciGameState {
    fn new_game(&mut self);
    fn is_ready(&self, out: &mut impl Write) -> io::Result<()>;
    fn set_position(&mut self, b: Board, prev_move: PrevMoves);
    fn go(&mut self, out: &mut impl Write) -> SeachResult;
}
