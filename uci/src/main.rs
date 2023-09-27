use crate::game_state::GameState;
use chess::board::board::Board;
use chess::perft::Perft;
use chess::search::tt::{AtomicTTable, TTable};
use std::env::args;
use std::sync::Arc;
use std::time::Instant;

mod game_state;
pub mod uci;
mod wac_tester;

const PERFT: bool = true;

fn main() {
    chess::init();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    if PERFT {
        _do_perft();
        return;
    }

    let author = "George";
    let bot_name = "rustinator";
    let num_threads = 1;

    if num_threads == 1 {
        GameState::<TTable>::new_single_thread(author, bot_name).start()
    } else {
        GameState::<Arc<AtomicTTable>>::new_multi_threaded(author, bot_name, num_threads).start()
    };
}

fn _do_perft() {
    let b = Board::new();
    // let b = Board::new_fen("rnbq1bnr/ppppkppp/4pB2/8/8/1P6/P1PPPPPP/RN1QKBNR b KQ - 3 3").unwrap();

    let depth = 6;
    let start = Instant::now();

    let mut perft = Perft::new();

    perft.perft(&b, depth);
    let stop = start.elapsed();
    println!(
        "Depth: {depth}\t\tMoves: {}\t\tTime: {}ms",
        perft.mc,
        stop.as_millis()
    );
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    let perft = Perft::new();
    perft.perftree_root(depth, fen, moves);
}
