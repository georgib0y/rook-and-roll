use crate::board::board::{Board, ALL_PIECES, BKS_STATE, BQS_STATE, ROOK, WKS_STATE, WQS_STATE};
use crate::board::zorbist::Zorb;
use crate::movegen::move_info::{PST, SQUARES};
use crate::movegen::moves::{Move, MoveType};
use crate::search::eval::MAT_SCORES;
use std::ptr;

pub trait BoardCopier {
    fn copy(b: &Board, m: Move) -> Board;
}

pub struct BoardCopierWithHash;

impl BoardCopierWithHash {
    fn copy_pieces(pieces: &[u64; 12], piece: usize, from_to: u64) -> [u64; 12] {
        let mut new_pieces = [0; 12];
        // memcpy, this is safe as the src and dest pointers do not point to the
        // same data
        unsafe {
            let src = pieces.as_ptr();
            let dst = new_pieces.as_mut_ptr();
            ptr::copy_nonoverlapping(src, dst, 12);
        }
        new_pieces[piece] ^= from_to;
        new_pieces
    }

    fn copy_util(mut util: [u64; 3], ctm: usize, from_to: u64) -> [u64; 3] {
        util[ctm] ^= from_to;
        util[ALL_PIECES] ^= from_to;
        util
    }

    fn copy_hash(hash: u64, ep: usize, piece: usize, from: usize, to: usize) -> u64 {
        hash ^ Zorb::piece(piece, from)
            ^ Zorb::piece(piece, to)
            ^ (ep < 64) as u64 * Zorb::ep_file(ep)
    }

    fn copy_values(
        mut mg_value: i32,
        mut eg_value: i32,
        piece: usize,
        from: usize,
        to: usize,
    ) -> (i32, i32) {
        let (mg_from, eg_from) = PST::pst(piece, from);
        let (mg_to, eg_to) = PST::pst(piece, to);
        mg_value -= (mg_from + mg_to) as i32;
        eg_value -= (eg_from + eg_to) as i32;
        (mg_value, eg_value)
    }

    pub fn copy_castle_state(
        mut castle_state: u8,
        piece: usize,
        from: usize,
        to: usize,
        hash: &mut u64,
    ) -> u8 {
        // stop thinking you can optimise this you have the ifs for the hash

        if (piece == 10 || from == 7 || to == 7) && castle_state & 0b1000 > 0 {
            castle_state &= 0b0111;
            *hash ^= Zorb::castle_rights(WKS_STATE);
        }

        if (piece == 10 || from == 0 || to == 0) && castle_state & 0b100 > 0 {
            castle_state &= 0b1011;
            *hash ^= Zorb::castle_rights(WQS_STATE);
        }

        if (piece == 11 || from == 63 || to == 63) && castle_state & 0b10 > 0 {
            castle_state &= 0b1101;
            *hash ^= Zorb::castle_rights(BKS_STATE);
        }

        if (piece == 11 || from == 56 || to == 56) && castle_state & 0b1 > 0 {
            castle_state &= 0b1110;
            *hash ^= Zorb::castle_rights(BQS_STATE);
        }

        castle_state
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
        b.halfmove *= (piece > 1) as usize;
    }

    fn apply_double(b: &mut Board, to: usize) {
        b.ep = to - 8 + (b.ctm * 16);
        b.hash ^= Zorb::ep_file(b.ep);
        b.halfmove = 0;
    }

    fn apply_cap(b: &mut Board, xpiece: usize, to: usize) {
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.ctm ^ 1] ^= SQUARES[to];
        b.util[ALL_PIECES] ^= SQUARES[to];

        let (mg_to, eg_to) = PST::pst(xpiece, to);
        b.mg_value -= MAT_SCORES[xpiece] + mg_to as i32;
        b.eg_value -= MAT_SCORES[xpiece] + eg_to as i32;

        b.hash ^= Zorb::piece(xpiece, to);
        b.halfmove = 0;
    }

    fn apply_castle(b: &mut Board, colour: usize, from: usize, to: usize) {
        let sqs = SQUARES[from] | SQUARES[to];
        b.pieces[ROOK + colour] ^= sqs;
        b.util[colour] ^= sqs;
        b.util[ALL_PIECES] ^= sqs;

        let (mg_from, eg_from) = PST::pst(ROOK + colour, from);
        let (mg_to, eg_to) = PST::pst(ROOK + colour, to);
        b.mg_value -= (mg_from + mg_to) as i32;
        b.eg_value -= (eg_from + eg_to) as i32;

        b.hash ^= Zorb::piece(ROOK + colour, from);
        b.hash ^= Zorb::piece(ROOK + colour, to);
    }

    fn apply_promo(b: &mut Board, piece: usize, xpiece: usize, to: usize) {
        // toggle the pawn off and the toggled piece on
        b.pieces[b.ctm] ^= SQUARES[to];
        b.pieces[xpiece] ^= SQUARES[to];

        b.hash ^= Zorb::piece(piece, to);
        b.hash ^= Zorb::piece(xpiece, to);

        let (mg_to, eg_to) = PST::pst(piece, to);
        let (mg_xto, eg_xto) = PST::pst(xpiece, to);

        b.mg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
        b.mg_value -= (mg_to + mg_xto) as i32;
        b.eg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
        b.eg_value -= (eg_to + eg_xto) as i32;
        b.halfmove = 0;
    }

    fn apply_promo_cap(b: &mut Board, move_type: MoveType, piece: usize, xpiece: usize, to: usize) {
        // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
        // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
        let promo_piece = (move_type as usize - 7) * 2 + b.ctm;

        // toggle captured piece
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.ctm ^ 1] ^= SQUARES[to];
        // retoggle piece (as its been replaced by the capture-er)
        b.util[ALL_PIECES] ^= SQUARES[to];
        // toggle pawn off
        b.pieces[b.ctm] ^= SQUARES[to];
        // toggle promo
        b.pieces[promo_piece] ^= SQUARES[to];

        b.hash ^= Zorb::piece(piece, to);
        b.hash ^= Zorb::piece(promo_piece, to);
        b.hash ^= Zorb::piece(xpiece, to);

        // update mat value (the promo piece - the captured piece and the pre-promoted piece)
        let (mg_to, eg_to) = PST::pst(piece, to);
        let (mg_xto, eg_xto) = PST::pst(xpiece, to);
        let (mg_pto, eg_pto) = PST::pst(promo_piece, to);

        b.mg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
        b.mg_value -= (mg_xto - mg_to + mg_pto) as i32;
        b.eg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
        b.eg_value -= (eg_xto - eg_to + eg_pto) as i32;
        b.halfmove = 0;
    }

    fn apply_ep(b: &mut Board, to: usize) {
        let ep_sq = to - 8 + (b.ctm * 16);
        b.pieces[b.ctm ^ 1] ^= SQUARES[ep_sq]; // toggle capture pawn off
        b.util[b.ctm ^ 1] ^= SQUARES[ep_sq];
        b.util[ALL_PIECES] ^= SQUARES[ep_sq];

        b.hash ^= Zorb::piece(b.ctm ^ 1, ep_sq);

        let (mg_ep, eg_ep) = PST::pst(b.ctm ^ 1, ep_sq);
        b.mg_value -= MAT_SCORES[b.ctm ^ 1] + mg_ep as i32;
        b.eg_value -= MAT_SCORES[b.ctm ^ 1] + eg_ep as i32;
        b.halfmove = 0;
    }
}

impl BoardCopier for BoardCopierWithHash {
    fn copy(b: &Board, m: Move) -> Board {
        let (from, to, piece, xpiece, move_type) = m.all();
        let from_to = SQUARES[from] | SQUARES[to];

        let mut hash = Self::copy_hash(b.hash, b.ep, piece, from, to);
        let (mg_value, eg_value) = Self::copy_values(b.mg_value, b.eg_value, piece, from, to);
        let castle_state = Self::copy_castle_state(b.castle_state, piece, from, to, &mut hash);

        let mut board = Board {
            pieces: Self::copy_pieces(&b.pieces, piece, from_to),
            util: Self::copy_util(b.util, b.ctm, from_to),
            ctm: b.ctm,
            castle_state,
            ep: 64,
            halfmove: b.halfmove + 1,
            hash,
            mg_value,
            eg_value,
        };

        Self::apply_move(&mut board, to, piece, xpiece, move_type);

        board.hash ^= Zorb::colour();
        board.ctm ^= 1;

        board
    }
}

pub struct BoardCopierWithoutHash;

impl BoardCopierWithoutHash {
    fn copy_pieces(mut pieces: [u64; 12], piece: usize, from_to: u64) -> [u64; 12] {
        pieces[piece] ^= from_to;
        pieces
    }

    fn copy_util(mut util: [u64; 3], ctm: usize, from_to: u64) -> [u64; 3] {
        util[ctm] ^= from_to;
        util[ALL_PIECES] ^= from_to;
        util
    }

    pub fn copy_castle_state(mut castle_state: u8, piece: usize, from: usize, to: usize) -> u8 {
        // stop thinking you can optimise this you have the ifs for the hash

        if (piece == 10 || from == 7 || to == 7) && castle_state & 0b1000 > 0 {
            castle_state &= 0b0111;
        }

        if (piece == 10 || from == 0 || to == 0) && castle_state & 0b100 > 0 {
            castle_state &= 0b1011;
        }

        if (piece == 11 || from == 63 || to == 63) && castle_state & 0b10 > 0 {
            castle_state &= 0b1101;
        }

        if (piece == 11 || from == 56 || to == 56) && castle_state & 0b1 > 0 {
            castle_state &= 0b1110;
        }

        castle_state
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
            MoveType::Promo => Self::apply_promo(b, xpiece, to),
            MoveType::NPromoCap
            | MoveType::RPromoCap
            | MoveType::BPromoCap
            | MoveType::QPromoCap => Self::apply_promo_cap(b, move_type, xpiece, to),
            MoveType::Ep => Self::apply_ep(b, to),
        }
    }

    fn apply_quiet(b: &mut Board, piece: usize) {
        b.halfmove *= (piece > 1) as usize;
    }

    fn apply_double(b: &mut Board, to: usize) {
        b.ep = to - 8 + (b.ctm * 16);
        b.halfmove = 0;
    }

    fn apply_cap(b: &mut Board, xpiece: usize, to: usize) {
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.ctm ^ 1] ^= SQUARES[to];
        b.util[ALL_PIECES] ^= SQUARES[to];
        b.halfmove = 0;
    }

    fn apply_castle(b: &mut Board, colour: usize, from: usize, to: usize) {
        let sqs = SQUARES[from] | SQUARES[to];
        b.pieces[ROOK + colour] ^= sqs;
        b.util[colour] ^= sqs;
        b.util[ALL_PIECES] ^= sqs;
    }

    fn apply_promo(b: &mut Board, xpiece: usize, to: usize) {
        // toggle the pawn off and the toggled piece on
        b.pieces[b.ctm] ^= SQUARES[to];
        b.pieces[xpiece] ^= SQUARES[to];
        b.halfmove = 0;
    }

    fn apply_promo_cap(b: &mut Board, move_type: MoveType, xpiece: usize, to: usize) {
        // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
        // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
        let promo_piece = (move_type as usize - 7) * 2 + b.ctm;

        // toggle captured piece
        b.pieces[xpiece] ^= SQUARES[to];
        b.util[b.ctm ^ 1] ^= SQUARES[to];
        // retoggle piece (as its been replaced by the capture-er)
        b.util[ALL_PIECES] ^= SQUARES[to];
        // toggle pawn off
        b.pieces[b.ctm] ^= SQUARES[to];
        // toggle promo
        b.pieces[promo_piece] ^= SQUARES[to];

        b.halfmove = 0;
    }

    fn apply_ep(b: &mut Board, to: usize) {
        let ep_sq = to - 8 + (b.ctm * 16);
        b.pieces[b.ctm ^ 1] ^= SQUARES[ep_sq]; // toggle capture pawn off
        b.util[b.ctm ^ 1] ^= SQUARES[ep_sq];
        b.util[ALL_PIECES] ^= SQUARES[ep_sq];
        b.halfmove = 0;
    }
}

impl BoardCopier for BoardCopierWithoutHash {
    fn copy(b: &Board, m: Move) -> Board {
        let (from, to, piece, xpiece, move_type) = m.all();
        let from_to = SQUARES[from] | SQUARES[to];

        let castle_state = Self::copy_castle_state(b.castle_state, piece, from, to);

        let mut board = Board {
            pieces: Self::copy_pieces(b.pieces, piece, from_to),
            util: Self::copy_util(b.util, b.ctm, from_to),
            ctm: b.ctm,
            castle_state,
            ep: 64,
            halfmove: b.halfmove + 1,
            hash: 0,
            mg_value: 0,
            eg_value: 0,
        };

        Self::apply_move(&mut board, to, piece, xpiece, move_type);

        board.ctm ^= 1;

        board
    }
}
