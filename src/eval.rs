use crate::Board;

// TODO these values are not final, have been taken directly from rustinator 1
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 400;
const ROOK_VALUE: i32 = 525;
const BISHOP_VALUE: i32 = 350;
const QUEEN_VALUE: i32 = 1000;
const KING_VALUE: i32 = 100000;



pub fn eval(b: &Board, colour_mul: i32) -> i32 {
    material(b) * colour_mul
}

fn material(b: &Board) -> i32 {
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