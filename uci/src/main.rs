// use crate::game_state::GameState;
use crate::game_state::game_state_mt::GameStateMT;
use crate::game_state::game_state_st::GameStateST;
use crate::uci::{Uci, UciWriter};
use chess::board::board::Board;
use chess::movegen::moves::{Move, PrevMoves};
use chess::perft::Perft;
use chess::search::search::single_searcher::iterative_deepening;
use chess::search::tt::tt::TTable;
use std::env::args;
use std::fs::File;
use std::time::Instant;

// mod game_state;
mod game_state;
pub mod uci;

const SEARCH: bool = false;
const PERFT: bool = false;

fn main() {
    chess::init();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    // let m = Move::_new_from_u32(14610688);
    // println!("{m}");
    // return;

    if SEARCH {
        _do_search();
        return;
    }

    if PERFT {
        _do_perft();
        return;
    }

    let mut uci = Uci::new(GameStateST::new());
    // let mut uci = Uci::new(GameStateMT::new(4));

    uci.start();
}

fn _do_perft() {
    let b = Board::new();
    // let b = Board::new_fen("r3k2r/p1ppqpb1/1n2pnp1/1b1PN3/1p2P3/2N2Q2/PPPBBPpP/R4R1K w kq - 0 3")
    //     .unwrap();

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

fn _do_search() {
    let b = Board::new();
    let mut tt = TTable::new();
    let mut prev_moves = PrevMoves::new();
    let mut out = UciWriter::new();
    iterative_deepening(&b, &mut tt, &mut prev_moves, &mut out).unwrap();
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    let perft = Perft::new();
    perft.perftree_root(depth, fen, moves);
}
