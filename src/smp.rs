use crate::board::Board;
use crate::move_tables::MoveTables;
use crate::moves::{KillerMoves, Move, PrevMoves};
use crate::tt::ParaTT;

const MAX_THREADS: usize = 12;

pub fn lazy_smp(
    board: &Board,
    mt: &MoveTables,
    tt: &ParaTT,
    km: &KillerMoves,
    prev_moves: &PrevMoves
) -> Option<Move> {

}

struct SmpSearcher<'a> {
    mt: &'a MoveTables,
    tt: &'a ParaTT,
    km: KillerMoves,
    prev_moves: PrevMoves
}

impl <'a> SmpSearcher<'a> {
    fn new(
        mt: &'a MoveTables,
        tt: &'a ParaTT,
        km: &KillerMoves,
        prev_moves: &PrevMoves
    ) -> SmpSearcher<'a> {
        SmpSearcher {
            mt, tt, km: km.clone(), prev_moves: prev_moves.clone()
        }
    }
}