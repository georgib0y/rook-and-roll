#![allow(unused)]

use std::sync::Arc;
use rand::prelude::*;
use crate::board::Board;
use crate::move_info::{BISHOP_MAGIC, BISHOP_MASK, FA, FB, FG, FH, R1, R3, R6, R8, RAYS, ROOK_MAGIC, ROOK_MASK, SQUARES};
use crate::moves::Move;
use crate::print_bb;

pub struct MoveTables {
    pub pawn_moves: [[u64;64];2],
    pub pawn_attacks: [[u64;64];2],
    pub knight_moves: [u64;64],
    pub king_moves: [u64;64],
    pub rook_moves: [[u64; 4096]; 64],
    pub bishop_moves: [[u64; 512]; 64],
    pub rays: &'static [[u64; 65]; 8],
}

impl MoveTables {
    pub fn new() -> MoveTables {
        MoveTables {
            pawn_moves: gen_pawn_move_table(),
            pawn_attacks: gen_pawn_attack_table(),
            knight_moves: gen_knight_move_table(),
            king_moves: gen_king_move_table(),
            rook_moves: gen_rook_move_table(),
            bishop_moves: gen_bishop_move_table(),
            rays: &RAYS
        }
    }

    pub fn new_boxed() -> Box<MoveTables> {
        Box::new(MoveTables::new())
    }
    
    pub fn new_arc() -> Arc<MoveTables> { Arc::new(MoveTables::new()) }

    //noinspection ALL
    // TODO dont get the shiftin biz
    pub fn get_rook_moves(&self, mut occupancy: u64, sq: usize) -> u64 {
        occupancy &= ROOK_MASK[sq];
        occupancy = occupancy.wrapping_mul(ROOK_MAGIC[sq]);
        occupancy >>= 64 - R_BIT; // (64-12)
        self.rook_moves[sq][occupancy as usize]
    }

    pub fn get_rook_xray(&self, occupancy: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = self.get_rook_moves(occupancy, sq);
        blockers &= attacks;
        attacks ^ self.get_rook_moves(occupancy ^ blockers, sq)
    }

    pub fn get_bishop_xray(&self, occupancy: u64, mut blockers: u64, sq: usize) -> u64 {
        let attacks = self.get_bishop_moves(occupancy, sq);
        blockers &= attacks;
        attacks ^ self.get_bishop_moves(occupancy ^ blockers, sq)
    }

    // TODO issues probably to do with the shift????
    pub fn get_bishop_moves(&self, mut occupancy: u64, sq: usize) -> u64 {
        occupancy &= BISHOP_MASK[sq];
        occupancy = occupancy.wrapping_mul(BISHOP_MAGIC[sq]);
        occupancy >>= 64 - B_BIT;
        self.bishop_moves[sq][occupancy as usize]
    }

    pub fn superray(&self, sq: usize) -> u64 {
        self.rays[0][sq] | self.rays[1][sq] | self.rays[2][sq] | self.rays[3][sq] |
            self.rays[4][sq] | self.rays[5][sq] | self.rays[6][sq] | self.rays[7][sq]
    }
}

fn gen_pawn_move_table() -> [[u64;64];2] {
    let mut pawn_moves = [[0;64];2];

    for (i, sq) in SQUARES.iter().enumerate().take(64) {
        // white pawns
        if sq & R3 > 0 {
            pawn_moves[0][i] = sq << 8 | sq << 16
        } else {
            pawn_moves[0][i] = (sq & !R8) << 8
        }

        // black pawns
        if sq & R6 > 0 {
            pawn_moves[1][i] = sq >> 8 | sq >> 16;
        } else {
            pawn_moves[1][i] = (sq & !R1) >> 8;
        }
    }

    pawn_moves
}

fn gen_pawn_attack_table() -> [[u64;64];2] {
    let mut pawn_attacks = [[0;64];2];

    for (i, sq) in SQUARES.iter().enumerate().take(64) {
        //white
        if sq & !R8 > 0 {
            pawn_attacks[0][i] = (sq & !FA) << 7 | (sq & !FH) << 9;
        }

        //black
        if sq & !R1 > 0 {
            pawn_attacks[1][i] = (sq & !FH) >> 7 | (sq & !FA) >> 9;
        }
    }

    pawn_attacks
}

fn gen_knight_move_table() -> [u64;64] {
    let mut knight_moves = [0;64];

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

fn gen_king_move_table() -> [u64;64] {
    let mut king_moves = [0;64];

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

fn gen_rook_move_table() -> [[u64; 4096]; 64] {
    let mut rook_moves = [[0;4096];64];
    for sq in 0..64 {
        for blocker_idx in 0..(1 << R_BIT) {
            // add rook moves
            let blockers = index_to_u64(blocker_idx,
                                        ROOK_MASK[sq].count_ones() as i32,
                                        ROOK_MASK[sq]);

            rook_moves[sq][((blockers.wrapping_mul(ROOK_MAGIC[sq])) >> (64 - R_BIT)) as usize] =
                ratt(sq as i32, blockers);
        }
    }

    rook_moves
}

fn gen_bishop_move_table() -> [[u64; 512]; 64] {
    let mut bishop_moves = [[0;512];64];
    for sq in 0..64 {
        for blocker_idx in 0..(1<<B_BIT) {
            let blockers = index_to_u64(blocker_idx,
                                        BISHOP_MASK[sq].count_ones() as i32,
                                        BISHOP_MASK[sq]);

            bishop_moves[sq][((blockers.wrapping_mul(BISHOP_MAGIC[sq])) >> (64 - B_BIT)) as usize] =
                batt(sq as i32, blockers)
        }
    }

    bishop_moves
}

fn rand_few_bit_u64() -> u64 {
    let mut rng = rand::thread_rng();
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
63, 30, 3, 32, 25, 41, 22, 33, 15, 50, 42, 13, 11, 53, 19, 34, 61, 29, 2,
51, 21, 43, 45, 10, 18, 47, 1, 54, 9, 57, 0, 35, 62, 31, 40, 4, 49, 5, 52,
26, 60, 6, 23, 44, 46, 27, 56, 16, 7, 39, 48, 24, 59, 14, 12, 55, 38, 28,
58, 20, 37, 17, 36, 8
];

fn pop_1st_bit(bb: &mut u64) -> i32 {
    let b = *bb ^ (*bb - 1);
    let fold: u32 = ((b & 0xffffffff) ^ (b >> 32)) as u32;
    *bb &= (*bb - 1);
    BIT_TABLE[ (fold.wrapping_mul(0x783a9b23) >> 26 ) as usize]
}

fn index_to_u64 (index: i32, bits: i32, mut mask: u64) -> u64 {
    let mut result: u64 = 0;
    for i in 0..bits {
        let j = pop_1st_bit(&mut mask);
        if index & (1 << i) != 0 {
            result |= 1<<j;
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

    let mut r = rank+1;
    while r <= 7 {
        result |= one <<(file + r*8);
        if block & (one << (file + r*8))  != 0 { break; }
        r += 1;
    }


    let mut r = rank - 1;
    while r >= 0 {
        result |= one <<(file + r*8);
        if block & (one << (file + r*8)) != 0 { break; }
        r -= 1;
    }

    let mut f = file + 1;
    while f <= 7 {
        result |= one << (f+ rank*8);
        if block & (one << (f + rank*8)) != 0 { break; }
        f += 1;
    }

    let mut f = file - 1;
    while f >= 0 {
        result |= one << (f+ rank*8);
        if block & (one << (f + rank*8))  != 0 { break; }
        f -= 1;
    }

    result
}

pub fn batt(sq: i32, block: u64) -> u64 {
    let mut result: u64 = 0;
    let rank = sq / 8;
    let file = sq % 8;

    let mut r = rank+1;
    let mut f = file+1;
    while r <= 7 && f <= 7 {
        result |= 1 << (f + r*8);
        if block & (1 << (f + r*8)) != 0 { break; }
        r+=1;
        f+=1;
    }

    let mut r = rank+1;
    let mut f = file-1;
    while r <= 7 && f >= 0 {
        result |= 1 << (f + r*8);
        if block & (1 << (f + r*8)) != 0 { break; }
        r+=1;
        f-=1;
    }

    let mut r = rank-1;
    let mut f = file+1;
    while r >= 0 && f <= 7 {
        result |= 1 << (f + r*8);
        if block & (1 << (f + r*8)) != 0 { break; }
        r-=1;
        f+=1;
    }

    let mut r = rank-1;
    let mut f = file-1;
    while r >= 0 && f >= 0 {
        result |= 1 << (f + r*8);
        if block & (1 << (f + r*8)) != 0 { break; }
        r-=1;
        f-=1;
    }

    result
}

fn transform(b: u64, magic: u64, bits: i32) -> usize {
    (b.wrapping_mul(magic) >> (64 - bits) ) as usize
}

pub fn find_magic(sq: i32, m: i32, bishop: bool) -> u64 {
    let mut a: [u64; 4096] = [0; 4096];
    let mut b: [u64; 4096] = [0; 4096];
    let mut used: [u64; 4096] = [0; 4096];

    let mask = if bishop { BISHOP_MASK[sq as usize] } else { ROOK_MASK[sq as usize] };

    let n = mask.count_ones();

    for i in 0..(1<<n) {
        b[i] = index_to_u64(i as i32,n as i32, mask);
        a[i] = if bishop { batt(sq, b[i])} else { ratt(sq, b[i]) };
    }

    for _ in 0..100000000 {
        let magic = rand_few_bit_u64();
        if (mask.wrapping_mul(magic) & 0xFF00000000000000 ).count_ones() < 6 { continue; }
        used = [0; 4096];
        let mut fail = false;
        for i in 0..(1<<n) {
            if fail { break; }

            let j = transform(b[i], magic, m);
            if used[j] == 0 {
                used[j] = a[i];
            } else if used[j] != a[i] {
                fail = true;
            }
        }
        if !fail { return magic; }
    }
    println!("Failed");

    0
}

pub fn print_new_magics() {
    println!("pub const ROOK_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        println!("\t{:#X},", find_magic(sq, R_BIT, false));
    }
    println!("];\n");

    println!("pub const BISHOP_MAGIC: [u64; 64] = [");
    for sq in 0..64 {
        println!("\t{:#X},", find_magic(sq, B_BIT, true));
    }
    println!("];");
}

// TODO Uneeeded if going down the fixed shift route
// rbits is now 12
pub const R_BITS: [i32;64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12
];

// bbit is now 9
pub const B_BITS: [i32;64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6
];

pub const R_BIT: i32 = 12;
pub const B_BIT: i32 = 9;

