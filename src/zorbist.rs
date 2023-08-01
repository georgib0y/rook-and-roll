use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

static mut ZORB_ARR: Vec<u64> = Vec::new();

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
pub struct Zorb;

impl Zorb {
    pub unsafe fn init() {
        let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);
        (0..781).for_each(|_| ZORB_ARR.push(prng.gen()));
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
        unsafe { ZORB_ARR.iter().for_each(|z| println!("\t{z:#0x},")) }
        println!("];");
    }
}
