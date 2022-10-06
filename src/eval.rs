use crate::move_info::PST;
use crate::Board;

pub const CHECKMATE: i32 = (i16::MIN as i32) / 2;

// TODO these values are not final, have been taken directly from rustinator 1
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 320;
const ROOK_VALUE: i32 = 500;
const BISHOP_VALUE: i32 = 350;
const QUEEN_VALUE: i32 = 900;
const KING_VALUE: i32 = 20000;

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

//TODO https://www.chessprogramming.org/Score#Grain  could consider this at the end of eval func?

pub fn eval(b: &Board, colour_mul: i32) -> i32 {
    let mut eval = 0;
    eval += b.mat_value;
    eval += eval_pst(b);

    eval * colour_mul
}

pub fn eval_pst(b: &Board) -> i32 {
    let mut pos = 0;

    for piece in (0..=10).step_by(2) {
        let mut w_piece = b.pieces[piece];
        while w_piece > 0 {
            let sq = w_piece.trailing_zeros() as usize;
            pos += PST[piece][sq] as i32;
            w_piece &= w_piece - 1;
        }

        let mut b_piece = b.pieces[piece + 1];
        while b_piece > 0 {
            let sq = b_piece.trailing_zeros() as usize;
            pos -= PST[piece + 1][sq] as i32;
            b_piece &= b_piece - 1;
        }
    }

    pos
}

pub fn gen_mat_value(b: Board) -> i32 {
    let mut mat = b.pieces[0].count_ones() as i32 * PAWN_VALUE;
    mat -= b.pieces[1].count_ones() as i32 * PAWN_VALUE;

    mat += b.pieces[2].count_ones() as i32 * KNIGHT_VALUE;
    mat -= b.pieces[3].count_ones() as i32 * KNIGHT_VALUE;

    mat += b.pieces[4].count_ones() as i32 * ROOK_VALUE;
    mat -= b.pieces[5].count_ones() as i32 * ROOK_VALUE;

    mat += b.pieces[6].count_ones() as i32 * BISHOP_VALUE;
    mat -= b.pieces[7].count_ones() as i32 * BISHOP_VALUE;

    mat += b.pieces[8].count_ones() as i32 * QUEEN_VALUE;
    mat -= b.pieces[9].count_ones() as i32 * QUEEN_VALUE;

    mat
}
