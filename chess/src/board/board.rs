use crate::board::board_copier::{BoardCopier, BoardCopierWithHash};
use crate::board::zorbist::Zorb;
use crate::movegen::move_info::SQUARES;
use crate::movegen::movegen::{get_piece, get_xpiece};
use crate::movegen::moves::Move;
use crate::search::eval::{gen_board_value, gen_mat_value, gen_pst_value};
use std::marker::PhantomData;
use std::{fmt, num};

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
    DEFAULT_PIECES[0]
        | DEFAULT_PIECES[2]
        | DEFAULT_PIECES[4]
        | DEFAULT_PIECES[6]
        | DEFAULT_PIECES[8]
        | DEFAULT_PIECES[10], // white
    DEFAULT_PIECES[1]
        | DEFAULT_PIECES[3]
        | DEFAULT_PIECES[5]
        | DEFAULT_PIECES[7]
        | DEFAULT_PIECES[9]
        | DEFAULT_PIECES[11], // black
    DEFAULT_PIECES[0]
        | DEFAULT_PIECES[2]
        | DEFAULT_PIECES[4]
        | DEFAULT_PIECES[6]
        | DEFAULT_PIECES[8]
        | DEFAULT_PIECES[10]
        | DEFAULT_PIECES[1]
        | DEFAULT_PIECES[3]
        | DEFAULT_PIECES[5]
        | DEFAULT_PIECES[7]
        | DEFAULT_PIECES[9]
        | DEFAULT_PIECES[11], // all
];

type Copier = BoardCopierWithHash;

// 0 - white to move, 1 - black to move
#[derive(Copy, Clone)]
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
        Copier::copy(self, m)
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
            $( crate::board::board::_print_bb($args); )*
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
    crate::init();
    let board =
        Board::new_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();

    let quiet_move = Move::new(0, 1, ROOK as u32, 0, crate::movegen::moves::MoveType::Quiet);
    let quiet_board = board.copy_make(quiet_move);

    let mat = gen_mat_value(&quiet_board);
    let (mg, eg) = gen_pst_value(&quiet_board);
    let mg_quiet = mat + mg;
    let eg_quiet = mat + eg;

    assert_eq!(quiet_board.mg_value, mg_quiet);
    assert_eq!(quiet_board.eg_value, eg_quiet);

    let cap_move = Move::new(
        25,
        32,
        BISHOP as u32,
        KING as u32 + 1,
        crate::movegen::moves::MoveType::Cap,
    );
    let cap_board = board.copy_make(cap_move);

    let mat = gen_mat_value(&cap_board);
    let (mg, eg) = gen_pst_value(&cap_board);
    let mg_cap = mat + mg;
    let eg_cap = mat + eg;

    assert_eq!(cap_board.mg_value, mg_cap);
    assert_eq!(cap_board.eg_value, eg_cap);
}
