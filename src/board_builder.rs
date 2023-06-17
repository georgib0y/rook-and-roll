use crate::board::{Board, BKS_STATE, BQS_STATE, ROOK, WKS_STATE, WQS_STATE};
use crate::eval::MAT_SCORES;
use crate::move_info::{PST, SQUARES};
use crate::moves::MoveType;
use crate::zorbist::Zorb;

pub struct BoardBuilder {
    pieces: [u64; 12],
    util: [u64; 3],
    ctm: usize,
    castle_state: u8,
    ep: usize,
    halfmove: usize,
    hash: u64,
    mg_value: i32,
    eg_value: i32,
}

impl BoardBuilder {
    pub fn new(board: &Board, from: usize, to: usize, piece: usize) -> BoardBuilder {
        let ft = SQUARES[from] | SQUARES[to];
        let mut hash =
            copy_hash(board, piece, from, to) ^ (board.ep < 64) as u64 * Zorb::ep_file(board.ep);
        let (mg_value, eg_value) = copy_values(board, piece, from, to);
        let castle_state = copy_castle_state(board.castle_state, piece, from, to, &mut hash);

        BoardBuilder {
            pieces: copy_pieces(board, piece, ft),
            util: copy_util(board, ft),
            ctm: board.ctm,
            castle_state,
            ep: 64,
            halfmove: board.halfmove + 1,
            hash,
            mg_value,
            eg_value,
        }
    }

    pub fn apply_move(
        mut self,
        to: usize,
        piece: usize,
        xpiece: usize,
        move_type: MoveType,
    ) -> Self {
        match move_type {
            MoveType::Quiet => self.apply_quiet(piece),
            MoveType::Double => self.apply_double(to),
            MoveType::Cap => self.apply_cap(xpiece, to),
            MoveType::WKingSide => self.apply_castle(0, 7, 5),
            MoveType::BKingSide => self.apply_castle(1, 63, 61),
            MoveType::WQueenSide => self.apply_castle(0, 0, 3),
            MoveType::BQueenSide => self.apply_castle(1, 56, 59),
            MoveType::Promo => self.apply_promo(piece, xpiece, to),
            MoveType::NPromoCap
            | MoveType::RPromoCap
            | MoveType::BPromoCap
            | MoveType::QPromoCap => self.apply_promo_cap(move_type, piece, xpiece, to),
            MoveType::Ep => self.apply_ep(to),
        }

        self
    }

    pub fn build(&self) -> Board {
        Board {
            pieces: self.pieces,
            util: self.util,
            ctm: self.ctm ^ 1,
            castle_state: self.castle_state,
            ep: self.ep,
            halfmove: self.halfmove,
            hash: self.hash ^ Zorb::colour(),
            mg_value: self.mg_value,
            eg_value: self.eg_value,
        }
    }

    pub fn apply_quiet(&mut self, piece: usize) {
        self.halfmove *= (piece > 1) as usize;
    }

    pub fn apply_double(&mut self, to: usize) {
        self.ep = to - 8 + (self.ctm * 16);
        self.hash ^= Zorb::ep_file(self.ep);
        self.halfmove = 0;
    }

    pub fn apply_cap(&mut self, xpiece: usize, to: usize) {
        self.pieces[xpiece] ^= SQUARES[to];
        self.util[self.ctm ^ 1] ^= SQUARES[to];
        self.util[2] ^= SQUARES[to];

        self.mg_value -= MAT_SCORES[xpiece] + PST::mid_pst(xpiece, to);
        self.eg_value -= MAT_SCORES[xpiece] + PST::end_pst(xpiece, to);

        self.hash ^= Zorb::piece(xpiece, to);
        self.halfmove = 0;
    }

    pub fn apply_castle(&mut self, colour: usize, from: usize, to: usize) {
        let sqs = SQUARES[from] | SQUARES[to];
        self.pieces[ROOK + colour] ^= sqs;
        self.util[colour] ^= sqs;
        self.util[2] ^= sqs;

        self.mg_value += -PST::mid_pst(ROOK + colour, from) + PST::mid_pst(ROOK + colour, to);
        self.eg_value += -PST::end_pst(ROOK + colour, from) + PST::end_pst(ROOK + colour, to);

        self.hash ^= Zorb::piece(ROOK + colour, from);
        self.hash ^= Zorb::piece(ROOK + colour, to);
    }

    pub fn apply_promo(&mut self, piece: usize, xpiece: usize, to: usize) {
        // toggle the pawn off and the toggled piece on
        self.pieces[self.ctm] ^= SQUARES[to];
        self.pieces[xpiece] ^= SQUARES[to];

        self.hash ^= Zorb::piece(piece, to);
        self.hash ^= Zorb::piece(xpiece, to);

        self.mg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
        self.mg_value += -PST::mid_pst(piece, to) + PST::mid_pst(xpiece, to);
        self.eg_value += -MAT_SCORES[piece] + MAT_SCORES[xpiece];
        self.eg_value += -PST::end_pst(piece, to) + PST::end_pst(xpiece, to);
        self.halfmove = 0;
    }

    pub fn apply_promo_cap(&mut self, move_type: MoveType, piece: usize, xpiece: usize, to: usize) {
        // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
        // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
        let promo_piece = (move_type as usize - 7) * 2 + self.ctm;

        // toggle captured piece
        self.pieces[xpiece] ^= SQUARES[to];
        self.util[self.ctm ^ 1] ^= SQUARES[to];
        // retoggle piece (as its been replaced by the capture-er)
        self.util[2] ^= SQUARES[to];
        // toggle pawn off
        self.pieces[self.ctm] ^= SQUARES[to];
        // toggle promo
        self.pieces[promo_piece] ^= SQUARES[to];

        self.hash ^= Zorb::piece(piece, to);
        self.hash ^= Zorb::piece(promo_piece, to);
        self.hash ^= Zorb::piece(xpiece, to);

        // update mat value (the promo piece - the captured piece and the pre-promoted piece)
        self.mg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
        self.mg_value +=
            -PST::mid_pst(xpiece, to) - PST::mid_pst(piece, to) + PST::mid_pst(promo_piece, to);
        self.eg_value += MAT_SCORES[promo_piece] - MAT_SCORES[xpiece] - MAT_SCORES[piece];
        self.eg_value +=
            -PST::end_pst(xpiece, to) - PST::end_pst(piece, to) + PST::end_pst(promo_piece, to);
        self.halfmove = 0;
    }

    pub fn apply_ep(&mut self, to: usize) {
        let ep_sq = to - 8 + (self.ctm * 16);
        self.pieces[self.ctm ^ 1] ^= SQUARES[ep_sq]; // toggle capture pawn off
        self.util[self.ctm ^ 1] ^= SQUARES[ep_sq];
        self.util[2] ^= SQUARES[ep_sq];

        self.hash ^= Zorb::piece(self.ctm ^ 1, ep_sq);

        self.mg_value -= MAT_SCORES[self.ctm ^ 1] + PST::mid_pst(self.ctm ^ 1, ep_sq);
        self.eg_value -= MAT_SCORES[self.ctm ^ 1] + PST::end_pst(self.ctm ^ 1, ep_sq);
        self.halfmove = 0;
    }
}

fn copy_hash(board: &Board, piece: usize, from: usize, to: usize) -> u64 {
    board.hash ^ Zorb::piece(piece, from) ^ Zorb::piece(piece, to)
}

fn copy_pieces(board: &Board, piece: usize, from_to: u64) -> [u64; 12] {
    let mut pieces = board.pieces;
    pieces[piece] ^= from_to;
    pieces
}

fn copy_util(board: &Board, from_to: u64) -> [u64; 3] {
    let mut util = board.util;
    util[board.ctm] ^= from_to;
    util[2] ^= from_to;
    util
}

fn copy_ep(board: &Board, hash: &mut u64) -> usize {
    *hash ^= (board.ep < 64) as u64 * Zorb::ep_file(board.ep);
    64
}

fn copy_values(board: &Board, piece: usize, from: usize, to: usize) -> (i32, i32) {
    let mg_value = board.mg_value - PST::mid_pst(piece, from) + PST::mid_pst(piece, to);
    let eg_value = board.eg_value - PST::end_pst(piece, from) + PST::end_pst(piece, to);
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

fn _copy_castle_state(
    mut castle_state: u8,
    piece: usize,
    from: usize,
    to: usize,
    hash: &mut u64,
) -> u8 {
    let wks = ((piece == 10 || from == 7 || to == 7) && castle_state & 0b1000 == 0) as u8;
    castle_state &= wks << 3 | 0b0111;
    *hash ^= wks as u64 * Zorb::castle_rights(WKS_STATE);

    let wqs = ((piece == 10 || from == 0 || to == 0) && castle_state & 0b100 == 0) as u8;
    castle_state &= wqs << 2 | 0b1011;
    *hash ^= wqs as u64 * Zorb::castle_rights(WQS_STATE);

    let bks = ((piece == 11 || from == 63 || to == 63) && castle_state & 0b10 == 0) as u8;
    castle_state &= bks << 1 | 0b1101;
    *hash ^= bks as u64 * Zorb::castle_rights(BKS_STATE);

    let bqs = ((piece == 11 || from == 56 || to == 56) && castle_state & 0b1 == 0) as u8;
    castle_state &= bqs | 0b1110;
    *hash ^= bqs as u64 * Zorb::castle_rights(BQS_STATE);

    castle_state
}
