#![allow(unused)]
use std::env;
use std::env::args;
use std::fs::{File, remove_file};
use std::os::unix::fs;
use std::ptr::null;
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use rand::prelude::*;
use rand_chacha::rand_core::impls::fill_bytes_via_next;
use rand_distr::Normal;
use simplelog::{Config, LevelFilter, WriteLogger};
use threadpool::ThreadPool;

use crate::board::{print_bb, Board};
use crate::move_info::BISHOP_MASK;
use crate::move_tables::{find_magic, print_new_magics, ratt, MoveTables, B_BIT, R_BIT};
use crate::movegen::MoveList;
// use crate::movegen::{
//     gen_all_moves, gen_check_moves, gen_moves, is_in_check, is_legal_move, moved_into_check,
//     sq_attacked,
// };
use crate::moves::Move;
use crate::perft::{Perft};
use crate::search::Search;
use crate::tt::{AtomicTT, SeqTT};
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


fn main() {
    Zorb::init();
    let mt = MoveTables::new();
    let perft = Perft::new();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    // debug();
    do_perf();
    // do_search();
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
    let mut b = Board::new_fen("6r1/p4k2/8/1p1N4/2pKP2P/5R1N/6P1/R1B5 b - - 0 33");


    let ml = MoveList::all(&b, &mt, true);
    ml.moves.iter().for_each(|m| println!("{m}"));
}

fn do_search() {
    let board = Board::new();
    // let board = Board::new_fen("");
    let mut tt = SeqTT::new();
    let mt = MoveTables::new();

    let mut search = Search::new(&mt, &mut tt);
    let best_move = search.iterative_deepening(&board);
    println!("best move: {}", best_move.unwrap().as_uci_string());
}

fn do_perf() {
    // let b = Board::new();
    let b = Board::new_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let depth = 6;
    let start = Instant::now();
    // let mc = perft(&b, depth);
    let mt = MoveTables::new();
    let mut perft = Perft::new();
    perft.perft_new_movegen(&b, depth);
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
