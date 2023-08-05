use std::sync::OnceLock;

use crate::move_info::*;
use rand::prelude::*;

pub const R_BIT: i32 = 12;
pub const B_BIT: i32 = 9;

pub static MOVE_TABLE: OnceLock<MoveTables> = OnceLock::new();

pub struct MT;

impl MT {
    pub fn init() {
        init_rays();

        MOVE_TABLE
            .set(MoveTables {
                pawn_attacks: gen_pawn_attack_table(),
                knight_moves: gen_knight_move_table(),
                king_moves: gen_king_move_table(),
                rook_moves: gen_rook_move_table(),
                bishop_moves: gen_bishop_move_table(),
                superrays: gen_superray(),
            })
            .unwrap();
    }

    pub fn pawn_attacks(colour: usize, sq: usize) -> u64 {
        *MOVE_TABLE
            .get()
            .unwrap()
            .pawn_attacks
            .get(sq + colour * 64)
            .unwrap()
    }

    pub fn knight_moves(sq: usize) -> u64 {
        *MOVE_TABLE.get().unwrap().knight_moves.get(sq).unwrap()
    }

    pub fn king_moves(sq: usize) -> u64 {
        *MOVE_TABLE.get().unwrap().king_moves.get(sq).unwrap()
    }

    pub fn rook_moves(occ: u64, sq: usize) -> u64 {
        MOVE_TABLE.get().unwrap().get_rook_moves(occ, sq)
    }

    pub fn rook_xray_moves(occ: u64, blockers: u64, sq: usize) -> u64 {
        MOVE_TABLE.get().unwrap().get_rook_xray(occ, blockers, sq)
    }

    pub fn bishop_moves(occ: u64, sq: usize) -> u64 {
        MOVE_TABLE.get().unwrap().get_bishop_moves(occ, sq)
    }

    pub fn bishop_xray_moves(occ: u64, blockers: u64, sq: usize) -> u64 {
        MOVE_TABLE.get().unwrap().get_bishop_xray(occ, blockers, sq)
    }

    pub fn rays(dir: usize, sq: usize) -> u64 {
        *RAYS.get().unwrap().get(dir).unwrap().get(sq).unwrap()
    }

    pub fn superrays(sq: usize) -> u64 {
        *MOVE_TABLE.get().unwrap().superrays.get(sq).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct MoveTables {
    pub pawn_attacks: Vec<u64>,
    pub knight_moves: Vec<u64>,
    pub king_moves: Vec<u64>,
    pub rook_moves: Vec<[u64; 4096]>,
    pub bishop_moves: Vec<[u64; 512]>,
    pub superrays: Vec<u64>,
}

impl MoveTables {
    #[inline]
    pub fn get_rook_moves(&self, mut occupancy: u64, sq: usize) -> u64 {
        unsafe {
            occupancy &= ROOK_MASK.get_unchecked(sq);
            occupancy = occupancy.wrapping_mul(*ROOK_MAGIC.get_unchecked(sq));
            occupancy >>= 64 - R_BIT;
            *self
                .rook_moves
                .get_unchecked(sq)
                .get_unchecked(occupancy as usize)
        }
    }

    #[inline]
    pub fn get_rook_xray(&self, occupancy: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = self.get_rook_moves(occupancy, sq);
        blockers &= attacks;
        attacks ^ self.get_rook_moves(occupancy ^ blockers, sq)
    }

    #[inline]
    pub fn get_bishop_xray(&self, occupancy: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = self.get_bishop_moves(occupancy, sq);
        blockers &= attacks;
        attacks ^ self.get_bishop_moves(occupancy ^ blockers, sq)
    }

    #[inline]
    pub fn get_bishop_moves(&self, mut occupancy: u64, sq: usize) -> u64 {
        unsafe {
            occupancy &= BISHOP_MASK.get_unchecked(sq);
            occupancy = occupancy.wrapping_mul(*BISHOP_MAGIC.get_unchecked(sq));
            occupancy >>= 64 - B_BIT;
            *self
                .bishop_moves
                .get_unchecked(sq)
                .get_unchecked(occupancy as usize)
        }
    }
}

fn gen_pawn_attack_table() -> Vec<u64> {
    let mut pawn_attacks = vec![0; 64 * 2];

    for (i, sq) in SQUARES.iter().enumerate().take(64) {
        //white
        if sq & !R8 > 0 {
            pawn_attacks[i] = (sq & !FA) << 7 | (sq & !FH) << 9;
        }

        //black
        if sq & !R1 > 0 {
            pawn_attacks[i + 64] = (sq & !FH) >> 7 | (sq & !FA) >> 9;
        }
    }

    pawn_attacks
}

fn gen_knight_move_table() -> Vec<u64> {
    let mut knight_moves = vec![0; 64];

    for index in 0..64 {
        let mut mv = 0;
        mv |= (SQUARES[index] & !FA & !FB) << 6;
        mv |= (SQUARES[index] & !FA) << 15;
        mv |= (SQUARES[index] & !FH) << 17;
        mv |= (SQUARES[index] & !FG & !FH) << 10;

        mv |= (SQUARES[index] & !FH & !FG) >> 6;
        mv |= (SQUARES[index] & !FH) >> 15;
        mv |= (SQUARES[index] & !FA) >> 17;
        mv |= (SQUARES[index] & !FA & !FB) >> 10;

        knight_moves[index] = mv;
    }

    knight_moves
}

fn gen_king_move_table() -> Vec<u64> {
    let mut king_moves = vec![0; 64];

    for index in 0..64 {
        let mut mv = 0;
        let k_clear_a = SQUARES[index] & !FA;
        let k_clear_h = SQUARES[index] & !FH;

        mv |= SQUARES[index] << 8;
        mv |= SQUARES[index] >> 8;
        mv |= k_clear_a << 7;
        mv |= k_clear_a >> 1;
        mv |= k_clear_a >> 9;
        mv |= k_clear_h << 9;
        mv |= k_clear_h << 1;
        mv |= k_clear_h >> 7;

        king_moves[index] = mv;
    }

    king_moves
}

fn gen_rook_move_table() -> Vec<[u64; 4096]> {
    let mut rook_moves = vec![[0; 4096]; 64];
    for sq in 0..64 {
        for blocker_idx in 0..(1 << R_BIT) {
            // add rook moves
            let blockers = index_to_u64(
                blocker_idx,
                ROOK_MASK[sq].count_ones() as i32,
                ROOK_MASK[sq],
            );

            rook_moves[sq][((blockers.wrapping_mul(ROOK_MAGIC[sq])) >> (64 - R_BIT)) as usize] =
                ratt(sq as i32, blockers);
        }
    }

    rook_moves
}

fn gen_bishop_move_table() -> Vec<[u64; 512]> {
    let mut bishop_moves = vec![[0; 512]; 64];
    for sq in 0..64 {
        for blocker_idx in 0..(1 << B_BIT) {
            let blockers = index_to_u64(
                blocker_idx,
                BISHOP_MASK[sq].count_ones() as i32,
                BISHOP_MASK[sq],
            );

            bishop_moves[sq][((blockers.wrapping_mul(BISHOP_MAGIC[sq])) >> (64 - B_BIT)) as usize] =
                batt(sq as i32, blockers)
        }
    }

    bishop_moves
}

fn gen_superray() -> Vec<u64> {
    let mut superray = vec![0; 64];
    for (sq, sray) in superray.iter_mut().enumerate().take(64) {
        let rays = RAYS.get().unwrap();
        *sray = rays[0][sq]
            | rays[1][sq]
            | rays[2][sq]
            | rays[3][sq]
            | rays[4][sq]
            | rays[5][sq]
            | rays[6][sq]
            | rays[7][sq]
    }

    superray
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// FOLLOWING CODE IS FROM https://www.chessprogramming.org/Looking_for_Magics
// only used for generating the magic numbers (not used in actual running)

fn _rand_few_bit_u64() -> u64 {
    let mut rng = thread_rng();
    let count = 3;

    let mut randoms = Vec::with_capacity(count);
    for _ in 0..count {
        let r1: u64 = rng.gen::<u64>() & 0xFFFF;
        let r2: u64 = rng.gen::<u64>() & 0xFFFF;
        let r3: u64 = rng.gen::<u64>() & 0xFFFF;
        let r4: u64 = rng.gen::<u64>() & 0xFFFF;

        randoms.push(r1 | (r2 << 16) | (r3 << 32) | (r4 << 48));
    }

    let mut rand: u64 = 0xFFFFFFFFFFFFFFFF;
    for r in randoms {
        rand &= r;
    }

    rand
}

const BIT_TABLE: [i32; 64] = [
    63, 30, 3, 32, 25, 41, 22, 33, 15, 50, 42, 13, 11, 53, 19, 34, 61, 29, 2, 51, 21, 43, 45, 10,
    18, 47, 1, 54, 9, 57, 0, 35, 62, 31, 40, 4, 49, 5, 52, 26, 60, 6, 23, 44, 46, 27, 56, 16, 7,
    39, 48, 24, 59, 14, 12, 55, 38, 28, 58, 20, 37, 17, 36, 8,
];

fn pop_1st_bit(bb: &mut u64) -> i32 {
    let b = *bb ^ (*bb - 1);
    let fold: u32 = ((b & 0xffffffff) ^ (b >> 32)) as u32;
    *bb &= *bb - 1;
    BIT_TABLE[(fold.wrapping_mul(0x783a9b23) >> 26) as usize]
}

fn index_to_u64(index: i32, bits: i32, mut mask: u64) -> u64 {
    let mut result: u64 = 0;
    for i in 0..bits {
        let j = pop_1st_bit(&mut mask);
        if index & (1 << i) != 0 {
            result |= 1 << j;
        }
    }

    result
}

//noinspection ALL
pub fn ratt(sq: i32, block: u64) -> u64 {
    let mut result: u64 = 0;
    let rank = sq / 8;
    let file = sq % 8;

    // dbg!(rank, file);

    let one: u64 = 1;

    let mut r = rank + 1;
    while r <= 7 {
        result |= one << (file + r * 8);
        if block & (one << (file + r * 8)) != 0 {
            break;
        }
        r += 1;
    }

    let mut r = rank - 1;
    while r >= 0 {
        result |= one << (file + r * 8);
        if block & (one << (file + r * 8)) != 0 {
            break;
        }
        r -= 1;
    }

    let mut f = file + 1;
    while f <= 7 {
        result |= one << (f + rank * 8);
        if block & (one << (f + rank * 8)) != 0 {
            break;
        }
        f += 1;
    }

    let mut f = file - 1;
    while f >= 0 {
        result |= one << (f + rank * 8);
        if block & (one << (f + rank * 8)) != 0 {
            break;
        }
        f -= 1;
    }

    result
}

pub fn batt(sq: i32, block: u64) -> u64 {
    let mut result: u64 = 0;
    let rank = sq / 8;
    let file = sq % 8;

    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 7 && f <= 7 {
        result |= 1 << (f + r * 8);
        if block & (1 << (f + r * 8)) != 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    let mut r = rank + 1;
    let mut f = file - 1;
    while r <= 7 && f >= 0 {
        result |= 1 << (f + r * 8);
        if block & (1 << (f + r * 8)) != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    let mut r = rank - 1;
    let mut f = file + 1;
    while r >= 0 && f <= 7 {
        result |= 1 << (f + r * 8);
        if block & (1 << (f + r * 8)) != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    let mut r = rank - 1;
    let mut f = file - 1;
    while r >= 0 && f >= 0 {
        result |= 1 << (f + r * 8);
        if block & (1 << (f + r * 8)) != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    result
}

fn _transform(b: u64, magic: u64, bits: i32) -> usize {
    (b.wrapping_mul(magic) >> (64 - bits)) as usize
}

pub fn _find_magic(sq: i32, m: i32, bishop: bool) -> u64 {
    let mut a: [u64; 4096] = [0; 4096];
    let mut b: [u64; 4096] = [0; 4096];
    let mut used: [u64; 4096];

    let mask = if bishop {
        BISHOP_MASK[sq as usize]
    } else {
        ROOK_MASK[sq as usize]
    };

    let n = mask.count_ones();

    for i in 0..(1 << n) {
        b[i] = index_to_u64(i as i32, n as i32, mask);
        a[i] = if bishop {
            batt(sq, b[i])
        } else {
            ratt(sq, b[i])
        };
    }

    for _ in 0..100000000 {
        let magic = _rand_few_bit_u64();
        if (mask.wrapping_mul(magic) & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        used = [0; 4096];
        let mut fail = false;
        for i in 0..(1 << n) {
            if fail {
                break;
            }

            let j = _transform(b[i], magic, m);
            if used[j] == 0 {
                used[j] = a[i];
            } else if used[j] != a[i] {
                fail = true;
            }
        }
        if !fail {
            return magic;
        }
    }
    println!("Failed");

    0
}

pub fn _print_new_magics() {
    println!("pub const ROOK_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        println!("\t{:#X},", _find_magic(sq, R_BIT, false));
    }
    println!("];\n");

    println!("pub const BISHOP_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        println!("\t{:#X},", _find_magic(sq, B_BIT, true));
    }
    println!("];");
}