
use std::sync::OnceLock;

use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

static ZORB_ARR: OnceLock<Vec<u64>> = OnceLock::new();

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
pub struct Zorb;

impl Zorb {
    pub unsafe fn init() {
        let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);
        let zorb_arr: Vec<u64> = (0..781).map(|_| prng.gen()).collect();
        ZORB_ARR.set(zorb_arr).unwrap();
     }

    #[inline]
    pub fn piece(piece: usize, sq: usize) -> u64 {
        *ZORB_ARR.get().unwrap().get(piece * 64 + sq).unwrap()
    }

    #[inline]
    pub fn colour() -> u64 {
        *ZORB_ARR.get().unwrap().get(768).unwrap()
    }

    #[inline]
    pub fn castle_rights(idx: usize) -> u64 {
        *ZORB_ARR.get().unwrap().get(769 + idx).unwrap()
    }

    #[inline]
    pub fn ep_file(sq: usize) -> u64 {
        *ZORB_ARR.get().unwrap().get(773 + (sq % 8)).unwrap()
    }

    pub fn print_zorb() {
        println!("pub const ZORB: [u64; 781] = [");
        ZORB_ARR.get().unwrap().iter().for_each(|z| println!("\t{z:#0x},"));
        println!("];");
    }
}
