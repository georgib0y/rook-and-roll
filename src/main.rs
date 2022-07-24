#![allow(unused)]

use std::env;
use std::env::args;
use std::sync::Arc;
use std::time::Instant;

use crate::board::{Board, print_bb};
use crate::move_info::BISHOP_MASK;
use crate::move_tables::{B_BIT, find_magic, MoveTables, print_new_magics, R_BIT, ratt};
use crate::movegen::{gen_all_moves, gen_check_moves, is_in_check, moved_into_check, sq_attacked};
use crate::perft::{Counter, perft_debug, perftree_root, perft, perft_mt_root};

mod board;
mod move_tables;
mod moves;
mod movegen;
mod perft;
mod move_info;

fn main() {
    if args().count() > 1 {
        do_perftree();
    }


    do_perf();
    // debug();
    // do_perftree();

    // print_bb(8796093022208);
}

fn debug() {
    let b = Board::new_fen("rnbqkbnr/1p1ppppp/8/pPp5/8/8/P1PPPPPP/RNBQKBNR w KQkq c6 0 1");
    let mt = MoveTables::new_boxed();
    let all = gen_all_moves(&b, &mt);
    // let check = gen_check_moves(&b, &mt);

    dbg!(all.len());//, check.len());
    all.for_each(|m| println!("{m}"))
    // dbg!(is_in_check(&b, &mt));
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    perftree_root(depth, fen, moves);
}

fn do_perf() {
    let b = Board::new();
    let mt = MoveTables::new_boxed();
    let depth = 6;
    let start = Instant::now();
    let mc = perft(&b, depth, &mt);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {mc}\t\tTime: {}ms", stop.as_millis());
}

fn do_perf_mt() {
    let b = Board::new();
    let mt = MoveTables::new_arc();
    let depth = 6;
    let start = Instant::now();
    let mc = perft_mt_root(Arc::new(b), depth, mt, 12);
    let stop = start.elapsed();
    println!("Depth: {depth}\t\tMoves: {mc}\t\tTime: {}ms", stop.as_millis());
}

fn do_perf_counter() {
    let depth = 4;
    let b = Board::new();
    let mt = MoveTables::new_boxed();
    let mut counter = Counter::new();
    // println!("{}", movegen::gen_all_moves(&b, &mt).len());
    perft_debug(&b, depth, &mt, None, &mut counter);
    dbg!(counter);
}