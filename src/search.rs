use crate::{Board, is_in_check, is_legal_move, moved_into_check, MoveTables};
use crate::eval::eval;
use crate::movegen::gen_moves;

pub fn iterative_deepening() {

}


pub fn pvs(
    board: &Board,
    depth: usize,
    mut alpha: i32,
    beta: i32,
    p_mul: i32,         // player multiplier - to be passed down to eval
    mt: &MoveTables
) -> i32 {
    if depth == 0 {
        return eval(board, p_mul)
    }

    let mut pv = true;

    let check = is_in_check(board, mt);

    for m in gen_moves(board, mt, check) {
        let b = board.copy_make(&m, mt);
        if (!check && moved_into_check(&b, &m, mt)) || !is_legal_move(&b, &m, mt) { continue; }

        let mut score: i32;

        // TODO could take away this pv if statement and have it above the for to make it a *little* more branchless
        if pv {
            score = -pvs(&b, depth-1, -beta, -alpha, -p_mul, mt);
        } else {
            score = -pvs(&b, depth-1, -alpha-1, -alpha, -p_mul, mt);
            if score > alpha {
                score = -pvs(&b, depth-1, -beta, -alpha, -p_mul, mt);
            }
        }

        if score >= beta { return beta; }
        if score > alpha { alpha = score; }
    }


    alpha
}