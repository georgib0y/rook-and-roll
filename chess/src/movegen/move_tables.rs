use crate::movegen::magic::{
    batt, index_to_u64, ratt, BISHOP_MAGIC, BISHOP_MASK, B_BIT, ROOK_MAGIC, ROOK_MASK, R_BIT,
};
use std::sync::OnceLock;

use crate::movegen::move_info::*;

static mut MOVE_TABLE: MoveTables = MoveTables::empty_movetables();

pub struct MT;

impl MT {
    pub fn init() {
        unsafe {
            MOVE_TABLE = MoveTables {
                pawn_attacks: gen_pawn_attack_table(),
                knight_moves: gen_knight_move_table(),
                king_moves: gen_king_move_table(),
                rook_moves: gen_rook_move_table(),
                bishop_moves: gen_bishop_move_table(),
                superrays: gen_superray(),
            }
        }
    }

    pub fn pawn_attacks(colour: usize, sq: usize) -> u64 {
        unsafe { MOVE_TABLE.pawn_attacks[sq + colour * 64] }
    }

    pub fn knight_moves(sq: usize) -> u64 {
        unsafe { MOVE_TABLE.knight_moves[sq] }
    }

    pub fn king_moves(sq: usize) -> u64 {
        unsafe { MOVE_TABLE.king_moves[sq] }
    }

    pub fn rook_moves(occ: u64, sq: usize) -> u64 {
        unsafe { MOVE_TABLE.get_rook_moves(occ, sq) }
    }

    pub fn rook_xray_moves(occ: u64, blockers: u64, sq: usize) -> u64 {
        unsafe { MOVE_TABLE.get_rook_xray(occ, blockers, sq) }
    }

    pub fn bishop_moves(occ: u64, sq: usize) -> u64 {
        unsafe { MOVE_TABLE.get_bishop_moves(occ, sq) }
    }

    pub fn bishop_xray_moves(occ: u64, blockers: u64, sq: usize) -> u64 {
        unsafe { MOVE_TABLE.get_bishop_xray(occ, blockers, sq) }
    }

    pub fn rays(dir: usize, sq: usize) -> u64 {
        RAYS::get(dir, sq)
    }

    pub fn superrays(sq: usize) -> u64 {
        unsafe { MOVE_TABLE.superrays[sq] }
    }
}

#[derive(Clone, Debug)]
pub struct MoveTables {
    pub pawn_attacks: &'static [u64],
    pub knight_moves: &'static [u64],
    pub king_moves: &'static [u64],
    pub rook_moves: &'static [[u64; 4096]],
    pub bishop_moves: &'static [[u64; 512]],
    pub superrays: &'static [u64],
}

impl MoveTables {
    const fn empty_movetables() -> MoveTables {
        MoveTables {
            pawn_attacks: &[0; 64 * 2],
            knight_moves: &[0; 64],
            king_moves: &[0; 64],
            rook_moves: &[],
            bishop_moves: &[],
            superrays: &[0; 64],
        }
    }

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

fn gen_pawn_attack_table() -> &'static [u64] {
    let mut pawn_attacks = vec![0; 64 * 2];

    for (i, sq) in SQUARES.into_iter().enumerate().take(64) {
        //white
        if sq & !R8 > 0 {
            pawn_attacks[i] = (sq & !FA) << 7 | (sq & !FH) << 9;
        }

        //black
        if sq & !R1 > 0 {
            pawn_attacks[i + 64] = (sq & !FH) >> 7 | (sq & !FA) >> 9;
        }
    }

    pawn_attacks.leak()
}

fn gen_knight_move_table() -> &'static [u64] {
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

    knight_moves.leak()
}

fn gen_king_move_table() -> &'static [u64] {
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

    king_moves.leak()
}

fn gen_rook_move_table() -> &'static [[u64; 4096]] {
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

    rook_moves.leak()
}

fn gen_bishop_move_table() -> &'static [[u64; 512]] {
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

    bishop_moves.leak()
}

fn gen_superray() -> &'static [u64] {
    let mut superray = vec![0; 64];
    for (sq, sray) in superray.iter_mut().enumerate().take(64) {
        *sray = RAYS::get(0, sq)
            | RAYS::get(1, sq)
            | RAYS::get(2, sq)
            | RAYS::get(3, sq)
            | RAYS::get(4, sq)
            | RAYS::get(5, sq)
            | RAYS::get(6, sq)
            | RAYS::get(7, sq)
    }

    superray.leak()
}
