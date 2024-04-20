use crate::board::Board;
use crate::game_state::GameState;
use crate::moves::PrevMoves;
use crate::perft::HashPerft;
use crate::searcher::{iterative_deepening, lazy_smp};
use crate::uci::Uci;
use http::run_http;
use move_info::{PST, RAYS};
use std::env::args;
use std::time::Instant;
use tt::{NoTTable, SmpTTable, TTable};

use board::Zorb;

use crate::move_info::MT;

pub mod board;
pub mod eval;
pub mod fen;
pub mod game_state;
pub mod hh;
pub mod http;
pub mod magic;
pub mod move_info;
pub mod move_list;
pub mod movegen;
pub mod moves;
pub mod perft;
pub mod searcher;
pub mod tt;
pub mod uci;

pub fn init() {
    Zorb::init();
    RAYS::init();
    PST::init();
    MT::init();
}

#[tokio::main]
async fn main() {
    crate::init();

    if args().count() > 1 {
        do_perftree();
        return;
    }

    match option_env!("MODE").unwrap_or("") {
        "search" => {
            _do_search();
            return;
        }
        "perft" => {
            _do_perft();
            return;
        }
        "http" => {
            run_http().await;
            return;
        }
        _ => {}
    }

    // let mut uci = Uci::new(GameState::new_no_tt());

    // let mut uci = Uci::new(GameState::new_smp(8));

    GameState::new().start();
}

fn _do_perft() {
    let perfts: Vec<(usize, usize, Board)> = vec![
        (
            6,
            8031647685,
            Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
        ),
        (
            8,
            3009794393,
            Board::new_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -").unwrap(),
        ),
        (
            6,
            706045033,
            Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap(),
        ),
        (
            6,
            706045033,
            Board::new_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1")
                .unwrap(),
        ),
        (
            5,
            89941194,
            Board::new_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap(),
        ),
        (
            6,
            6923051137,
            Board::new_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
            .unwrap(),
        ),
        (1, 20, Board::new()),
        (2, 400, Board::new()),
        (3, 8902, Board::new()),
        (4, 197281, Board::new()),
        (5, 4865609, Board::new()),
        (6, 119060324, Board::new()),
        (7, 3195901860, Board::new()),
    ];

    // for (depth, moves, b) in &perfts {
    //     let mut perft = Perft::new();
    //     let start = Instant::now();

    //     perft.perft(b, *depth);
    //     let stop = start.elapsed();
    //     println!(
    //         "Depth: {depth}\t\tMoves (Expected): {} ({})\t\tTime: {}ms",
    //         perft.mc,
    //         moves,
    //         stop.as_millis()
    //     );

    //     // assert_eq!(perft.mc, moves)
    // }

    // println!();

    for (depth, moves, b) in &perfts {
        let mut perft = HashPerft::new();
        let start = Instant::now();

        let mc = perft.perft(b, *depth as u64);
        let stop = start.elapsed();
        println!(
            "Depth: {depth}\t\tMoves (Expected): {} ({})\t\tTime: {}ms",
            mc,
            moves,
            stop.as_millis()
        );

        // let start = Instant::now();

        // let mc = perft.perft(b, *depth as u64);
        // let stop = start.elapsed();
        // println!(
        //     "Depth: {depth}\t\tMoves (Expected): {} ({})\t\tTime: {}ms",
        //     mc,
        //     moves,
        //     stop.as_millis()
        // );

        // assert_eq!(perft.mc, moves)
    }
}

fn _do_search() {
    let num_threads = 1;

    let b = Board::new();
    let prev_moves = PrevMoves::new();

    let start = Instant::now();

    let res = match num_threads {
        0 => iterative_deepening(&b, NoTTable::default(), prev_moves).unwrap(),
        1 => iterative_deepening(&b, &mut TTable::new(), prev_moves).unwrap(),
        t => lazy_smp(&b, SmpTTable::new(), prev_moves, t).unwrap(),
    };

    println!(
        "bestmove: {} with score {}, took {}ms",
        res.1.as_uci_string(),
        res.0,
        start.elapsed().as_millis()
    );
}

fn do_perftree() {
    let args: Vec<String> = args().collect();
    let depth: usize = args[1].parse().unwrap();
    let fen = &args[2];
    let moves: Option<&String> = args.get(3);

    // let perft = Perft::new();
    let mut perft = HashPerft::new();
    perft.perftree_root(depth, fen, moves);
}
