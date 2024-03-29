use rand::{thread_rng, Rng};

pub const R_BIT: i32 = 12;
pub const B_BIT: i32 = 9;

pub const ROOK_MAGIC: [u64; 64] = [
    0x40800022400A1080,
    0x420401001E800,
    0x100402000110005,
    0x4288002010500008,
    0x60400200040001C0,
    0x50001000208C400,
    0x1008240803000840,
    0x2000044018A2201,
    0x70401040042000,
    0x2882030131020803,
    0x4A00100850800,
    0x205400400400840,
    0x3012000401100620,
    0x80104200008404,
    0x148325380100,
    0x8000120222408100,
    0x8484821011400400,
    0x8204044020203000,
    0x88020300A0010004,
    0x4120200102024280,
    0x100200092408044C,
    0x80208014010000C0,
    0x1000820820040,
    0x10600A000401100,
    0x4824080013020,
    0x8010200008844040,
    0x41000424044040,
    0x1C08008012400220,
    0x2200200041200,
    0x1040049088460400,
    0x218C4800412A0,
    0x2009A008004080,
    0x80010200A40808,
    0x2010004801200092,
    0x220B02004040005,
    0xC00080080801000,
    0x3002110400080044,
    0x40002021110C2,
    0x2010081042009104,
    0x460802000480104,
    0x5441020100202800,
    0x800810221160400,
    0x1084200E0008,
    0x10281003010002,
    0x2204004081000800,
    0x1803204140100400,
    0x840B002110024,
    0x201805082220001,
    0x7324118001006208,
    0x1012402001830004,
    0x100E000806002020,
    0xA0201408020200,
    0x110100802110018,
    0x30001800080,
    0x2280005200911080,
    0x101024220108008,
    0x2000800100402011,
    0x11020080400A,
    0x200200044184111A,
    0x68900A0004121036,
    0x600900100380083,
    0x8001000400020481,
    0x60068802491402,
    0x8000010038804402,
];

pub const BISHOP_MAGIC: [u64; 64] = [
    0x2140004101030008,
    0xA30208100100420,
    0x102028202000101,
    0x141104008002500,
    0x6008142001A8002A,
    0x81402400A8300,
    0x20904410420020,
    0x8048108804202010,
    0x8001480520440080,
    0x108920168001080,
    0x10821401002208,
    0x9004100D000,
    0x80A00444804C6010,
    0x8004020200240001,
    0x10000882002A0A48,
    0x2000100220681412,
    0x2240800700410,
    0x38080020401082,
    0x12C0920100410100,
    0x220100404288000,
    0x24009A00850000,
    0x2422000040100180,
    0x322C010022820040,
    0x89040C010040,
    0x400602001022230,
    0x401008000128006C,
    0x421004420080,
    0xA420202008008020,
    0x1010120104000,
    0x8881480000882C0,
    0x860112C112104108,
    0x10A1082042000420,
    0x100248104100684,
    0x214188200A00640,
    0x4881008210820,
    0x2000280800020A00,
    0x40008201610104,
    0x2004093020001220,
    0x81004501000800C,
    0x234841900C081016,
    0x704009221000402,
    0x4540380010000214,
    0x2030082000040,
    0x8050808104093,
    0x101188107464808,
    0x5041020802400802,
    0x4010B44808850040,
    0x10100040088000E0,
    0x84C010108010,
    0x800488140100,
    0x1000028020218440,
    0x5010048A06220000,
    0x8001040812041000,
    0x1840026008109400,
    0x1046002206001882,
    0x20204400D84000,
    0x1270C20060804000,
    0x2000021113042200,
    0x40002412282008A,
    0xC000000041100,
    0x1000200060005104,
    0x1840042164280880,
    0x964AD0002100AA00,
    0x2190900041002410,
];

pub const ROOK_MASK: [u64; 64] = [
    0x101010101017e,
    0x202020202027c,
    0x404040404047a,
    0x8080808080876,
    0x1010101010106e,
    0x2020202020205e,
    0x4040404040403e,
    0x8080808080807e,
    0x1010101017e00,
    0x2020202027c00,
    0x4040404047a00,
    0x8080808087600,
    0x10101010106e00,
    0x20202020205e00,
    0x40404040403e00,
    0x80808080807e00,
    0x10101017e0100,
    0x20202027c0200,
    0x40404047a0400,
    0x8080808760800,
    0x101010106e1000,
    0x202020205e2000,
    0x404040403e4000,
    0x808080807e8000,
    0x101017e010100,
    0x202027c020200,
    0x404047a040400,
    0x8080876080800,
    0x1010106e101000,
    0x2020205e202000,
    0x4040403e404000,
    0x8080807e808000,
    0x1017e01010100,
    0x2027c02020200,
    0x4047a04040400,
    0x8087608080800,
    0x10106e10101000,
    0x20205e20202000,
    0x40403e40404000,
    0x80807e80808000,
    0x17e0101010100,
    0x27c0202020200,
    0x47a0404040400,
    0x8760808080800,
    0x106e1010101000,
    0x205e2020202000,
    0x403e4040404000,
    0x807e8080808000,
    0x7e010101010100,
    0x7c020202020200,
    0x7a040404040400,
    0x76080808080800,
    0x6e101010101000,
    0x5e202020202000,
    0x3e404040404000,
    0x7e808080808000,
    0x7e01010101010100,
    0x7c02020202020200,
    0x7a04040404040400,
    0x7608080808080800,
    0x6e10101010101000,
    0x5e20202020202000,
    0x3e40404040404000,
    0x7e80808080808000,
];

pub const BISHOP_MASK: [u64; 64] = [
    0x40201008040200,
    0x402010080400,
    0x4020100A00,
    0x40221400,
    0x2442800,
    0x204085000,
    0x20408102000,
    0x2040810204000,
    0x20100804020000,
    0x40201008040000,
    0x4020100A0000,
    0x4022140000,
    0x244280000,
    0x20408500000,
    0x2040810200000,
    0x4081020400000,
    0x10080402000200,
    0x20100804000400,
    0x4020100A000A00,
    0x402214001400,
    0x24428002800,
    0x2040850005000,
    0x4081020002000,
    0x8102040004000,
    0x8040200020400,
    0x10080400040800,
    0x20100A000A1000,
    0x40221400142200,
    0x2442800284400,
    0x4085000500800,
    0x8102000201000,
    0x10204000402000,
    0x4020002040800,
    0x8040004081000,
    0x100A000A102000,
    0x22140014224000,
    0x44280028440200,
    0x8500050080400,
    0x10200020100800,
    0x20400040201000,
    0x2000204081000,
    0x4000408102000,
    0xA000A10204000,
    0x14001422400000,
    0x28002844020000,
    0x50005008040200,
    0x20002010080400,
    0x40004020100800,
    0x20408102000,
    0x40810204000,
    0xA1020400000,
    0x142240000000,
    0x284402000000,
    0x500804020000,
    0x201008040200,
    0x402010080400,
    0x2040810204000,
    0x4081020400000,
    0xA102040000000,
    0x14224000000000,
    0x28440200000000,
    0x50080402000000,
    0x20100804020000,
    0x40201008040200,
];

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

const fn pop_1st_bit(mut bb: u64) -> (u64, i32) {
    let b = bb ^ (bb - 1);
    let fold: u32 = ((b & 0xffffffff) ^ (b >> 32)) as u32;
    bb &= bb - 1;
    (
        bb,
        BIT_TABLE[(fold.wrapping_mul(0x783a9b23) >> 26) as usize],
    )
}

pub const fn index_to_u64(index: i32, bits: i32, mut mask: u64) -> u64 {
    let mut result: u64 = 0;
    let mut i = 0;
    let mut j;
    while i < bits {
        (mask, j) = pop_1st_bit(mask);
        if index & (1 << i) != 0 {
            result |= 1 << j;
        }
        i += 1;
    }

    result
}

pub const fn ratt(sq: i32, block: u64) -> u64 {
    let mut result: u64 = 0;
    let rank = sq / 8;
    let file = sq % 8;

    let mut r = rank + 1;
    while r <= 7 {
        result |= 1 << (file + r * 8);
        if block & (1 << (file + r * 8)) != 0 {
            break;
        }
        r += 1;
    }

    let mut r = rank - 1;
    while r >= 0 {
        result |= 1 << (file + r * 8);
        if block & (1 << (file + r * 8)) != 0 {
            break;
        }
        r -= 1;
    }

    let mut f = file + 1;
    while f <= 7 {
        result |= 1 << (f + rank * 8);
        if block & (1 << (f + rank * 8)) != 0 {
            break;
        }
        f += 1;
    }

    let mut f = file - 1;
    while f >= 0 {
        result |= 1 << (f + rank * 8);
        if block & (1 << (f + rank * 8)) != 0 {
            break;
        }
        f -= 1;
    }

    result
}

pub const fn batt(sq: i32, block: u64) -> u64 {
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

fn _find_magic(sq: i32, m: i32, bishop: bool) -> u64 {
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

pub fn _find_new_magics() {
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
