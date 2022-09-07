#![allow(unused)]
extern crate core;

use std::env;
use std::env::args;
use std::ptr::null;
use std::sync::Arc;
use std::time::Instant;

use rand::prelude::*;
use rand_distr::Normal;
use threadpool::ThreadPool;


use crate::board::{Board, print_bb};
use crate::EntryType::Set;
use crate::move_info::BISHOP_MASK;
use crate::move_tables::{B_BIT, find_magic, MoveTables, print_new_magics, R_BIT, ratt};
use crate::movegen::{gen_all_moves, gen_check_moves, is_in_check, is_legal_move, moved_into_check, sq_attacked};
use crate::moves::Move;
use crate::perft::{perftree_root, perft, perft_mt_root};
use crate::search::iterative_deepening;
use crate::tt::{EntryType, SeqTT, AtomicTT, UNSET_TT_FLAG, SET_TT_FLAG};

mod board;
mod move_tables;
mod moves;
mod movegen;
mod perft;
mod move_info;
mod opening_book;
mod search;
mod eval;
mod uci;
mod tt;


fn main() {
    if args().count() > 1 {
        do_perftree();
        return;
    }

    // do_search();
    // do_perf();
    do_perf_mt();
    // debug();
    // do_perftree();

    // print_bb(8796093022208);
}

fn debug() {
    let mt = MoveTables::new();
    let mut b = Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    println!("{b}");
    let extra_moves = String::from("a2a4 a6b5 a4b5 c7c5");
    for m_str in extra_moves.split(' ') {
        let m = Move::new_from_text(m_str, &b);
        println!("{m}");
        b = b.copy_make(&m);
    }
    println!("{b}");
    let moves = gen_all_moves(&b);
    // let moves = gen_check_moves(&b, &mt);
    moves.iter()
        .filter(|m| is_legal_move(&b,m))
        .for_each(|m| println!("{}\n{}", m, moved_into_check(&b, m)));

    dbg!(moves.len());
    // dbg!(moves[0]);
    // dbg!(moved_into_check(&b, &moves[0], &mt));
    // dbg!(all.len());//, check.len());
    // dbg!(is_in_check(&b, &mt));
}

fn do_search() {
    let board = Board::new();
    // let board = Board::new_fen("");
    let best_move = iterative_deepening(&board);
    println!("best move: {}", best_move.unwrap().as_uci_string());
}

fn do_perf() {
    let b = Board::new();
    let mt = MoveTables::new();
    // let b = Board::new_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", &mt);
    let depth = 6;
    let start = Instant::now();
    let mc = perft(&b, depth);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {mc}\t\tTime: {}ms", stop.as_millis());
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    perftree_root(depth, fen, moves);
}

fn do_perf_mt() {
    let b = Board::new();
    let mt = MoveTables::new_arc();
    let depth = 6;
    let start = Instant::now();
    let mc = perft_mt_root(Arc::new(b), depth, 12);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {mc}\t\tTime: {}ms", stop.as_millis());
}
