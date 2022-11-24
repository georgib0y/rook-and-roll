use crate::move_info::PST;
use crate::Board;
use crate::board::KING;

pub const CHECKMATE: i32 = -1000000000;
pub const MATED: i32 = -CHECKMATE;

pub const STALEMATE: i32 = 0;

// TODO these values are not final, have been taken directly from rustinator 1
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 325;
const ROOK_VALUE: i32 = 500;
const BISHOP_VALUE: i32 = 325;
pub const QUEEN_VALUE: i32 = 1000;
const KING_VALUE: i32 = 20000;

pub const PIECE_VALUES: [i32;12] = [
    PAWN_VALUE, PAWN_VALUE,
    KNIGHT_VALUE, KNIGHT_VALUE,
    ROOK_VALUE, ROOK_VALUE,
    BISHOP_VALUE, BISHOP_VALUE,
    QUEEN_VALUE, QUEEN_VALUE,
    KING_VALUE, KING_VALUE
];

pub const MAT_SCORES: [i32; 12] = [
    PAWN_VALUE, -PAWN_VALUE,
    KNIGHT_VALUE, -KNIGHT_VALUE,
    ROOK_VALUE, -ROOK_VALUE,
    BISHOP_VALUE, -BISHOP_VALUE,
    QUEEN_VALUE, -QUEEN_VALUE,
    KING_VALUE, -KING_VALUE,
];


const _BISHOP_PAIR_BONUS: i32 = 30;
const _ROOK_PAIR_PEN: i32 = -10;
const _KNIGHT_PAIR_PEN: i32 = -10;
const _NO_PAWNS_PEN: i32 = -200;


pub fn eval(board: &Board, colour_mul: i32) -> i32 {
    let mut eval = 0;
    eval += board.value;

    if board.is_endgame() {
        // adjust king pst to endgame pst
        let ksq = board.pieces[KING+board.colour_to_move].trailing_zeros() as usize;
        eval += (-PST[KING+board.colour_to_move][ksq] + -PST[KING+board.colour_to_move+2][ksq]) as i32
    }

    // // bishop pair bonus
    // if board.pieces[BISHOP + board.colour_to_move].count_ones() == 2 {
    //     eval += BISHOP_PAIR_BONUS;
    // }
    // // rook pair pen (redundancy)
    // if board.pieces[ROOK + board.colour_to_move].count_ones() == 2 {
    //     eval += ROOK_PAIR_PEN;
    // }
    // // knight pair pen (worse than other minor piece pairs)
    // // TODO dont know if this is really all that effective
    // if board.pieces[KNIGHT + board.colour_to_move].count_ones() == 2 {
    //     eval += KNIGHT_PAIR_PEN;
    // }
    //
    // // no pawn penalty
    // if board.pieces[board.colour_to_move].count_ones() == 0 {
    //     eval += NO_PAWNS_PEN;
    // }


    eval * colour_mul
}

// fn mobility(board: &Board) -> i32 {
//     let mut mobility = 0;
//
//
//
//     mobility
// }

#[test]
fn iterator_funny_buiz() {

    let range: Vec<(i32, i32)> = (0..12).step_by(2).zip((1..12).step_by(2))
        .collect();

    let arr: Vec<(i32, i32)> = vec![(0 , 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)];

    assert_eq!(range, arr);
}

pub fn gen_pst_value(board: &Board) -> i32 {
    let mut pos = 0;

    // (0..12).step_by(2).zip((1..12).step_by(2)).for_each(|(w, b)|)

    [(0 , 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)].iter().for_each(|(w, b)| {
        let mut w_piece = board.pieces[*w];
        while w_piece > 0 {
            let sq = w_piece.trailing_zeros() as usize;
            pos += PST[*w][sq] as i32;
            w_piece &= w_piece - 1;
        }

        let mut b_piece = board.pieces[*b];
        while b_piece > 0 {
            let sq = b_piece.trailing_zeros() as usize;
            pos += PST[*b][sq] as i32;
            b_piece &= b_piece - 1;
        }
    });

    pos
}

pub fn gen_mat_value(b: &Board) -> i32 {
    let mut mat = 0;

    for piece in (0..10).step_by(2) {
        mat += b.pieces[piece].count_ones() as i32 * PIECE_VALUES[piece];
        mat -= b.pieces[piece+1].count_ones() as i32 * PIECE_VALUES[piece];
    }

    mat
}


#[test]
fn pst_symm() {
    for (w, b) in PST.iter().step_by(2).zip(PST.iter().skip(1).step_by(2)) {
        let fwd: Vec<&i8> = w.iter().collect();
        let bkw: Vec<&i8> = b.iter().rev().collect();

        assert_eq!(fwd, bkw);
    }
}