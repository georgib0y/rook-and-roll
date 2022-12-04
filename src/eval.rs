use crate::move_info::PST;
use crate::Board;

pub const CHECKMATE: i32 = -1000000000;
pub const MATED: i32 = -CHECKMATE;

pub const STALEMATE: i32 = 0;

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

// const BISHOP_PAIR_BONUS: i32 = 30;
// const _ROOK_PAIR_PEN: i32 = -10;
// const _KNIGHT_PAIR_PEN: i32 = -10;
// const NO_PAWNS_PEN: i32 = -200;

const PIECE_PHASE_VAL: [i32; 12] = [0,0,1,1,1,1,2,2,4,4,0,0];

// TODO incremental update of game phase
pub fn eval(board: &Board, colour_mul: i32) -> i32 {
    // let mat = gen_mat_value(board);
    // let (mg_pst, eg_pst) = gen_pst_value(board);
    // assert_eq!(board.mg_value, mat+mg_pst);
    // assert_eq!(board.eg_value, mat+eg_pst);


    let mut mg_phase = 0;
    for (p, pieces) in board.pieces.iter().enumerate().skip(2).take(8) {
        mg_phase += PIECE_PHASE_VAL[p] * pieces.count_ones() as i32;
    }

    if mg_phase > 24 { mg_phase = 24 }
    let eg_phase = 24 - mg_phase;

    let eval = (board.mg_value * mg_phase + board.eg_value * eg_phase) / 24;

    // if board.is_endgame() {
    //     // adjust king pst to endgame pst
    //     let ksq = board.pieces[KING+board.colour_to_move].trailing_zeros() as usize;
    //     eval += (-PST[KING+board.colour_to_move][ksq] + -PST[KING+board.colour_to_move+2][ksq]) as i32
    // }

    // bishop pair bonus
    // if board.pieces[BISHOP + board.colour_to_move].count_ones() == 2 {
    //     eval += BISHOP_PAIR_BONUS;
    // }
    // // // rook pair pen (redundancy)
    // if board.pieces[ROOK + board.colour_to_move].count_ones() == 2 {
    //     eval += ROOK_PAIR_PEN;
    // }
    // // knight pair pen (worse than other minor piece pairs)
    // if board.pieces[KNIGHT + board.colour_to_move].count_ones() == 2 {
    //     eval += KNIGHT_PAIR_PEN;
    // }
    //
    // no pawn penalty
    // if board.pieces[board.colour_to_move].count_ones() == 0 {
    //     eval += NO_PAWNS_PEN;
    // }


    eval * colour_mul
}



#[test]
fn iterator_funny_buiz() {

    let range: Vec<(i32, i32)> = (0..12).step_by(2).zip((1..12).step_by(2))
        .collect();

    let arr: Vec<(i32, i32)> = vec![(0 , 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)];

    assert_eq!(range, arr);
}

/// returns (mg, eg) as values
pub fn gen_pst_value(board: &Board) -> (i32, i32) {
    let mut mg = 0;
    let mut eg = 0;

    // (0..12).step_by(2).zip((1..12).step_by(2)).for_each(|(w, b)|)

    [(0 , 1), (2, 3), (4, 5), (6, 7), (8, 9), (10, 11)].iter().for_each(|(w, b)| {
        let mut w_piece = board.pieces[*w];
        while w_piece > 0 {
            let sq = w_piece.trailing_zeros() as usize;
            // pos += PST[*w][sq] as i32;
            mg += PST::mid_pst(*w, sq);
            eg += PST::end_pst(*w, sq);
            w_piece &= w_piece - 1;
        }

        let mut b_piece = board.pieces[*b];
        while b_piece > 0 {
            let sq = b_piece.trailing_zeros() as usize;
            // pos += PST[*b][sq] as i32;
            mg += PST::mid_pst(*b, sq);
            eg += PST::end_pst(*b, sq);
            b_piece &= b_piece - 1;
        }
    });

    (mg, eg)
}

// r1bqkb1r/ppp2ppp/4pn2/8/Q1nP1B2/2N1PN2/PP3PPP/R3K2R b KQkq - 1 8

pub fn gen_mat_value(b: &Board) -> i32 {
    let mut mat = 0;

    for piece in (0..10).step_by(2) {
        mat += b.pieces[piece].count_ones() as i32 * PIECE_VALUES[piece];
        mat -= b.pieces[piece+1].count_ones() as i32 * PIECE_VALUES[piece];
    }

    mat
}


// #[test]
// fn pst_symm() {
//     for (w, b) in  {
//         let fwd: Vec<&i8> = w.iter().collect();
//         let bkw: Vec<&i8> = b.iter().rev().collect();
//
//         assert_eq!(fwd, bkw);
//     }
// }