use crate::movegen::move_info::MT;

pub mod board;
pub mod movegen;
pub mod perft;
pub mod search;

pub fn init() {
    MT::init();
}

#[cfg(test)]
mod perft_tests {
    use crate::board::Board;
    use crate::perft::Perft;
    use std::time::Instant;

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
