use crate::eval::{gen_board_value, MAT_SCORES};
use crate::move_info::{PST, SQUARES};
use crate::movegen::{get_piece, get_xpiece};
use crate::moves::{Move, MoveType};
use std::fmt;
use std::ops::Index;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const ALL_PIECES: usize = 2;
pub const PAWN: usize = 0;
pub const KNIGHT: usize = 2;
pub const ROOK: usize = 4;
pub const BISHOP: usize = 6;
pub const QUEEN: usize = 8;
pub const KING: usize = 10;

pub const PIECE_NAMES: [&str; 12] = ["P", "p", "N", "n", "R", "r", "B", "b", "Q", "q", "K", "k"];

const DEFAULT_PIECES: [u64; 12] = [
    0x000000000000FF00, //wp 0
    0x00FF000000000000, //bp 1
    0x0000000000000042, //wn 2
    0x4200000000000000, //bn 3
    0x0000000000000081, //wr 4
    0x8100000000000000, //br 5
    0x0000000000000024, //wb 6
    0x2400000000000000, //bb 7
    0x0000000000000008, //wq 8
    0x0800000000000000, //bq 9
    0x0000000000000010, //wk 10
    0x1000000000000000, //bk 11
];

const DEFAULT_UTIL: [u64; 3] = [
    0x000000000000FFFF, // white
    0xFFFF000000000000, // black
    0xFFFF00000000FFFF, // all
];

// 0 - white to move, 1 - black to move
#[derive(Debug, Copy, Clone, Eq)]
pub struct Board {
    pub(super) pieces: [u64; 12],
    pub(super) util: [u64; 3],
    pub(super) ctm: u8,
    pub(super) castle_state: u8,
    pub(super) ep: u8,
    pub(super) halfmove: u16,
    pub(super) hash: u64,
    pub(super) mg_value: i32,
    pub(super) eg_value: i32,
}

impl Default for Board {
    fn default() -> Self {
        Board::new()
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board: Board = Board {
            pieces: DEFAULT_PIECES,
            util: DEFAULT_UTIL,
            ctm: WHITE as u8,
            castle_state: 0b1111,
            ep: 64,
            halfmove: 0,
            hash: 0,
            mg_value: 0,
            eg_value: 0,
        };

        board.hash = gen_hash(board);

        (board.mg_value, board.eg_value) = gen_board_value(&board);

        board
    }

    #[inline]
    pub fn ctm(&self) -> usize {
        self.ctm as usize
    }

    #[inline]
    pub fn opp_ctm(&self) -> usize {
        self.ctm as usize ^ 1
    }

    #[inline]
    pub fn pieces_iter(&self) -> impl Iterator<Item = &u64> {
        self.pieces.iter()
    }

    #[inline]
    pub fn pieces<T: Into<usize>>(&self, piece: T) -> u64 {
        self.pieces[piece.into()]
    }

    #[inline]
    pub fn pawns(&self, ctm: usize) -> u64 {
        self.pieces[ctm]
    }

    #[inline]
    pub fn knights(&self, ctm: usize) -> u64 {
        self.pieces[KNIGHT + ctm]
    }

    #[inline]
    pub fn rooks(&self, ctm: usize) -> u64 {
        self.pieces[ROOK + ctm]
    }

    #[inline]
    pub fn bishops(&self, ctm: usize) -> u64 {
        self.pieces[BISHOP + ctm]
    }

    #[inline]
    pub fn queens(&self, ctm: usize) -> u64 {
        self.pieces[QUEEN + ctm]
    }

    #[inline]
    pub fn king(&self, ctm: usize) -> u64 {
        self.pieces[KING + ctm]
    }

    #[inline]
    pub fn king_idx(&self, ctm: usize) -> usize {
        self.king(ctm).trailing_zeros() as usize
    }

    #[inline]
    pub fn occ(&self, ctm: usize) -> u64 {
        self.util[ctm]
    }

    #[inline]
    pub fn all_occ(&self) -> u64 {
        self.util[ALL_PIECES]
    }

    #[inline]
    pub fn ep(&self) -> usize {
        self.ep as usize
    }

    #[inline]
    pub fn castle_state(&self) -> u8 {
        self.castle_state
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }

    #[inline]
    pub fn halfmove(&self) -> usize {
        self.halfmove as usize
    }

    #[inline]
    pub fn mg_value(&self) -> i32 {
        self.mg_value
    }

    #[inline]
    pub fn eg_value(&self) -> i32 {
        self.eg_value
    }

    pub fn copy_make(&self, m: Move) -> Board {
        let (from, to, piece, xpiece, move_type) = m.all();
        let from_to = SQUARES[from] | SQUARES[to];

        let mut board = *self;
        board.set_pieces(piece, from_to);
        board.set_util(from_to);
        board.set_castle_state(piece, from, to);
        board.set_hash(piece, from, to);
        board.set_values(piece, from, to);
        board.ep = 64;
        board.halfmove += 1;
        board.apply_move(to, piece, xpiece, move_type);
        board.hash ^= Zorb::colour();
        board.ctm ^= 1;

        // let (mg, eg) = gen_board_value(&board);
        // assert_eq!(board.mg_value, mg);
        // assert_eq!(board.eg_value, eg);

        board
    }

    fn set_pieces(&mut self, piece: usize, from_to: u64) {
        self.pieces[piece] ^= from_to;
    }

    fn set_util(&mut self, from_to: u64) {
        self.util[self.ctm()] ^= from_to;
        self.util[ALL_PIECES] ^= from_to;
    }

    fn set_hash(&mut self, piece: usize, from: usize, to: usize) {
        self.hash ^= Zorb::piece(piece, from)
            ^ Zorb::piece(piece, to)
            ^ ((self.ep < 64) as u64 * Zorb::ep_file(self.ep()))
    }

    fn set_values(&mut self, piece: usize, from: usize, to: usize) {
        self.add_piece_value(piece, to);
        self.remove_piece_value(piece, from);
    }

    pub fn set_castle_state(&mut self, piece: usize, from: usize, to: usize) {
        // stop thinking you can optimise this

        if (piece == 10 || from == 7 || to == 7) && self.castle_state & 0b1000 > 0 {
            self.castle_state &= 0b0111;
            self.hash ^= Zorb::castle_rights(WKS_STATE);
        }

        if (piece == 10 || from == 0 || to == 0) && self.castle_state & 0b100 > 0 {
            self.castle_state &= 0b1011;
            self.hash ^= Zorb::castle_rights(WQS_STATE);
        }

        if (piece == 11 || from == 63 || to == 63) && self.castle_state & 0b10 > 0 {
            self.castle_state &= 0b1101;
            self.hash ^= Zorb::castle_rights(BKS_STATE);
        }

        if (piece == 11 || from == 56 || to == 56) && self.castle_state & 0b1 > 0 {
            self.castle_state &= 0b1110;
            self.hash ^= Zorb::castle_rights(BQS_STATE);
        }
    }
    fn apply_move(&mut self, to: usize, piece: usize, xpiece: usize, move_type: MoveType) {
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
    }

    fn apply_quiet(&mut self, piece: usize) {
        self.halfmove *= (piece > 1) as u16;
    }

    fn apply_double(&mut self, to: usize) {
        self.ep = to as u8 - 8 + (self.ctm * 16);
        self.hash ^= Zorb::ep_file(self.ep());
        self.halfmove = 0;
    }

    fn apply_cap(&mut self, xpiece: usize, to: usize) {
        self.pieces[xpiece] ^= SQUARES[to];
        self.util[self.opp_ctm()] ^= SQUARES[to];
        self.util[ALL_PIECES] ^= SQUARES[to];

        self.remove_piece_value(xpiece, to);
        self.toggle_piece_hash(xpiece, to);

        self.halfmove = 0;
    }

    fn apply_castle(&mut self, colour: usize, from: usize, to: usize) {
        let sqs = SQUARES[from] | SQUARES[to];
        self.pieces[ROOK + colour] ^= sqs;
        self.util[colour] ^= sqs;
        self.util[ALL_PIECES] ^= sqs;

        self.add_piece_value(ROOK + colour, to);
        self.remove_piece_value(ROOK + colour, from);

        self.toggle_piece_hash(ROOK + colour, to);
        self.toggle_piece_hash(ROOK + colour, from);
    }

    fn apply_promo(&mut self, piece: usize, xpiece: usize, to: usize) {
        // toggle the pawn off and the toggled piece on
        self.pieces[self.ctm()] ^= SQUARES[to];
        self.pieces[xpiece] ^= SQUARES[to];

        self.toggle_piece_hash(piece, to);
        self.toggle_piece_hash(xpiece, to);

        self.remove_piece_value(piece, to);
        self.add_piece_value(xpiece, to);

        self.halfmove = 0;
    }

    fn apply_promo_cap(&mut self, move_type: MoveType, piece: usize, xpiece: usize, to: usize) {
        // N_PROMO_CAP (8) - 7 = [1], [1] * 2 + b.colour_to_move == 2 or 3 (knight idx)
        // R_PROMO_CAP (9) - 7 = [2], [2] * 2 + b.colour_to_move == 4 or 5 (rook idx) etc
        let promo_piece = (move_type as usize - 7) * 2 + self.ctm();

        // toggle captured piece
        self.pieces[xpiece] ^= SQUARES[to];
        self.util[self.opp_ctm()] ^= SQUARES[to];
        // retoggle piece (as its been replaced by the capture-er)
        self.util[ALL_PIECES] ^= SQUARES[to];
        // toggle pawn off
        self.pieces[self.ctm()] ^= SQUARES[to];
        // toggle promo
        self.pieces[promo_piece] ^= SQUARES[to];

        self.toggle_piece_hash(piece, to);
        self.toggle_piece_hash(promo_piece, to);
        self.toggle_piece_hash(xpiece, to);

        self.remove_piece_value(piece, to);
        self.remove_piece_value(xpiece, to);
        self.add_piece_value(promo_piece, to);

        self.halfmove = 0;
    }

    fn apply_ep(&mut self, to: usize) {
        let ep_sq = to - 8 + (self.ctm() * 16);
        self.pieces[self.opp_ctm()] ^= SQUARES[ep_sq]; // toggle capture pawn off
        self.util[self.opp_ctm()] ^= SQUARES[ep_sq];
        self.util[ALL_PIECES] ^= SQUARES[ep_sq];

        self.toggle_piece_hash(self.opp_ctm(), ep_sq);
        self.remove_piece_value(self.opp_ctm(), ep_sq);

        self.halfmove = 0;
    }

    fn toggle_piece_hash(&mut self, piece: usize, to: usize) {
        self.hash ^= Zorb::piece(piece, to);
    }

    fn add_piece_value(&mut self, piece: usize, sq: usize) {
        let mat = MAT_SCORES[piece];
        let (mg, eg) = PST::pst(piece, sq);

        self.mg_value += mat + mg as i32;
        self.eg_value += mat + eg as i32;
    }

    fn remove_piece_value(&mut self, piece: usize, sq: usize) {
        let mat = MAT_SCORES[piece];
        let (mg, eg) = PST::pst(piece, sq);

        self.mg_value -= mat + mg as i32;
        self.eg_value -= mat + eg as i32;
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        if self.pieces != other.pieces {
            return false;
        }

        if self.util != other.util {
            return false;
        }

        if self.ctm != other.ctm {
            return false;
        }

        if self.castle_state != other.castle_state {
            return false;
        }

        if self.ep != other.ep {
            return false;
        }

        if self.hash != other.hash {
            return false;
        }

        if self.mg_value != other.mg_value || self.eg_value != other.eg_value {
            return false;
        }

        true
    }
}

const SQ_PIECES: [&str; 12] = [
    "P ", "p ", "N ", "n ", "R ", "r ", "B ", "b ", "Q ", "q ", "K ", "k ",
];

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let add_sq = |s, sq| {
            format!(
                "{s}{}",
                get_piece(self, sq)
                    .or(get_xpiece(self, sq))
                    .map_or("- ", |piece| SQ_PIECES[piece as usize])
            )
        };

        // iterate over every row (56-63, 48-55, ... 0-7) and concat the pieces of that row to the out string
        let mut out = (0..8)
            .rev()
            .map(|i| (i + 1, (i * 8..i * 8 + 8)))
            .fold(String::new(), |out, (row_num, row)| {
                format!("{out}\n{row_num}   {}", row.fold(String::new(), add_sq))
            });

        out.push_str("\n\n    A B C D E F G H\n");
        write!(f, "{}", out)
    }
}

pub const WKS_STATE: usize = 0;
pub const WQS_STATE: usize = 1;
pub const BKS_STATE: usize = 2;
pub const BQS_STATE: usize = 3;

static mut ZORB_ARR: [u64; 781] = [0; 781];

const SEED: u64 = 7252092290252765432;

const fn xorshift(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

const fn gen_zorb() -> [u64; 781] {
    let mut zorb = [0; 781];

    let mut rand = SEED;
    let mut i = 0;
    while i < 781 {
        rand = xorshift(rand);
        zorb[i] = rand;
        i += 1;
    }

    zorb
}

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
pub struct Zorb;

impl Index<usize> for Zorb {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &ZORB_ARR[index] }
    }
}

impl Zorb {
    pub fn init() {
        unsafe {
            ZORB_ARR = gen_zorb();
        }
    }

    #[inline]
    pub fn piece(piece: usize, sq: usize) -> u64 {
        Zorb[piece * 64 + sq]
    }

    #[inline]
    pub fn colour() -> u64 {
        Zorb[768]
    }

    #[inline]
    pub fn castle_rights(idx: usize) -> u64 {
        Zorb[769 + idx]
    }

    #[inline]
    pub fn ep_file(sq: usize) -> u64 {
        Zorb[773 + (sq % 8)]
    }
}

pub fn gen_hash(board: Board) -> u64 {
    let mut hash = 0;

    for piece in 0..12 {
        for (i, sq) in SQUARES.iter().enumerate().take(64) {
            if (board.pieces[piece] & sq) > 0 {
                hash ^= Zorb::piece(piece, i);
            }
        }
    }

    // if black to move toggle zorb
    if board.ctm == 1 {
        hash ^= Zorb::colour();
    }
    if (board.castle_state & 0b1000) == 8 {
        hash ^= Zorb::castle_rights(WKS_STATE);
    }
    if (board.castle_state & 0b100) == 4 {
        hash ^= Zorb::castle_rights(WQS_STATE);
    }
    if (board.castle_state & 0b10) == 2 {
        hash ^= Zorb::castle_rights(BKS_STATE);
    }
    if (board.castle_state & 0b1) == 1 {
        hash ^= Zorb::castle_rights(BQS_STATE);
    }
    if board.ep < 64 {
        hash ^= Zorb::ep_file(board.ep as usize);
    }

    hash
}

// macro to print a list of bitboards (u64s) one after each other, v similar to dbg!() but only for bbs
#[macro_export]
macro_rules! print_bb {
    ( $( $args:expr ),* ) => {
        {
            $( $crate::board::_print_bb($args); )*
        }
    };
}

pub fn _print_bb(bb: u64) {
    let mut out = String::new();

    for i in (1..9).rev() {
        out.push_str(&i.to_string());
        out.push(' ');

        for sq in SQUARES.iter().skip(i * 8 - 8).take(8) {
            if sq & bb > 0 {
                out.push_str(" X ");
            } else {
                out.push_str(" - ");
            }
        }
        out.push('\n');
    }
    out.push_str("   A  B  C  D  E  F  G  H\n");

    println!("{}", out);
}

#[test]
fn inc_value_update() {
    use crate::eval::{gen_mat_value, gen_pst_value};
    crate::init();
    let board =
        Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();

    let quiet_move = Move::new(0, 1, ROOK as u32, 0, crate::moves::MoveType::Quiet);
    let quiet_board = board.copy_make(quiet_move);

    let mat = gen_mat_value(&quiet_board);
    let (mg, eg) = gen_pst_value(&quiet_board);
    let mg_quiet = mat + mg;
    let eg_quiet = mat + eg;

    assert_eq!(quiet_board.mg_value, mg_quiet);
    assert_eq!(quiet_board.eg_value, eg_quiet);

    let cap_move = Move::new(25, 32, BISHOP as u32, KING as u32 + 1, MoveType::Cap);
    let cap_board = board.copy_make(cap_move);

    let mat = gen_mat_value(&cap_board);
    let (mg, eg) = gen_pst_value(&cap_board);
    let mg_cap = mat + mg;
    let eg_cap = mat + eg;

    assert_eq!(cap_board.mg_value, mg_cap);
    assert_eq!(cap_board.eg_value, eg_cap);
}

#[test]
fn test_inc_values_and_hash_copy_make() {
    crate::init();
    use crate::board::*;

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

#[test]
fn add_rm_piece_value() {
    crate::init();

    let mut b = Board::new();
    assert_eq!(b.mg_value, 0);
    assert_eq!(b.eg_value, 0);

    // a2a3
    b.remove_piece_value(0, 8);
    assert_eq!(b.mg_value, 0 - 100 - -35);
    assert_eq!(b.eg_value, 0 - 100 - 13);

    b.add_piece_value(0, 16);
    assert_eq!(b.mg_value, 0 - 100 - -35 + 100 + -26);
    assert_eq!(b.eg_value, 0 - 100 - 13 + 100 + 4);
}
