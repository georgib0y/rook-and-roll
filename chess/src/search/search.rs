use crate::board::{Board, BLACK, KING, WHITE};
use crate::movegen::move_list::{ScoredMoveList, MAX_MOVES};
use crate::movegen::movegen::{
    gen_all_attacks, gen_check_moves, gen_moves, is_in_check, is_legal_move, moved_into_check,
};
use crate::movegen::moves::{Move, MoveType};
use crate::search::eval::{eval, CHECKMATE, MATED, PIECE_VALUES, STALEMATE};
use crate::search::searchers::{SeachResult, SearchError, Searcher, MIN_SCORE};
use crate::search::tt::EntryScore;
use crate::search::tt::EntryScore::{Alpha, Beta, PV};

pub fn root_pvs(
    s: &mut impl Searcher,
    b: &Board,
    mut alpha: i32,
    beta: i32,
    depth: usize,
) -> SeachResult {
    s.init_search(b, depth);

    let in_check = is_in_check(b);

    let mut ml: ScoredMoveList<_, MAX_MOVES> = ScoredMoveList::new(b, s, depth);
    gen_moves(b, &mut ml, in_check);

    let mut best_move = None;
    let mut best_score = MIN_SCORE;

    for m in ml {
        let Some(score) = try_move(s, b, m, alpha, beta, depth) else {
            continue;
        };

        if score > alpha {
            best_score = score;
            best_move = Some(m);
            alpha = score;
        }

        if score >= beta {
            beta_cuttoff(s, b, m, beta, depth);
            return Ok((best_score, best_move.ok_or(SearchError::FailHigh)?));
        }

        s.store_tt(b.hash(), Alpha(alpha), best_move);
    }

    s.store_tt(b.hash(), PV(alpha), best_move);
    Ok((best_score, best_move.ok_or(SearchError::FailLow)?))
}

fn pvs(s: &mut impl Searcher, b: &Board, mut alpha: i32, beta: i32, depth: usize) -> i32 {
    if s.has_aborted() {
        return MIN_SCORE;
    }

    // probe tt
    if let Some(score) = s.probe_tt(b.hash(), alpha, beta) {
        return score;
    }

    if depth == 0 {
        let q_score = quiesce(s, b, alpha, beta);
        s.store_tt(b.hash(), PV(q_score), None);
        return q_score;
    }

    let in_check = is_in_check(b);
    let mut best_move = None;
    let mut entry_score = Alpha(alpha);

    let mut ml: ScoredMoveList<_, MAX_MOVES> = ScoredMoveList::new(b, s, depth);
    gen_moves(b, &mut ml, in_check);

    let mut found_pv = false;
    let mut has_moved = false;

    for m in ml {
        let score = if found_pv {
            let Some(mut score) = try_non_pv_move(s, b, m, alpha, depth) else {
                continue;
            };

            if score > alpha && score < beta {
                score = try_move(s, b, m, alpha, beta, depth).unwrap();
            }

            score
        } else {
            let Some(score) = try_move(s, b, m, alpha, beta, depth) else {
                continue;
            };
            has_moved = true;
            score
        };

        if score >= beta {
            beta_cuttoff(s, b, m, beta, depth);
            return beta;
        }

        if score > alpha {
            alpha = score;
            best_move = Some(m);
            entry_score = PV(alpha);
            found_pv = true;
        }
    }

    update_alpha_checkmate_score(s, &mut alpha, &mut entry_score, in_check, has_moved);
    s.store_tt(b.hash(), entry_score, best_move);

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
    s.store_tt(b.hash(), Beta(beta), Some(m));

    if m.move_type() == MoveType::Quiet {
        s.km_store(m, depth);
        s.store_hh_score(b.ctm(), m.from() as usize, m.to() as usize, depth);
    }
}

fn update_alpha_checkmate_score(
    s: &mut impl Searcher,
    alpha: &mut i32,
    entry_score: &mut EntryScore,
    is_in_check: bool,
    has_moved: bool,
) {
    if is_in_check && !has_moved {
        *alpha = CHECKMATE + s.ply();
        *entry_score = PV(*alpha);
    } else if !has_moved {
        *alpha = STALEMATE;
        *entry_score = PV(*alpha);
    }
}

pub fn quiesce(s: &mut impl Searcher, board: &Board, mut alpha: i32, beta: i32) -> i32 {
    let in_check = is_in_check(board);

    let eval = if in_check {
        alpha
    } else {
        let eval = eval(board, s.colour_multiplier());

        if eval >= beta {
            return beta;
        }

        if alpha < eval {
            alpha = eval;
        }

        eval
    };

    let mut ml = ScoredMoveList::<_, MAX_MOVES>::new(board, s, 0);
    if in_check {
        gen_check_moves(board, &mut ml);
    } else {
        gen_all_attacks(board, &mut ml);
    }

    let mut has_moved = false;
    for m in ml {
        if m.xpiece() >= KING as u32 {
            return MATED - s.ply();
        }

        let Some(score) = try_move_quiesce(s, board, m, alpha, beta, eval) else {
            continue;
        };

        has_moved = true;

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }

    if in_check && !has_moved {
        CHECKMATE + s.ply()
    } else {
        alpha
    }
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
