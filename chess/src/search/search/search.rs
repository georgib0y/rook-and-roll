use crate::board::board::{Board, BLACK, KING, WHITE};
use crate::movegen::move_list::{ScoredMoveList, StackMoveList, MAX_MOVES};
use crate::movegen::movegen::{is_in_check, is_legal_move, moved_into_check};
use crate::movegen::moves::{Move, MoveType};
use crate::movegen::piece_gen::{gen_all_attacks, gen_moves};
use crate::search::eval::{eval, MATED, PIECE_VALUES};
use crate::search::search::{SeachResult, SearchError, Searcher, MIN_SCORE};
use crate::search::tt::EntryType::{Alpha, Beta, PV};

pub fn root_pvs(
    s: &mut impl Searcher,
    b: &Board,
    alpha: i32,
    beta: i32,
    depth: usize,
) -> SeachResult {
    s.init_search(b);

    let in_check = is_in_check(b);

    let mut ml: ScoredMoveList<_, MAX_MOVES> = ScoredMoveList::new(b, s, depth);
    gen_moves(b, &mut ml, in_check);

    let mut best_move = None;
    let mut best_score = MIN_SCORE;

    for m in ml {
        let Some(score) = try_move(s, b, m, alpha, beta, depth) else {
            continue;
        };

        if score > best_score {
            best_score = score;
            best_move = Some(m)
        }
    }

    if best_score == beta {
        Err(SearchError::FailHigh)?
    }

    Ok((best_score, best_move.ok_or(SearchError::FailLow)?))
}

// TODO negamax for now, but switch to pvs when move ordering is done
fn pvs(s: &mut impl Searcher, b: &Board, mut alpha: i32, beta: i32, depth: usize) -> i32 {
    if s.has_aborted() {
        return MIN_SCORE;
    }

    // probe tt
    if let Some(score) = s.probe_tt(b.hash(), alpha, beta, depth) {
        return score;
    }

    if depth == 0 {
        let q_score = quiesce(s, b, alpha, beta);
        s.store_tt(b.hash(), q_score, PV, depth, None);
        return q_score;
    }

    let in_check = is_in_check(b);
    let mut best_move = None;
    let mut entry_type = Alpha;

    let mut ml: ScoredMoveList<_, MAX_MOVES> = ScoredMoveList::new(b, s, depth);
    gen_moves(b, &mut ml, in_check);

    for m in ml {
        let Some(score) = try_move(s, b, m, alpha, beta, depth) else {
            continue;
        };

        if score >= beta {
            beta_cuttoff(s, b, m, beta, depth);
            return beta;
        }

        if score > alpha {
            entry_type = PV;
            best_move = Some(m);
            alpha = score;
        }
    }

    s.store_tt(b.hash(), alpha, entry_type, depth, best_move);

    alpha
}

fn try_move(
    s: &mut impl Searcher,
    board: &Board,
    m: Move,
    alpha: i32,
    beta: i32,
    depth: usize,
) -> Option<i32> {
    let b = board.copy_make(m);

    if !is_legal_move(&b, m, s.prev_moves()) || moved_into_check(&b, m) {
        return None;
    }

    s.push_ply();
    s.push_prev_move(b.hash());

    let score = -pvs(s, &b, -beta, -alpha, depth - 1);

    s.pop_ply();
    s.pop_prev_move(b.hash());

    Some(score)
}

fn try_non_pv_move(
    s: &mut impl Searcher,
    board: &Board,
    m: Move,
    alpha: i32,
    depth: usize,
) -> Option<i32> {
    let b = board.copy_make(m);

    if !is_legal_move(&b, m, s.prev_moves()) || moved_into_check(&b, m) {
        return None;
    }

    s.push_ply();
    s.push_prev_move(b.hash());

    let score = -pvs(s, &b, -alpha - 1, -alpha, depth - 1);

    s.pop_ply();
    s.pop_prev_move(b.hash());

    Some(score)
}

fn beta_cuttoff(s: &mut impl Searcher, b: &Board, m: Move, beta: i32, depth: usize) {
    s.store_tt(b.hash(), beta, Beta, depth, Some(m));

    if m.move_type() == MoveType::Quiet {
        s.km_store(m, depth);
        s.store_hh_score(b.ctm(), m.from() as usize, m.to() as usize, depth);
    }
}

pub fn quiesce(s: &mut impl Searcher, board: &Board, mut alpha: i32, beta: i32) -> i32 {
    if is_in_check(board) {
        return pvs(s, board, alpha, beta, 1);
    }

    let eval = eval(board, s.colour_multiplier());

    if eval >= beta {
        return beta;
    }

    if alpha < eval {
        alpha = eval;
    }

    let mut ml = StackMoveList::default();
    gen_all_attacks(board, &mut ml);

    for m in ml {
        if m.xpiece() >= KING as u32 {
            return MATED - s.ply();
        }

        let Some(score) = try_move_quiesce(s, board, m, alpha, beta, eval) else {
            continue;
        };

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn try_move_quiesce(
    s: &mut impl Searcher,
    board: &Board,
    m: Move,
    alpha: i32,
    beta: i32,
    eval: i32,
) -> Option<i32> {
    let b = board.copy_make(m);

    if moved_into_check(&b, m) || delta_prune(&b, alpha, eval, m) {
        return None;
    }

    s.push_ply();
    let score = -quiesce(s, &b, -beta, -alpha);
    s.pop_ply();

    Some(score)
}

fn delta_prune(b: &Board, alpha: i32, eval: i32, m: Move) -> bool {
    eval + PIECE_VALUES[m.xpiece() as usize] + 200 < alpha
        && !m.move_type().is_promo()
        && (b.all_occ() ^ b.pawns(WHITE) ^ b.pawns(BLACK)).count_ones() > 4
}
