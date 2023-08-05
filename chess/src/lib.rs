use move_tables::MT;
use zorbist::Zorb;

pub mod board;
mod next_board_builder;
mod eval;
pub mod fen;
pub mod lazy_smp;
pub mod move_info;
mod move_tables;
mod movegen;
pub mod moves;
mod perft;
pub mod search;
pub mod tt;
mod tt_entry;
mod zorbist;

pub fn init() {
    Zorb::init();
    MT::init();
}

#[cfg(test)]
mod perft_tests {
    use std::time::Instant;

    use board::Board;

    use crate::{board, perft::Perft};

    #[test]
    fn perft() {
        crate::init();

        let b = Board::new();
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
}
