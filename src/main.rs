// #![allow(unused)]
// extern crate core;

use std::env::args;
use std::fs::{File};
use std::ops::{Range, RangeInclusive};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;

use simplelog::{Config, LevelFilter, WriteLogger};

use crate::board::Board;
use crate::move_tables::MT;
use crate::moves::{Move};
use crate::perft::Perft;
use crate::search::MIN_SCORE;
use crate::tt::{AtomicTTEntry, EntryType, SeqTT, TT, TTable, TTableMT, TTableST, TTEntry};
use crate::uci::{GameState, GameStateST};
use crate::wac_tester::wac_tests;
// use crate::wac_tester::wac_tests;
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

mod wac_tester;
mod fen;

// const LOG_DIR: &str = "/home/george/Documents/progs/rookandroll/logs/last-game.log";
const LOG_DIR: &str = "/home/george/CLionProjects/rustinator-rook_and_roll/logs/last-game.log";


fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    let perft = Perft::new();
    perft.perftree_root(depth, fen, moves);
}

pub fn init() {
    unsafe {
        Zorb::init();
        MT::init();
    }
}

fn main() {
    // let asserts_work_in_release = true;
    // assert_eq!(asserts_work_in_release, false);
    init();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    // _do_perf();
    // _do_search();
    // return;
    // _debug();
    // _do_wac_tests();
    // return;

    // set up logger
    let file = File::options()
        .write(true)
        .truncate(true)
        .open(LOG_DIR)
        .unwrap();

    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        file
    );

    let author = "george";
    let bot_name = "rustinator2";
    let num_threads = 1;

    if num_threads == 1 {
        GameState::<TTableST>::new_single_threaded(author, bot_name).start();
    } else {
        GameState::<TTableMT>::new_multi_threaded(author, bot_name, num_threads).start();
    }
}

fn _debug() {
    // let mut rng = thread_rng();
    // let mut rand = |rng: &mut ThreadRng| (
    //     rng.gen_range(HASH_RANGE),
    //     rng.gen_range(DEPTH_RANGE),
    //     rng.gen_range(SCORE_RANGE),
    //     rng.gen_range(SCORE_RANGE),
    //     rng.gen_range(DEPTH_RANGE)
    // );
    //
    // const HASH_RANGE: Range<u64> = 1..100000;
    // const DEPTH_RANGE: Range<usize> = 0..10;
    // const SCORE_RANGE: RangeInclusive<i32> = MIN_SCORE..=-MIN_SCORE;
    //
    // let num_read_writes= 1000000000usize;
    //
    // let mut seqtt = SeqTT::new();
    // let start = Instant::now();
    // let mut best_score = MIN_SCORE;
    // for _ in 0..num_read_writes {
    //     let (hash, depth, alpha, beta, ply) = rand(&mut rng);
    //     let score = seqtt.get_score(hash, depth, alpha, beta, ply).unwrap_or(MIN_SCORE);
    //     if best_score < score { best_score = score}
    //
    //
    //     let (hash, depth, alpha, _, ply) = rand(&mut rng);
    //     seqtt.insert(hash, alpha, EntryType::PV, depth, None, ply);
    // }
    //
    // println!("best_score {best_score}");
    // let seq_time = start.elapsed().as_millis();
    //
    // let refcelltt = TTableST::new_single_threaded();
    // let start = Instant::now();
    // let mut best_score = MIN_SCORE;
    // for _ in 0..num_read_writes {
    //     let (hash, depth, alpha, beta, ply) = rand(&mut rng);
    //     let score = refcelltt.get_score(hash, depth, alpha, beta, ply).unwrap_or(MIN_SCORE);
    //     if best_score < score { best_score = score}
    //
    //
    //     let (hash, depth, alpha, _, ply) = rand(&mut rng);
    //     refcelltt.insert(hash, alpha, EntryType::PV, depth, None, ply);
    // }
    //
    // println!("best_score {best_score}");
    // let refcell_time = start.elapsed().as_millis();
    //
    //
    // let num_threads = 8;
    //
    // let rwlocktt = TTable::<AtomicTTEntry>::new_multi_threaded();
    // let start = Instant::now();
    //
    // let threads: Vec<_> = (0..num_threads)
    //     .map(|_| Arc::clone(&rwlocktt))
    //     .map(|tt| thread::spawn(move || {
    //         let mut trng = thread_rng();
    //         let mut best_score = MIN_SCORE;
    //         for _ in 0..num_read_writes/num_threads {
    //             let (hash, depth, alpha, beta, ply) = rand(&mut trng);
    //             let score = tt.get_score(hash, depth, alpha, beta, ply).unwrap_or(MIN_SCORE);
    //             if best_score < score { best_score = score}
    //
    //
    //             let (hash, depth, alpha, _, ply) = rand(&mut trng);
    //             tt.insert(hash, alpha, EntryType::PV, depth, None, ply);
    //         }
    //         println!("best_score {best_score}")
    //     }))
    //     .collect();
    //
    // threads.into_iter().for_each(|thread| thread.join().unwrap());
    //
    // let rw_time = start.elapsed().as_millis();
    //
    // let atomictt = TTable::<AtomicTTEntry>::new_multi_threaded();
    // let start = Instant::now();
    //
    // let threads: Vec<_> = (0..num_threads)
    //     .map(|_| Arc::clone(&atomictt))
    //     .map(|tt| thread::spawn(move || {
    //         let mut trng = thread_rng();
    //         let mut best_score = MIN_SCORE;
    //         for _ in 0..num_read_writes/num_threads {
    //             let (hash, depth, alpha, beta, ply) = rand(&mut trng);
    //             let score = tt.get_score(hash, depth, alpha, beta, ply).unwrap_or(MIN_SCORE);
    //             if best_score < score { best_score = score}
    //
    //             let (hash, depth, alpha, _, ply) = rand(&mut trng);
    //             tt.insert(hash, alpha, EntryType::PV, depth, None, ply);
    //         }
    //         println!("best_score {best_score}")
    //     }))
    //     .collect();
    //
    // threads.into_iter().for_each(|thread| thread.join().unwrap());
    //
    // let atomic_time = start.elapsed().as_millis();
    //
    // println!("SeqTT took {}ms to do {num_read_writes} r/w", seq_time);
    // println!("RefCellTT took {}ms to do {num_read_writes} r/w", refcell_time);
    // println!("RwLockTT took {}ms to do {num_read_writes} r/w", rw_time);
    // println!("AtomicTT took {}ms to do {num_read_writes} r/w", atomic_time);
}

// fn _debug() {
//     // let mut b = Board::new();
//     // println!("{b}");
//     // position fen rnbqkb1r/pp3ppp/2p1pn2/3p4/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 1 5 moves e4e5 f6e4 c3e4 d5e4 f3g5 c6c5 f1b5 c8d7 b5d7 b8d7 g5e4 c5d4 d1d4 d8a5 c2c3 d7e5 f2f4 e5c6 d4c4 a5d5 c4e2 f8e7 g2g4 e7h4 e1f1 e8c8 g4g5 h7h6 b2b4 h6g5 f4g5 f7f5 g5f6
//     // position fen rn1qk2r/pp3ppp/2p1pn2/5b2/PbBP4/2N1PN2/1P3PPP/R1BQK2R w KQkq - 1 8 moves d1b3 b4c3 b2c3 d8c7 a4a5 b8d7 e1g1 c6c5 d4d5 e6d5 c4d5 f6d5 b3d5 f5c2 a1a2 d7f6 d5c4 c2e4 c4b5 e8e7 c1a3 b7b6 f3d4 f6g4 g2g3 g4e5 a2d2 h7h6 a5a6 h6h5 f2f4 e5g4 h2h3 g4f6 b5e2 h5h4 g3g4 e4c6 d2d3 f6e4 c3c4 e4g3 d4c6 c7c6 e2f3 c6f3 f1f3 g3e2 g1g2 a8d8 d3d5 e2c3 d5e5 e7f8 f3f2 f7f6 e5e6 f8f7 f4f5 c3d1 f2e2 d8d3 a3c1 d3c3 c1d2 c3c4 e6c6 c4e4 c6c7 e4e7 c7e7 f7e7 d2e1 c5c4 g2f3 h8d8 e1h4 d1c3 e2c2 d8d3 g4g5 f6g5 h4g5 e7f7 h3h4 b6b5 h4h5 b5b4 c2h2 b4b3 f3f4 c3b1 h5h6 g7h6 h2h6 b1c3 h6h7 f7g8 h7a7 b3b2 a7a8 g8f7 a8a7 f7g8 a7a8 g8f7 a8a7 f7f8 a7a8
//     let mut game_state = GameStateSeq::new("", "");
//     // *game_state.board() = Board::new_fen("rn1qk2r/pp3ppp/2p1pn2/5b2/PbBP4/2N1PN2/1P3PPP/R1BQK2R w KQkq - 1 8");
//
//     game_state.position("position fen rnbqkb1r/pp3ppp/2p1pn2/3p4/3PP3/2N2N2/PPP2PPP/R1BQKB1R w KQkq - 1 5 moves e4e5 f6e4 c3e4 d5e4 f3g5 c6c5 f1b5 c8d7 b5d7 b8d7 g5e4 c5d4 d1d4 d8a5 c2c3 d7e5 f2f4 e5c6 d4c4 a5d5 c4e2 f8e7 g2g4 e7h4 e1f1 e8c8 g4g5 h7h6 b2b4 h6g5 f4g5 f7f5 g5f6");
//     // game_state.go("");
//
//     println!("----- POSITION -----\n{}\n----------------\n", game_state.board());
//
//     MoveList::get_moves_unscored(game_state.board(), MoveSet::All).moves.iter()
//         // .filter(|m| m.move_type() == MoveType::Ep)
//         .for_each(|m| {
//             let board = game_state.board().copy_make(*m);
//             println!("{board}{m}\n");
//         })
// }
//
//
fn _do_search() {
    let _pos = "fen r1bq1rk1/ppp1ppbp/2np1np1/8/2P5/3P1NP1/PP2PPBP/RNBQ1RK1 w - - 1 7 moves e2e4 c8g4 b1d2 d8c8 d1c2 e7e5 b2b3 g4h3 g2h3 c8h3 a2a3 f6g4 c1b2 g7h6 b3b4 h6d2 c2d2 a7a6 b4b5 a6b5 c4b5 c6a5 d2e2 c7c6 a3a4 a5b3 a1a3 f7f5 e2c2 f5e4 c2b3 g8g7 b5c6 e4f3 b3b7 g7f6 b2e5 f6f5 b7d7 f5g5 d7e7 g5f5 e7d7 f5g5 d7e7";
    // let fen = "r1bq1rk1/ppp1ppbp/2np1np1/8/2P5/3P1NP1/PP2PPBP/RNBQ1RK1 w - - 1 7";

    // let mut _state_mt = GameStateMT::new("","", 12);
    // *state.board() = Board::new_fen(fen);

    // let mut state = GameState::new("","");
    // *state.board() = Board::new_fen(fen);
    // *state.board() = Board::new();

    // for m_str in "e2e4 c8g4 b1d2 d8c8 d1c2 e7e5 b2b3 g4h3 g2h3 c8h3 a2a3 f6g4 c1b2 g7h6 b3b4 h6d2 c2d2 a7a6 b4b5 a6b5 c4b5 c6a5 d2e2 c7c6 a3a4 a5b3 a1a3 f7f5 e2c2 f5e4 c2b3 g8g7 b5c6 e4f3 b3b7 g7f6 b2e5 f6f5 b7d7 f5g5 d7e7 g5f5 e7d7 f5g5 d7e7".split(" ") {
    //     let m = Move::new_from_text(m_str, state.board());
    //     *state.board() = state.board().copy_make(m);
    //     println!("{m_str}\n{}", state.board());
    // }

    // state.position(pos);
    //
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
    // let mut game_state = GameStateMT::new("","", 8);
    // let mut game_state = GameState::new("","");

    let mut game_state = GameStateST::new_single_threaded("", "");

    let start = Instant::now();
    let best_move = game_state.find_best_move();
    println!("best move: {}\nTook {}ms\n\n",
        best_move.as_uci_string(),
        start.elapsed().as_millis()
    );
}


fn _do_perf() {
    let b = Board::new();
    // let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let depth = 6;
    let start = Instant::now();
    // let mc = perft(&b, depth);

    let mut perft = Perft::new();
    perft.perft(&b, depth);
    // perft.perft_mt_root(b, depth, 12);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {}\t\tTime: {}ms", perft.mc, stop.as_millis());
}

fn _do_wac_tests() {
    wac_tests();
}
