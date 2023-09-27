use std::sync::OnceLock;

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

static mut ZORB_ARR: &'static mut [u64] = &mut [0];

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
pub struct Zorb;

impl Zorb {
    pub fn init() {
        let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);
        let zorb: Vec<u64> = (0..781).map(|_| prng.gen()).collect();
        unsafe { ZORB_ARR = zorb.leak() }
    }

    #[inline]
    pub fn piece(piece: usize, sq: usize) -> u64 {
        unsafe { ZORB_ARR[piece * 64 + sq] }
    }

    #[inline]
    pub fn colour() -> u64 {
        unsafe { ZORB_ARR[768] }
    }

    #[inline]
    pub fn castle_rights(idx: usize) -> u64 {
        unsafe { ZORB_ARR[769 + idx] }
    }

    #[inline]
    pub fn ep_file(sq: usize) -> u64 {
        unsafe { ZORB_ARR[773 + (sq % 8)] }
    }

    pub fn print_zorb() {
        println!("pub const ZORB: [u64; 781] = [");
        unsafe {
            ZORB_ARR.iter().for_each(|z| println!("\t{z:#0x},"));
        }
        println!("];");
    }
}
