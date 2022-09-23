use lazy_static::lazy_static;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

lazy_static!{
    pub static ref ZORB_STRUCT: Box<[u64]> = init_zorbist_array();
    pub static ref ZORB: Zorb = Zorb::new();
}

// idea is that ive read that a struct of arrays is better than an array of structs
// so maybe it would be more efficient to split the zorbist array into many parts -v-
// this is why im trying it anyways

pub struct Zorb {
    pieces: [[u64;64]; 12],
    colour: u64,
    castle_rights: [u64;4],
    ep_file: [u64;8],
}

impl Zorb {
    pub fn new() -> Zorb {
        let mut rand = ChaCha20Rng::seed_from_u64(72520922902527);

        let wp= gen_piece_array(&mut rand);
        let bp= gen_piece_array(&mut rand);
        let wn= gen_piece_array(&mut rand);
        let bn= gen_piece_array(&mut rand);
        let wr= gen_piece_array(&mut rand);
        let br= gen_piece_array(&mut rand);
        let wb= gen_piece_array(&mut rand);
        let bb= gen_piece_array(&mut rand);
        let wq= gen_piece_array(&mut rand);
        let bq= gen_piece_array(&mut rand);
        let wk= gen_piece_array(&mut rand);
        let bk= gen_piece_array(&mut rand);
        let pieces = [wp, bp, wn, bn, wr, br, wb, bb, wq, bq, wk, bk];

        Zorb {
            pieces,
            colour: rand.gen::<u64>(),
            castle_rights: gen_castle_array(&mut rand),
            ep_file: gen_ep_file_array(&mut rand),
        }
    }

    #[inline]
    pub fn piece(&self, piece: usize, sq: usize) -> u64 {
        self.pieces[piece][sq]
    }

    #[inline]
    pub fn colour(&self) -> u64 {
        self.colour
    }

    #[inline]
    pub fn castle_rights(&self, idx: usize) -> u64 {
        self.castle_rights[idx]
    }

    #[inline]
    pub fn ep_file(&self, sq: usize) -> u64 {
        self.ep_file[sq%8]
    }

}

fn gen_piece_array(rand: &mut ChaCha20Rng) -> [u64;64] {
    let mut rand_array = [0;64];
    for i in 0..64 {
        rand_array[i] = rand.gen::<u64>();
    }
    rand_array
}

fn gen_castle_array(rand: &mut ChaCha20Rng) -> [u64;4] {
    let mut rand_array = [0;4];
    for i in 0..4 {
        rand_array[i] = rand.gen::<u64>();
    }
    rand_array
}

fn gen_ep_file_array(rand: &mut ChaCha20Rng) -> [u64;8] {
    let mut rand_array = [0; 8];
    for i in 0..8 {
        rand_array[i] = rand.gen::<u64>();
    }
    rand_array
}

// zorbist array indexing:
// 0-767: piece positions, 768: colour, 769-772: castle rights, 773-780: file of ep square
fn init_zorbist_array() -> Box<[u64]> {
    let mut zorbist_array: [u64; 781] = [0; 781];

    // may be a good seed or may not be (could try flipping the reverse around if not)
    let mut prng = ChaCha20Rng::seed_from_u64(72520922902527);

    for z in &mut zorbist_array  {
        *z = prng.gen::<u64>();
    }


    Box::new(zorbist_array)
}
