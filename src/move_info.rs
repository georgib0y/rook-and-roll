use crate::magic::{
    batt, index_to_u64, ratt, BISHOP_MAGIC, BISHOP_MASK, B_BIT, ROOK_MAGIC, ROOK_MASK, R_BIT,
};

const fn generate_squares() -> [u64; 65] {
    let mut squares = [0; 65];
    let mut i = 0;
    while i < 64 {
        squares[i] = 1 << i;
        i += 1;
    }

    squares
}

pub const SQUARES: [u64; 65] = generate_squares();

// file masks
pub const FA: u64 = 0x0101010101010101;
pub const FB: u64 = 0x0202020202020202;
pub const FC: u64 = 0x0404040404040404;
pub const FD: u64 = 0x0808080808080808;
pub const FE: u64 = 0x1010101010101010;
pub const FF: u64 = 0x2020202020202020;
pub const FG: u64 = 0x4040404040404040;
pub const FH: u64 = 0x8080808080808080;
pub const FILES: [u64; 8] = [FA, FB, FC, FD, FE, FF, FG, FH];

// rank masks
pub const R1: u64 = 0x00000000000000FF;
pub const R2: u64 = 0x000000000000FF00;
pub const R3: u64 = 0x0000000000FF0000;
pub const R4: u64 = 0x00000000FF000000;
pub const R5: u64 = 0x000000FF00000000;
pub const R6: u64 = 0x0000FF0000000000;
pub const R7: u64 = 0x00FF000000000000;
pub const R8: u64 = 0xFF00000000000000;
pub const RANKS: [u64; 8] = [R1, R2, R3, R4, R5, R6, R7, R8];

pub const SQ_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

pub const UP_LEFT_DIR: usize = 0;
pub const UP_DIR: usize = 1;
pub const UP_RIGHT_DIR: usize = 2;
pub const RIGHT_DIR: usize = 3;
pub const DOWN_RIGHT_DIR: usize = 4;
pub const DOWN_DIR: usize = 5;
pub const DOWN_LEFT_DIR: usize = 6;
pub const LEFT_DIR: usize = 7;

pub struct RAYS;

impl RAYS {
    pub fn init() {
        unsafe {
            RAY_ARRAY = gen_rays();
        }
    }

    pub fn get(dir: usize, sq: usize) -> u64 {
        unsafe { RAY_ARRAY[dir][sq] }
    }
}

pub static mut RAY_ARRAY: [[u64; 65]; 8] = [[0; 65]; 8];

fn gen_rays() -> [[u64; 65]; 8] {
    let (up, down) = gen_vertical();
    let (right, left) = gen_horizontal();
    let (up_right, down_left) = gen_pos_diagonal();
    let (up_left, down_right) = gen_neg_diagonal();

    [
        up_left, up, up_right, right, down_right, down, down_left, left,
    ]
}

fn get_above_below(ray: u64, sq: u64) -> (u64, u64) {
    (ray & (sq ^ sq.wrapping_neg()), ray & (sq - 1))
}

fn gen_vertical() -> ([u64; 65], [u64; 65]) {
    let mut up = [0; 65];
    let mut down = [0; 65];
    let mut i = 0;
    while i < 64 {
        (up[i], down[i]) = get_above_below(FILES[i % 8], 1 << i);
        i += 1;
    }
    (up, down)
}

fn gen_horizontal() -> ([u64; 65], [u64; 65]) {
    let mut right = [0; 65];
    let mut left = [0; 65];
    let mut i = 0;
    while i < 64 {
        (right[i], left[i]) = get_above_below(RANKS[i / 8], 1 << i);
        i += 1;
    }
    (right, left)
}

// https://www.chessprogramming.org/On_an_empty_Board#Line_Attacks
fn gen_pos_diagonal() -> ([u64; 65], [u64; 65]) {
    const DIAG: u64 = 0x8040201008040201;

    let mut up_right = [0; 65];
    let mut down_left = [0; 65];
    let mut i = 0;
    while i < 64 {
        let sq = 1u64 << i;
        let diag = (i as i64 & 7) - (i as i64 >> 3);

        let diagonal = if diag >= 0 {
            DIAG >> (diag * 8)
        } else {
            DIAG << (diag.wrapping_neg() * 8)
        };

        (up_right[i], down_left[i]) = get_above_below(diagonal, sq);
        i += 1;
    }
    (up_right, down_left)
}

fn gen_neg_diagonal() -> ([u64; 65], [u64; 65]) {
    const DIAG: u64 = 0x0102040810204080;

    let mut up_left = [0; 65];
    let mut down_right = [0; 65];
    let mut i = 0;
    while i < 64 {
        let sq = 1u64 << i;
        let diag = 7 - (i as i64 & 7) - (i as i64 >> 3);

        let diagonal = if diag >= 0 {
            DIAG >> (diag * 8)
        } else {
            DIAG << (diag.wrapping_neg() * 8)
        };

        (up_left[i], down_right[i]) = get_above_below(diagonal, sq);
        i += 1;
    }
    (up_left, down_right)
}

// all PST are considered from whites perspective
pub struct PST;

impl PST {
    pub fn init() {
        unsafe {
            MID_PST = [
                WPAWN_MID_PST,
                flip_pst(WPAWN_MID_PST),
                WKNIGHT_MID_PST,
                flip_pst(WKNIGHT_MID_PST),
                WROOK_MID_PST,
                flip_pst(WROOK_MID_PST),
                WBISHOP_MID_PST,
                flip_pst(WBISHOP_MID_PST),
                WQUEEN_MID_PST,
                flip_pst(WQUEEN_MID_PST),
                WKING_MID_PST,
                flip_pst(WKING_MID_PST),
            ];

            END_PST = [
                WPAWN_END_PST,
                flip_pst(WPAWN_END_PST),
                WKNIGHT_END_PST,
                flip_pst(WKNIGHT_END_PST),
                WROOK_END_PST,
                flip_pst(WROOK_END_PST),
                WBISHOP_END_PST,
                flip_pst(WBISHOP_END_PST),
                WQUEEN_END_PST,
                flip_pst(WQUEEN_END_PST),
                WKING_END_PST,
                flip_pst(WKING_END_PST),
            ]
        }
    }

    pub fn pst(piece: usize, sq: usize) -> (i16, i16) {
        unsafe { (MID_PST[piece][sq], END_PST[piece][sq]) }
    }
}

static mut MID_PST: [[i16; 64]; 12] = [[0; 64]; 12];

static mut END_PST: [[i16; 64]; 12] = [[0; 64]; 12];

fn flip_pst(wpst: [i16; 64]) -> [i16; 64] {
    let mut bpst = [0; 64];

    let mut i = 0;
    while i < wpst.len() {
        let i_rank = i / 8;
        let i_file = i % 8;

        let idx = (7 - i_rank) * 8 + i_file;

        bpst[idx] = -wpst[i];

        i += 1;
    }

    bpst
}

const WPAWN_MID_PST: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -35, -1, -20, -23, -15, 24, 38, -22, -26, -4, -4, -10, 3, 3, 33, -12,
    -27, -2, -5, 12, 17, 6, 10, -25, -14, 13, 6, 21, 23, 12, 17, -23, -6, 7, 26, 31, 65, 56, 25,
    -20, 98, 134, 61, 95, 68, 126, 34, -11, 0, 0, 0, 0, 0, 0, 0, 0,
];

const WPAWN_END_PST: [i16; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 13, 8, 8, 10, 13, 0, 2, -7, 4, 7, -6, 1, 0, -5, -1, -8, 13, 9, -3, -7,
    -7, -8, 3, -1, 32, 24, 13, 5, -2, 4, 17, 17, 94, 100, 85, 67, 56, 53, 82, 84, 178, 173, 158,
    134, 147, 132, 165, 187, 0, 0, 0, 0, 0, 0, 0, 0,
];

const WKNIGHT_MID_PST: [i16; 64] = [
    -105, -21, -58, -33, -17, -28, -19, -23, -29, -53, -12, -3, -1, 18, -14, -19, -23, -9, 12, 10,
    19, 17, 25, -16, -13, 4, 16, 13, 28, 19, 21, -8, -9, 17, 19, 53, 37, 69, 18, 22, -47, 60, 37,
    65, 84, 129, 73, 44, -73, -41, 72, 36, 23, 62, 7, -17, -167, -89, -34, -49, 61, -97, -15, -107,
];

const WKNIGHT_END_PST: [i16; 64] = [
    -29, -51, -23, -15, -22, -18, -50, -64, -42, -20, -10, -5, -2, -20, -23, -44, -23, -3, -1, 15,
    10, -3, -20, -22, -18, -6, 16, 25, 16, 17, 4, -18, -17, 3, 22, 22, 22, 11, 8, -18, -24, -20,
    10, 9, -1, -9, -19, -41, -25, -8, -25, -2, -9, -25, -24, -52, -58, -38, -13, -28, -31, -27,
    -63, -99,
];

const WBISHOP_MID_PST: [i16; 64] = [
    -33, -3, -14, -21, -13, -12, -39, -21, 4, 15, 16, 0, 7, 21, 33, 1, 0, 15, 15, 15, 14, 27, 18,
    10, -6, 13, 13, 26, 34, 12, 10, 4, -4, 5, 19, 50, 37, 37, 7, -2, -16, 37, 43, 40, 35, 50, 37,
    -2, -26, 16, -18, -13, 30, 59, 18, -47, -29, 4, -82, -37, -25, -42, 7, -8,
];

const WBISHOP_END_PST: [i16; 64] = [
    -23, -9, -23, -5, -9, -16, -5, -17, -14, -18, -7, -1, 4, -9, -15, -27, -12, -3, 8, 10, 13, 3,
    -7, -15, -6, 3, 13, 19, 7, 10, -3, -9, -3, 9, 12, 9, 14, 10, 3, 2, 2, -8, 0, -1, -2, 6, 0, 4,
    -8, -4, 7, -12, -3, -13, -4, -14, -14, -21, -11, -8, -7, -9, -17, -24,
];

const WROOK_MID_PST: [i16; 64] = [
    -19, -13, 1, 17, 16, 7, -37, -26, -44, -16, -20, -9, -1, 11, -6, -71, -45, -25, -16, -17, 3, 0,
    -5, -33, -36, -26, -12, -1, 9, -7, 6, -23, -24, -11, 7, 26, 24, 35, -8, -20, -5, 19, 26, 36,
    17, 45, 61, 16, 27, 32, 58, 62, 80, 67, 26, 44, 32, 42, 32, 51, 63, 9, 31, 43,
];

const WROOK_END_PST: [i16; 64] = [
    -9, 2, 3, -1, -5, -13, 4, -20, -6, -6, 0, 2, -9, -9, -11, -3, -4, 0, -5, -1, -7, -12, -8, -16,
    3, 5, 8, 4, -5, -6, -8, -11, 4, 3, 13, 1, 2, 1, -1, 2, 7, 7, 7, 5, 4, -3, -5, -3, 11, 13, 13,
    11, -3, 3, 8, 3, 13, 10, 18, 15, 12, 12, 8, 5,
];

const WQUEEN_MID_PST: [i16; 64] = [
    -1, -18, -9, 10, -15, -25, -31, -50, -35, -8, 11, 2, 8, 15, -3, 1, -14, 2, -11, -2, -5, 2, 14,
    5, -9, -26, -9, -10, -2, -4, 3, -3, -27, -27, -16, -16, -1, 17, -2, 1, -13, -17, 7, 8, 29, 56,
    47, 57, -24, -39, -5, 1, -16, 57, 28, 54, -28, 0, 29, 12, 59, 44, 43, 45,
];

const WQUEEN_END_PST: [i16; 64] = [
    -33, -28, -22, -43, -5, -32, -20, -41, -22, -23, -30, -16, -16, -23, -36, -32, -16, -27, 15, 6,
    9, 17, 10, 5, -18, 28, 19, 47, 31, 34, 39, 23, 3, 22, 24, 45, 57, 40, 57, 36, -20, 6, 9, 49,
    47, 35, 19, 9, -17, 20, 32, 41, 58, 25, 30, 0, -9, 22, 22, 27, 27, 19, 10, 20,
];

const WKING_MID_PST: [i16; 64] = [
    -15, 36, 12, -54, 8, -28, 24, 14, 1, 7, -8, -64, -43, -16, 9, 8, -14, -14, -22, -46, -44, -30,
    -15, -27, -49, -1, -27, -39, -46, -44, -33, -51, -17, -20, -12, -27, -30, -25, -14, -36, -9,
    24, 2, -16, -20, 6, 22, -22, 29, -1, -20, -7, -8, -4, -38, -29, -65, 23, 16, -15, -56, -34, 2,
    13,
];

const WKING_END_PST: [i16; 64] = [
    -53, -34, -21, -11, -28, -14, -24, -43, -27, -11, 4, 13, 14, 4, -5, -17, -19, -3, 11, 21, 23,
    16, 7, -9, -18, -4, 21, 24, 27, 23, 9, -11, -8, 22, 24, 27, 26, 33, 26, 3, 10, 17, 23, 15, 20,
    45, 44, 13, -12, 17, 14, 17, 17, 38, 23, 11, -74, -35, -18, -18, -11, 15, 4, -17,
];

static mut PAWN_ATTACKS: [u64; 128] = [0; 128];
static mut KNIGHT_MOVES: [u64; 64] = [0; 64];
static mut KING_MOVES: [u64; 64] = [0; 64];
static mut SUPERRAYS: [u64; 64] = [0; 64];
static mut ROOK_MOVES: [[u64; 4096]; 64] = [[0; 4096]; 64];
static mut BISHOP_MOVES: [[u64; 512]; 64] = [[0; 512]; 64];

pub struct MT;

impl MT {
    pub fn init() {
        unsafe {
            PAWN_ATTACKS = gen_pawn_attack_table();
            KNIGHT_MOVES = gen_knight_move_table();
            KING_MOVES = gen_king_move_table();
            SUPERRAYS = gen_superray();
            gen_rook_move_table();
            gen_bishop_move_table();
        }
    }

    #[inline]
    pub fn pawn_attacks(colour: usize, sq: usize) -> u64 {
        unsafe { PAWN_ATTACKS[sq + colour * 64] }
    }

    #[inline]
    pub fn knight_moves(sq: usize) -> u64 {
        unsafe { KNIGHT_MOVES[sq] }
    }

    #[inline]
    pub fn king_moves(sq: usize) -> u64 {
        unsafe { KING_MOVES[sq] }
    }

    #[inline]
    pub fn rook_moves(mut occ: u64, sq: usize) -> u64 {
        occ &= ROOK_MASK[sq];
        occ = occ.wrapping_mul(ROOK_MAGIC[sq]);
        occ >>= 64 - R_BIT;
        unsafe { ROOK_MOVES[sq][occ as usize] }
    }

    #[inline]
    pub fn rook_xray_moves(occ: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = Self::rook_moves(occ, sq);
        blockers &= attacks;
        attacks ^ Self::rook_moves(occ ^ blockers, sq)
    }

    #[inline]
    pub fn bishop_moves(mut occ: u64, sq: usize) -> u64 {
        occ &= BISHOP_MASK[sq];
        occ = occ.wrapping_mul(BISHOP_MAGIC[sq]);
        occ >>= 64 - B_BIT;
        unsafe { BISHOP_MOVES[sq][occ as usize] }
    }

    #[inline]
    pub fn bishop_xray_moves(occ: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = Self::bishop_moves(occ, sq);
        blockers &= attacks;
        attacks ^ Self::bishop_moves(occ ^ blockers, sq)
    }

    #[inline]
    pub fn rays(dir: usize, sq: usize) -> u64 {
        RAYS::get(dir, sq)
    }

    #[inline]
    pub fn superrays(sq: usize) -> u64 {
        unsafe { *SUPERRAYS.get_unchecked(sq) }
    }
}

fn gen_pawn_attack_table() -> [u64; 128] {
    let mut pawn_attacks = [0; 64 * 2];

    let mut i = 0;
    while i < 64 {
        let sq = SQUARES[i];
        //white
        if sq & !R8 > 0 {
            pawn_attacks[i] = (sq & !FA) << 7 | (sq & !FH) << 9;
        }

        //black
        if sq & !R1 > 0 {
            pawn_attacks[i + 64] = (sq & !FH) >> 7 | (sq & !FA) >> 9;
        }

        i += 1;
    }

    pawn_attacks
}

fn gen_knight_move_table() -> [u64; 64] {
    let mut knight_moves = [0; 64];

    let mut i = 0;
    while i < 64 {
        let mut mv = 0;
        mv |= (SQUARES[i] & !FA & !FB) << 6;
        mv |= (SQUARES[i] & !FA) << 15;
        mv |= (SQUARES[i] & !FH) << 17;
        mv |= (SQUARES[i] & !FG & !FH) << 10;

        mv |= (SQUARES[i] & !FH & !FG) >> 6;
        mv |= (SQUARES[i] & !FH) >> 15;
        mv |= (SQUARES[i] & !FA) >> 17;
        mv |= (SQUARES[i] & !FA & !FB) >> 10;

        knight_moves[i] = mv;
        i += 1;
    }

    knight_moves
}

fn gen_king_move_table() -> [u64; 64] {
    let mut king_moves = [0; 64];

    let mut i = 0;
    while i < 64 {
        let mut mv = 0;
        let k_clear_a = SQUARES[i] & !FA;
        let k_clear_h = SQUARES[i] & !FH;

        mv |= SQUARES[i] << 8;
        mv |= SQUARES[i] >> 8;
        mv |= k_clear_a << 7;
        mv |= k_clear_a >> 1;
        mv |= k_clear_a >> 9;
        mv |= k_clear_h << 9;
        mv |= k_clear_h << 1;
        mv |= k_clear_h >> 7;

        king_moves[i] = mv;
        i += 1;
    }

    king_moves
}

fn gen_rook_move_table() {
    let mut sq = 0;
    while sq < 64 {
        let mut blocker_idx = 0;
        while blocker_idx < (1 << R_BIT) {
            // add rook moves
            let blockers = index_to_u64(
                blocker_idx,
                ROOK_MASK[sq].count_ones() as i32,
                ROOK_MASK[sq],
            );

            unsafe {
                ROOK_MOVES[sq]
                    [((blockers.wrapping_mul(ROOK_MAGIC[sq])) >> (64 - R_BIT)) as usize] =
                    ratt(sq as i32, blockers);
            }

            blocker_idx += 1;
        }
        sq += 1;
    }
}

fn gen_bishop_move_table() {
    let mut sq = 0;
    while sq < 64 {
        let mut blocker_idx = 0;
        while blocker_idx < (1 << B_BIT) {
            let blockers = index_to_u64(
                blocker_idx,
                BISHOP_MASK[sq].count_ones() as i32,
                BISHOP_MASK[sq],
            );
            unsafe {
                BISHOP_MOVES[sq]
                    [((blockers.wrapping_mul(BISHOP_MAGIC[sq])) >> (64 - B_BIT)) as usize] =
                    batt(sq as i32, blockers);
            }
            blocker_idx += 1;
        }
        sq += 1;
    }
}

fn gen_superray() -> [u64; 64] {
    let mut superray = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        superray[sq] = RAYS::get(0, sq)
            | RAYS::get(1, sq)
            | RAYS::get(2, sq)
            | RAYS::get(3, sq)
            | RAYS::get(4, sq)
            | RAYS::get(5, sq)
            | RAYS::get(6, sq)
            | RAYS::get(7, sq);
        sq += 1;
    }

    superray
}
