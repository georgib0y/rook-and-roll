#![allow(unused)]
extern crate core;

use std::env;
use std::env::args;
use std::fs::{File, remove_file};
use std::os::unix::fs;
use std::ptr::null;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use rand::prelude::*;
use rand_chacha::rand_core::impls::fill_bytes_via_next;
use simplelog::{Config, LevelFilter, WriteLogger};

use crate::board::{print_bb, Board};
use crate::move_tables::{find_magic, print_new_magics, ratt, MoveTables, B_BIT, R_BIT};
use crate::movegen::MoveList;
// use crate::movegen::{
//     gen_all_moves, gen_check_moves, gen_moves, is_in_check, is_legal_move, moved_into_check,
//     sq_attacked,
// };
use crate::moves::{KillerMoves, Move, PrevMoves};
use crate::perft::{Perft};
use crate::search::Search;
use crate::tt::{ParaTT, SeqTT};
use crate::uci::Uci;
use crate::zorbist::{Zorb};

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


fn main() {
    Zorb::init();
    let mt = MoveTables::new();
    let perft = Perft::new();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    // debug();
    // do_perf();
    do_search();
    return;

    // set up logger
    let date_time = chrono::Local::now().format("%d%m%H%M%S").to_string();
    let mut filename = format!("/home/george/Documents/progs/rookandroll/logs/log-{date_time}.log");
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(filename.clone()).unwrap(),
    );

    remove_file("/home/george/Documents/progs/rookandroll/logs/last-game.log").unwrap();
    fs::symlink(filename, "/home/george/Documents/progs/rookandroll/logs/last-game.log").unwrap();

    Uci::new("george", "rustinator2").start();
}

fn debug() {
    let mt = MoveTables::new();
    // let mut b = Board::new();
    // println!("{b}");
    // position fen r1bqkbnr/pp1ppppp/2n5/2p5/2P1P3/8/PP1P1PPP/RNBQKBNR w KQkq - 1 3 moves f2f3 e7e5 g1e2 d7d6 b1c3 f8e7 d2d3 g8f6 d1a4 c8d7 c1e3 c6d4 e1c1 d4e2 f1e2 d7a4 c3a4 f6d7 e3g5 e7g5 c1b1 g5e3 f3f4
    let mut b = Board::new_fen("r1bqkbnr/pp1ppppp/2n5/2p5/2P1P3/8/PP1P1PPP/RNBQKBNR w KQkq - 1 3");
    println!("{b}");

    for m_str in "f2f3 e7e5 g1e2 d7d6 b1c3 f8e7 d2d3 g8f6 d1a4 c8d7 c1e3 c6d4 e1c1 d4e2 f1e2 d7a4 c3a4 f6d7 e3g5 e7g5 c1b1 g5e3 f3f4".split(" ") {
        let m = Move::new_from_text(m_str, &b);
        b = b.copy_make(m);
        println!("{b}");
    }

    let searched: Vec<u32> = vec![0x91d102, 0x1c4400, 0x755100, 0x622212, 0x54e102, 0x8b1212, 0x386127];//, 0xc6b212];//, 0xf34b00, 0xaf5212, 0xd35b22, 0x1064c2];
    searched.iter().map(|s| Move::new_from_u32(*s)).for_each(|m| {
        println!("{m}");
        b = b.copy_make(m);
        println!("{b}");
    });




    let ml = MoveList::all(&b, &mt, false);
    ml.moves.iter().for_each(|m| println!("{m}"));
}

fn do_search() {
    let board = Board::new();
    // let board = Board::new_fen("");
    let mut tt = SeqTT::new();
    let mt = MoveTables::new();
    let mut km = KillerMoves::new();
    let mut prev_moves = PrevMoves::new();

    let mut search = Search::new(&mt, &mut tt, &mut km, &mut prev_moves);
    let best_move = search.iterative_deepening(&board);
    println!("best move: {}", best_move.unwrap().as_uci_string());
}

fn do_perf() {
    let b = Board::new();
    // let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let depth = 6;
    let start = Instant::now();
    // let mc = perft(&b, depth);
    let mt = MoveTables::new();
    let mut perft = Perft::new();
    perft.perft(&b, depth);
    // perft.perft_mt_root(b, depth, 12);
    let stop = start.elapsed();
    println!(
        "Depth: {depth}\t\tMoves: {}\t\tTime: {}ms", perft.mc,
        stop.as_millis()
    );
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);
    let mt = MoveTables::new();

    let perft = Perft::new();
    perft.perftree_root(depth, fen, moves);
}

// fn do_perf_mt() {
//     let b = Board::new();
//     let mt = MoveTables::new_arc();
//     let depth = 6;
//     let start = Instant::now();
//     // let mc = perft_mt_root(Arc::new(b), depth, 12);
//     let stop = start.elapsed();
//     println!(
//         "Depth: {depth}\t\tMoves: {mc}\t\tTime: {}ms",
//         stop.as_millis()
//     );
// }
