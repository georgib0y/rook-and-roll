// #![allow(unused)]
// extern crate core;

use std::env::args;
use std::fs::{File, remove_file};
use std::time::Instant;

use simplelog::{Config, LevelFilter, WriteLogger};

use crate::board::Board;
use crate::move_tables::MT;
use crate::movegen::MoveSet;
use crate::moves::Move;
use crate::perft::Perft;
use crate::tt::SeqTT;
use crate::uci::{GameState, GameStateMT, Uci};
use crate::zorbist::Zorb;

mod board;
mod eval;
mod move_info;
mod move_tables;
mod moves;
mod opening_book;
mod perft;
mod search;
mod tt;
mod uci;
mod zorbist;
mod movegen;
mod smp;
mod move_scorer;

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    let perft = Perft::new();
    perft.perftree_root(depth, fen, moves);
}

fn init() {
    Zorb::init();
    MT::init();
}

fn main() {
    init();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    do_perf();
    // do_search();
    // debug();
    return;

    // set up logger
    let date_time = chrono::Local::now().format("%d%m%H%M%S").to_string();
    let filename = format!("/home/george/Documents/progs/rookandroll/logs/log-{date_time}.log");
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(filename.clone()).unwrap(),
    );

    remove_file("/home/george/Documents/progs/rookandroll/logs/last-game.log").unwrap();
    std::os::unix::fs::symlink(filename, "/home/george/Documents/progs/rookandroll/logs/last-game.log").unwrap();

    let author = "george";
    let bot_name = "rustinator2";
    let num_threads = 8;

    if num_threads > 1 {
        GameStateMT::new(author, bot_name, num_threads).start();
    } else {
        GameState::new(author, bot_name).start();
    }
}

fn debug() {
    // let mut b = Board::new();
    // println!("{b}");
    //position fen rnbqk2r/pp2bppp/2p2n2/3p2B1/3P4/2N2N2/PPQ1PPPP/R3KB1R b KQkq - 3 7 moves g7g6 g5f6 e7f6 e2e4 e8g8 e4e5 f6g7 f1d3 c8e6 c2b3 b7b5 e1g1 a7a5 c3e2 f7f6 b3c2 f6e5 f3e5 g7e5 d4e5 d8c7 d3g6 a8a7 f2f4 c7b6 g1h1 c6c5 g6h5 b8c6
    let mut b = Board::new_fen("rnbqk2r/pp2bppp/2p2n2/3p2B1/3P4/2N2N2/PPQ1PPPP/R3KB1R b KQkq - 3 7");
    println!("{b}");

    for m_str in "g7g6 g5f6 e7f6 e2e4 e8g8 e4e5 f6g7 f1d3 c8e6 c2b3 b7b5 e1g1 a7a5 c3e2 f7f6 b3c2 f6e5 f3e5 g7e5 d4e5 d8c7 d3g6 a8a7 f2f4 c7b6 g1h1 c6c5 g6h5 b8c6 e6d6 a6a7 d6e7 a7a8q e7e6 a1b1 e6e7 b1a1 e7e6 a1b1 e6e7".split(" ") {
        let m = Move::new_from_text(m_str, &b);
        b = b.copy_make(m);
        println!("{m_str}\n{b}");
    }
    // for m_hex in [0x3b9ac9f8] {
    //     let m = Move::new_from_u32(m_hex);
    //     b = b.copy_make(m);
    //     println!("{}\n{b}", m.as_uci_string());
    // }

    // let searched: Vec<u32> = vec![0x91d102, 0x1c4400, 0x755100, 0x622212, 0x54e102, 0x8b1212, 0x386127];//, 0xc6b212];//, 0xf34b00, 0xaf5212, 0xd35b22, 0x1064c2];
    // searched.iter().map(|s| Move::new_from_u32(*s)).for_each(|m| {
    //     println!("{m}");
    //     b = b.copy_make(m);
    //     println!("{b}");
    // });

    let ml = movegen::MoveList::get_moves(&b, MoveSet::All, None);
    ml.moves.iter().for_each(|m| println!("{m}"));
}


fn do_search() {
    // position fen r1bqkb1r/pp2pp1p/2p3p1/2p1P3/6n1/2NP1N2/PPP2PPP/R1BQK2R b KQkq - 0 7 moves f8g7 d1e2 d8c7 c1f4 b7b5 h2h3 b5b4 c3a4 c7a5 b2b3 g4h6 e2e4 a5b5 e4c4 c8e6 c4b5 c6b5 a4c5 a8c8 c5a6 c8c2 f3d4 c2c3 d4b5 c3d3 a6c5 d3d5 b5c7 e8f8 c5e6 f7e6 c7d5 e6d5 a1c1 f8f7 c1c7 h8a8 e1g1 h6f5 f1d1 g6g5 f4g5 g7e5 c7b7 a8g8 d1d5 e5f6 d5f5 g8g5 f5g5 f6g5 b7b4 a7a5 b4c4 g5d2 a2a3 h7h6 g1f1 e7e5 c4c2 d2g5 c2c5 f7e6 c5a5 g5d2 a5a7 e6d6 f1g1 d6c5 a7a6 d2g5 b3b4 c5c4 g1h1 g5d2 a6d6 d2e1 d6e6 c4d5 e6f6 d5c4 f2f4 e5e4 f6e6 c4d4 f4f5 d4d5 b4b5 e1g3 b5b6 h6h5 b6b7 g3b8 e6e8 b8d6 e8d8 d5c6 d8d6 c6d6 b7b8q d6e7 b8e5 e7f7 e5e4 f7f6 g2g3 f6f7 g3g4 h5g4 h3g4 f7f6 a3a4 f6g5 a4a5 g5f6 g4g5 f6g5 f5f6 g5f6 a5a6 f6f7 e4b1 f7e6  e6e7
    let fen = "rn1qkb1r/1bpp1ppp/p3pn2/8/1pP5/5NP1/PPNPPPBP/R1BQK2R w KQkq - 0 7";

    // let mut state_mt = GameStateMT::new("","", 8);
    // *state_mt.board() = Board::new_fen(fen);

    let mut state = GameState::new("","");
    // *state.board() = Board::new_fen(fen);
    *state.board() = Board::new();

    // for m_str in "e1g1 d7d5 c4d5 e6d5 d2d4 f8d6 f3e5 b8c6 f2f4 d8e7 c2e3 e8f8 g2d5 f6d5 e3d5 e7e6 e2e4 f8g8 a2a3".split(" ") {
    //     let m = Move::new_from_text(m_str, state.board());
    //     *state.board() = state.board().copy_make(m);
    //     println!("{m_str}\n{}", state.board());
    // }

    // let start = Instant::now();
    // let best_move = state.find_best_move().unwrap();
    // println!("single threaded best move: {}\nTook {}ms\n\n",
    //     best_move.as_uci_string(),
    //     start.elapsed().as_millis()
    // );


    //
    // for m_str in "f8g7 d1e2 d8c7 c1f4 b7b5 h2h3 b5b4 c3a4 c7a5 b2b3 g4h6 e2e4 a5b5 e4c4 c8e6 c4b5 c6b5 a4c5 a8c8 c5a6 c8c2 f3d4 c2c3 d4b5 c3d3 a6c5 d3d5 b5c7 e8f8 c5e6 f7e6 c7d5 e6d5 a1c1 f8f7 c1c7 h8a8 e1g1 h6f5 f1d1 g6g5 f4g5 g7e5 c7b7 a8g8 d1d5 e5f6 d5f5 g8g5 f5g5 f6g5 b7b4 a7a5 b4c4 g5d2 a2a3 h7h6 g1f1 e7e5 c4c2 d2g5 c2c5 f7e6 c5a5 g5d2 a5a7 e6d6 f1g1 d6c5 a7a6 d2g5 b3b4 c5c4 g1h1 g5d2 a6d6 d2e1 d6e6 c4d5 e6f6 d5c4 f2f4 e5e4 f6e6 c4d4 f4f5 d4d5 b4b5 e1g3 b5b6 h6h5 b6b7 g3b8 e6e8 b8d6 e8d8 d5c6 d8d6 c6d6 b7b8q d6e7 b8e5 e7f7 e5e4 f7f6 g2g3 f6f7 g3g4 h5g4 h3g4 f7f6 a3a4 f6g5 a4a5 g5f6 g4g5 f6g5 f5f6 g5f6 a5a6 f6f7 e4b1 f7e6".split(" ") {
    //     let m = Move::new_from_text(m_str, state_mt.board());
    //     *state_mt.board() = state_mt.board().copy_make(m);
    //     println!("{m_str}\n{}", state_mt.board());
    // }
    //
    let start = Instant::now();
    let best_move = state.find_best_move().unwrap();
    println!("best move: {}\nTook {}ms\n\n",
        best_move.as_uci_string(),
        start.elapsed().as_millis()
    );
}


fn do_perf() {
    let b = Board::new();
    // let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let depth = 6;
    let start = std::time::Instant::now();
    // let mc = perft(&b, depth);

    let mut perft = Perft::new();
    perft.perft(&b, depth);
    // perft.perft_mt_root(b, depth, 12);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {}\t\tTime: {}ms", perft.mc, stop.as_millis());
}
