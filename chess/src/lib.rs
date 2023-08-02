use move_tables::MT;
use zorbist::Zorb;

mod board;
mod board_builder;
mod eval;
mod fen;
mod lazy_smp;
mod move_info;
mod move_tables;
mod movegen;
mod moves;
mod perft;
mod search;
mod tt;
mod tt_entry;
mod uci;
mod wac_tester;
mod zorbist;

pub fn init() {
    Zorb::init();
    MT::init();
}

#[cfg(test)]
mod chess_tests {
    use std::time::Instant;

    use board::Board;

    use crate::{board, perft::Perft};

    #[test]
    fn perft() {
        crate::init();

        let b = Board::new();
        // let b = Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        let depth = 6;
        let start = Instant::now();
        // let mc = perft(&b, depth);

        let mut perft = Perft::new();
        perft.perft(&b, depth);
        // perft.perft_mt_root(b, depth, 12);
        let stop = start.elapsed();
        println!(
            "Depth: {depth}\t\tMoves: {}\t\tTime: {}ms",
            perft.mc,
            stop.as_millis()
        );
    }
}
