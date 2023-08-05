use crate::game_state::GameState;
use chess::tt::{AtomicTTable, TTable};
use std::sync::Arc;

mod game_state;
pub mod uci;
mod wac_tester;

fn main() {
    chess::init();

    let author = "George";
    let bot_name = "rustinator";
    let num_threads = 1;

    if num_threads == 1 {
        GameState::<TTable>::new_single_thread(author, bot_name).start()
    } else {
        GameState::<Arc<AtomicTTable>>::new_multi_threaded(author, bot_name, num_threads).start()
    };
}
