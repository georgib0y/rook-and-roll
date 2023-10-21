use crate::board::board::{Board, ALL_PIECES, BKS_STATE, BQS_STATE, ROOK, WKS_STATE, WQS_STATE};
use crate::board::zorbist::Zorb;
use crate::movegen::move_info::{PST, SQUARES};
use crate::movegen::moves::{Move, MoveType};
use crate::search::eval::{gen_board_value, MAT_SCORES};

pub trait BoardCopier {
    fn copy(b: &Board, m: Move) -> Board;
}

pub struct BoardCopierWithHash;

impl BoardCopierWithHash {
    fn set_pieces(b: &mut Board, piece: usize, from_to: u64) {
        b.pieces[piece] ^= from_to;
    }

    fn set_util(b: &mut Board, from_to: u64) {
        b.util[b.ctm()] ^= from_to;
        b.util[ALL_PIECES] ^= from_to;
    }

    fn set_hash(b: &mut Board, piece: usize, from: usize, to: usize) {
        b.hash ^= Zorb::piece(piece, from)
            ^ Zorb::piece(piece, to)
            ^ (b.ep < 64) as u64 * Zorb::ep_file(b.ep())
    }

    fn set_values(b: &mut Board, piece: usize, from: usize, to: usize) {
        Self::add_piece_value(b, piece, to);
        Self::remove_piece_value(b, piece, from);
    }

    pub fn set_castle_state(b: &mut Board, piece: usize, from: usize, to: usize) {
        // stop thinking you can optimise this

        if (piece == 10 || from == 7 || to == 7) && b.castle_state & 0b1000 > 0 {
            b.castle_state &= 0b0111;
            b.hash ^= Zorb::castle_rights(WKS_STATE);
        }

        if (piece == 10 || from == 0 || to == 0) && b.castle_state & 0b100 > 0 {
            b.castle_state &= 0b1011;
            b.hash ^= Zorb::castle_rights(WQS_STATE);
        }

        if (piece == 11 || from == 63 || to == 63) && b.castle_state & 0b10 > 0 {
            b.castle_state &= 0b1101;
            b.hash ^= Zorb::castle_rights(BKS_STATE);
        }

        if (piece == 11 || from == 56 || to == 56) && b.castle_state & 0b1 > 0 {
            b.castle_state &= 0b1110;
            b.hash ^= Zorb::castle_rights(BQS_STATE);
        }
    }
    fn apply_move(b: &mut Board, to: usize, piece: usize, xpiece: usize, move_type: MoveType) {
        match move_type {
            MoveType::Quiet => Self::apply_quiet(b, piece),
            MoveType::Double => Self::apply_double(b, to),
            MoveType::Cap => Self::apply_cap(b, xpiece, to),
            MoveType::WKingSide => Self::apply_castle(b, 0, 7, 5),
            MoveType::BKingSide => Self::apply_castle(b, 1, 63, 61),
            MoveType::WQueenSide => Self::apply_castle(b, 0, 0, 3),
            MoveType::BQueenSide => Self::apply_castle(b, 1, 56, 59),
            MoveType::Promo => Self::apply_promo(b, piece, xpiece, to),
            MoveType::NPromoCap
            | MoveType::RPromoCap
            | MoveType::BPromoCap
            | MoveType::QPromoCap => Self::apply_promo_cap(b, move_type, piece, xpiece, to),
            MoveType::Ep => Self::apply_ep(b, to),
        }
    }

    fn apply_quiet(b: &mut Board, piece: usize) {
        b.halfmove *= (piece > 1) as u16;
    }

    fn apply_double(b: &mut Board, to: usize) {
        b.ep = to as u8 - 8 + (b.ctm * 16);
        b.hash ^= Zorb::ep_file(b.ep());
        b.halfmove = 0;
    }

    fn apply_cap(b: &mut Board, xpiece: usize, to: usize) {
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.opp_ctm()] ^= SQUARES[to];
        b.util[ALL_PIECES] ^= SQUARES[to];

        Self::remove_piece_value(b, xpiece, to);
        Self::toggle_piece_hash(b, xpiece, to);

        b.halfmove = 0;
    }

    fn apply_castle(b: &mut Board, colour: usize, from: usize, to: usize) {
        let sqs = SQUARES[from] | SQUARES[to];
        b.pieces[ROOK + colour] ^= sqs;
        b.util[colour] ^= sqs;
        b.util[ALL_PIECES] ^= sqs;

        Self::add_piece_value(b, ROOK + colour, to);
        Self::remove_piece_value(b, ROOK + colour, from);

        Self::toggle_piece_hash(b, ROOK + colour, to);
        Self::toggle_piece_hash(b, ROOK + colour, from);
    }

    fn apply_promo(b: &mut Board, piece: usize, xpiece: usize, to: usize) {
        // toggle the pawn off and the toggled piece on
        b.pieces[b.ctm()] ^= SQUARES[to];
        b.pieces[xpiece] ^= SQUARES[to];

        Self::toggle_piece_hash(b, piece, to);
        Self::toggle_piece_hash(b, xpiece, to);

        Self::remove_piece_value(b, piece, to);
        Self::add_piece_value(b, xpiece, to);

        b.halfmove = 0;
    }

    fn apply_promo_cap(b: &mut Board, move_type: MoveType, piece: usize, xpiece: usize, to: usize) {
        // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
        // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
        let promo_piece = (move_type as usize - 7) * 2 + b.ctm();

        // toggle captured piece
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.opp_ctm()] ^= SQUARES[to];
        // retoggle piece (as its been replaced by the capture-er)
        b.util[ALL_PIECES] ^= SQUARES[to];
        // toggle pawn off
        b.pieces[b.ctm()] ^= SQUARES[to];
        // toggle promo
        b.pieces[promo_piece] ^= SQUARES[to];

        Self::toggle_piece_hash(b, piece, to);
        Self::toggle_piece_hash(b, promo_piece, to);
        Self::toggle_piece_hash(b, xpiece, to);

        Self::remove_piece_value(b, piece, to);
        Self::remove_piece_value(b, xpiece, to);
        Self::add_piece_value(b, promo_piece, to);

        b.halfmove = 0;
    }

    fn apply_ep(b: &mut Board, to: usize) {
        let ep_sq = to - 8 + (b.ctm() * 16);
        b.pieces[b.opp_ctm()] ^= SQUARES[ep_sq]; // toggle capture pawn off
        b.util[b.opp_ctm()] ^= SQUARES[ep_sq];
        b.util[ALL_PIECES] ^= SQUARES[ep_sq];

        Self::toggle_piece_hash(b, b.opp_ctm(), ep_sq);

        Self::remove_piece_value(b, b.opp_ctm(), ep_sq);

        b.halfmove = 0;
    }

    fn toggle_piece_hash(b: &mut Board, piece: usize, to: usize) {
        b.hash ^= Zorb::piece(piece, to);
    }

    fn add_piece_value(b: &mut Board, piece: usize, sq: usize) {
        let mat = MAT_SCORES[piece];
        let (mg, eg) = PST::pst(piece, sq);

        b.mg_value += mat + mg as i32;
        b.eg_value += mat + eg as i32;
    }

    fn remove_piece_value(b: &mut Board, piece: usize, sq: usize) {
        let mat = MAT_SCORES[piece];
        let (mg, eg) = PST::pst(piece, sq);

        b.mg_value -= mat + mg as i32;
        b.eg_value -= mat + eg as i32;
    }
}

impl BoardCopier for BoardCopierWithHash {
    fn copy(b: &Board, m: Move) -> Board {
        let (from, to, piece, xpiece, move_type) = m.all();
        let from_to = SQUARES[from] | SQUARES[to];

        let mut board = *b;

        Self::set_pieces(&mut board, piece, from_to);
        Self::set_util(&mut board, from_to);
        Self::set_castle_state(&mut board, piece, from, to);
        Self::set_hash(&mut board, piece, from, to);

        Self::set_values(&mut board, piece, from, to);

        board.ep = 64;
        board.halfmove += 1;

        Self::apply_move(&mut board, to, piece, xpiece, move_type);

        board.hash ^= Zorb::colour();
        board.ctm ^= 1;

        board
    }
}

#[test]
fn test_inc_values_and_hash_copy_make() {
    crate::init();
    use crate::board::board::*;

    let tests = vec![
        (
            "Captures",
            Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            vec![
                Move::new(21, 23, QUEEN as u32, BLACK as u32, MoveType::Cap),
                Move::new(12, 40, BISHOP as u32, BISHOP as u32 + 1, MoveType::Cap),
                Move::new(36, 53, KNIGHT as u32, BLACK as u32, MoveType::Cap),
            ],
        ),
        (
            "Castles W",
            Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
                .unwrap(),
            vec![
                Move::new(4, 2, KING as u32, 0, MoveType::WQueenSide),
                Move::new(4, 6, KING as u32, 0, MoveType::WKingSide),
            ],
        ),
        (
            "Castles B",
            Board::new_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq -")
                .unwrap(),
            vec![
                Move::new(60, 58, KING as u32 + 1, 0, MoveType::BQueenSide),
                Move::new(60, 62, KING as u32 + 1, 0, MoveType::BKingSide),
            ],
        ),
        (
            "Ep",
            Board::new_fen("8/8/3p4/KPp4r/4Pp1k/8/6P1/1R6 b - e3 0 2").unwrap(),
            vec![Move::new(29, 20, BLACK as u32, WHITE as u32, MoveType::Ep)],
        ),
    ];

    tests.into_iter().for_each(|(title, board, moves)| {
        println!("{title}\n{board}");

        let (mg_value, eg_value) = gen_board_value(&board);

        assert_eq!(board.mg_value, mg_value);
        assert_eq!(board.eg_value, eg_value);

        moves.into_iter().for_each(|m| {
            let b = board.copy_make(m);

            println!("{b}\n{m}");

            let (mg_value, eg_value) = gen_board_value(&b);

            assert_eq!(b.mg_value, mg_value);
            assert_eq!(b.eg_value, eg_value);

            let hash = gen_hash(b);
            assert_eq!(b.hash, hash)
        })
    });
}
