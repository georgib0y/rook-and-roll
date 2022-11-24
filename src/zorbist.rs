use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

pub static mut ZORB_ARR: [u64; 781] = [0; 781];

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
pub struct Zorb;

impl Zorb {
    pub fn init() {
        let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);
        unsafe { ZORB_ARR.iter_mut().for_each(|z| *z = prng.gen::<u64>() ) }
    }

    #[inline]
    pub fn piece(piece: usize, sq: usize) -> u64 { unsafe { ZORB_ARR[piece*64+sq] } }

    #[inline]
    pub fn colour() -> u64 { unsafe { ZORB_ARR[768] } }

    #[inline]
    pub fn castle_rights(idx: usize) -> u64 { unsafe { ZORB_ARR[769+idx] } }

    #[inline]
    pub fn ep_file(sq: usize) -> u64 { unsafe { ZORB_ARR[773 + (sq%8)] } }
}

