use crate::board::{Board, BLACK, WHITE};
use crate::move_info::PST;
use std::cmp::max;

pub const CHECKMATE: i32 = -1000000000;
pub const MATED: i32 = -CHECKMATE;
pub const STALEMATE: i32 = 0;
pub const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 325;
const ROOK_VALUE: i32 = 500;
const BISHOP_VALUE: i32 = 325;
pub const QUEEN_VALUE: i32 = 1000;
const KING_VALUE: i32 = 20000;

pub const PIECE_VALUES: [i32; 12] = [
    PAWN_VALUE,
    PAWN_VALUE,
    KNIGHT_VALUE,
    KNIGHT_VALUE,
    ROOK_VALUE,
    ROOK_VALUE,
    BISHOP_VALUE,
    BISHOP_VALUE,
    QUEEN_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
    KING_VALUE,
];

pub const MAT_SCORES: [i32; 12] = [
    PAWN_VALUE,
    -PAWN_VALUE,
    KNIGHT_VALUE,
    -KNIGHT_VALUE,
    ROOK_VALUE,
    -ROOK_VALUE,
    BISHOP_VALUE,
    -BISHOP_VALUE,
    QUEEN_VALUE,
    -QUEEN_VALUE,
    KING_VALUE,
    -KING_VALUE,
];

// const BISHOP_PAIR_BONUS: i32 = 30;
// const _ROOK_PAIR_PEN: i32 = -10;
// const _KNIGHT_PAIR_PEN: i32 = -10;
// const NO_PAWNS_PEN: i32 = -200;

const PIECE_PHASE_VAL: [i32; 12] = [0, 0, 1, 1, 1, 1, 2, 2, 4, 4, 0, 0];

// TODO incremental update of game phase
pub fn eval(board: &Board, colour_mul: i32) -> i32 {
    let mg_phase = max(
        24,
        board
            .pieces_iter()
            .enumerate()
            .skip(2)
            .take(8)
            .fold(0, |mg_phase, (p, pieces)| {
                mg_phase + PIECE_PHASE_VAL[p] * pieces.count_ones() as i32
            }),
    );

    let eg_phase = 24 - mg_phase;

    let eval = (board.mg_value() * mg_phase + board.eg_value() * eg_phase) / 24;

    // gen_mat_value(board) * colour_mul
    eval * colour_mul
    // board.mg_value() * colour_mul
}

pub fn gen_board_value(board: &Board) -> (i32, i32) {
    board
        .pieces_iter()
        .copied()
        .enumerate()
        .fold((0, 0), |values, (piece, mut pieces)| {
            let mut mg = 0;
            let mut eg = 0;

            while pieces > 0 {
                let sq = pieces.trailing_zeros() as usize;
                let (p_mg, p_eg) = PST::pst(piece, sq);
                mg += p_mg as i32 + MAT_SCORES[piece];
                eg += p_eg as i32 + MAT_SCORES[piece];
                pieces &= pieces - 1;
            }

            (values.0 + mg, values.1 + eg)
        })

    //
    // let values = |colour| {
    //     board
    //         .pieces_iter().copied()
    //         .enumerate()
    //         .skip(colour)
    //         .step_by(2)
    //         .fold((0, 0), |values, (piece, mut pieces)| {
    //             let mut p_mg = 0;
    //             let mut p_eg = 0;
    //             while pieces > 0 {
    //                 let sq = pieces.trailing_zeros() as usize;
    //                 let (mg, eg) = PST::pst(piece, sq);
    //
    //                 p_mg += mg as i32 + MAT_SCORES[piece];
    //                 p_eg += eg as i32 + MAT_SCORES[piece];
    //                 pieces &= pieces - 1;
    //             }
    //
    //             (values.0 + p_mg, values.1 + p_eg)
    //         })
    // };
    //
    // let (white_mg, white_eg) = values(WHITE);
    // let (black_mg, black_eg) = values(BLACK);
    //
    // (white_mg + black_mg, white_eg + black_eg)
}
/// returns (mg, eg) as values
pub fn gen_pst_value(board: &Board) -> (i32, i32) {
    let values = |colour| {
        board
            .pieces_iter()
            .copied()
            .enumerate()
            .skip(colour)
            .step_by(2)
            .fold((0, 0), |values, (piece, mut pieces)| {
                let mut p_mg = 0;
                let mut p_eg = 0;
                while pieces > 0 {
                    let sq = pieces.trailing_zeros() as usize;
                    let (mg, eg) = PST::pst(piece, sq);
                    p_mg += mg;
                    p_eg += eg;
                    pieces &= pieces - 1;
                }

                (values.0 + p_mg, values.1 + p_eg)
            })
    };

    let (white_mg, white_eg) = values(WHITE);
    let (black_mg, black_eg) = values(BLACK);

    ((white_mg + black_mg) as i32, (white_eg + black_eg) as i32)
}

pub fn gen_mat_value(b: &Board) -> i32 {
    b.pieces_iter()
        .map(|pieces| pieces.count_ones() as i32)
        .enumerate()
        .fold(0, |mat, (piece, count)| mat + MAT_SCORES[piece] * count)

    // let mut mat = 0;
    // for p in 0..12 {
    //     let count = b.pieces(p).count_ones();
    //     mat += MAT_SCORES[p] * count as i32;
    // }
    //
    // mat
}
