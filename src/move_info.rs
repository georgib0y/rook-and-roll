use std::collections::HashMap;
use crate::eval::gen_mat_value;

pub const SQUARES: [u64; 65] = [
    0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000,
    0x8000, 0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000, 0x1000000,
    0x2000000, 0x4000000, 0x8000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000, 0x100000000,
    0x200000000, 0x400000000, 0x800000000, 0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000,
    0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000, 0x100000000000, 0x200000000000,
    0x400000000000, 0x800000000000, 0x1000000000000, 0x2000000000000, 0x4000000000000,
    0x8000000000000, 0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000,
    0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000, 0x1000000000000000,
    0x2000000000000000, 0x4000000000000000, 0x8000000000000000, 0,
];

// file masks
pub const FA: u64 = 0x0101010101010101;
pub const FB: u64 = 0x0202020202020202;
pub const FC: u64 = 0x0404040404040404;
pub const FD: u64 = 0x0808080808080808;
pub const FE: u64 = 0x1010101010101010;
pub const FF: u64 = 0x2020202020202020;
pub const FG: u64 = 0x4040404040404040;
pub const FH: u64 = 0x8080808080808080;

// rank masks
pub const R1: u64 = 0x00000000000000FF;
pub const R2: u64 = 0x000000000000FF00;
pub const R3: u64 = 0x0000000000FF0000;
pub const R4: u64 = 0x00000000FF000000;
pub const R5: u64 = 0x000000FF00000000;
pub const R6: u64 = 0x0000FF0000000000;
pub const R7: u64 = 0x00FF000000000000;
pub const R8: u64 = 0xFF00000000000000;

pub const ROOK_MAGIC: [u64; 64] = [
    0x40800022400A1080, 0x420401001E800, 0x100402000110005, 0x4288002010500008, 0x60400200040001C0,
    0x50001000208C400, 0x1008240803000840, 0x2000044018A2201, 0x70401040042000, 0x2882030131020803,
    0x4A00100850800, 0x205400400400840, 0x3012000401100620, 0x80104200008404, 0x148325380100,
    0x8000120222408100, 0x8484821011400400, 0x8204044020203000, 0x88020300A0010004,
    0x4120200102024280, 0x100200092408044C, 0x80208014010000C0, 0x1000820820040, 0x10600A000401100,
    0x4824080013020, 0x8010200008844040, 0x41000424044040, 0x1C08008012400220, 0x2200200041200,
    0x1040049088460400, 0x218C4800412A0, 0x2009A008004080, 0x80010200A40808, 0x2010004801200092,
    0x220B02004040005, 0xC00080080801000, 0x3002110400080044, 0x40002021110C2, 0x2010081042009104,
    0x460802000480104, 0x5441020100202800, 0x800810221160400, 0x1084200E0008, 0x10281003010002,
    0x2204004081000800, 0x1803204140100400, 0x840B002110024, 0x201805082220001, 0x7324118001006208,
    0x1012402001830004, 0x100E000806002020, 0xA0201408020200, 0x110100802110018, 0x30001800080,
    0x2280005200911080, 0x101024220108008, 0x2000800100402011, 0x11020080400A, 0x200200044184111A,
    0x68900A0004121036, 0x600900100380083, 0x8001000400020481, 0x60068802491402, 0x8000010038804402,
];

pub const BISHOP_MAGIC: [u64; 64] = [
    0x2140004101030008, 0xA30208100100420, 0x102028202000101, 0x141104008002500, 0x6008142001A8002A,
    0x81402400A8300, 0x20904410420020, 0x8048108804202010, 0x8001480520440080, 0x108920168001080,
    0x10821401002208, 0x9004100D000, 0x80A00444804C6010, 0x8004020200240001, 0x10000882002A0A48,
    0x2000100220681412, 0x2240800700410, 0x38080020401082, 0x12C0920100410100, 0x220100404288000,
    0x24009A00850000, 0x2422000040100180, 0x322C010022820040, 0x89040C010040, 0x400602001022230,
    0x401008000128006C, 0x421004420080, 0xA420202008008020, 0x1010120104000, 0x8881480000882C0,
    0x860112C112104108, 0x10A1082042000420, 0x100248104100684, 0x214188200A00640, 0x4881008210820,
    0x2000280800020A00, 0x40008201610104, 0x2004093020001220, 0x81004501000800C, 0x234841900C081016,
    0x704009221000402, 0x4540380010000214, 0x2030082000040, 0x8050808104093, 0x101188107464808,
    0x5041020802400802, 0x4010B44808850040, 0x10100040088000E0, 0x84C010108010, 0x800488140100,
    0x1000028020218440, 0x5010048A06220000, 0x8001040812041000, 0x1840026008109400,
    0x1046002206001882, 0x20204400D84000, 0x1270C20060804000, 0x2000021113042200, 0x40002412282008A,
    0xC000000041100, 0x1000200060005104, 0x1840042164280880, 0x964AD0002100AA00, 0x2190900041002410,
];

pub const ROOK_MASK: [u64; 64] = [
    0x101010101017e, 0x202020202027c, 0x404040404047a, 0x8080808080876, 0x1010101010106e,
    0x2020202020205e, 0x4040404040403e, 0x8080808080807e, 0x1010101017e00, 0x2020202027c00,
    0x4040404047a00, 0x8080808087600, 0x10101010106e00, 0x20202020205e00, 0x40404040403e00,
    0x80808080807e00, 0x10101017e0100, 0x20202027c0200, 0x40404047a0400, 0x8080808760800,
    0x101010106e1000, 0x202020205e2000, 0x404040403e4000, 0x808080807e8000, 0x101017e010100,
    0x202027c020200, 0x404047a040400, 0x8080876080800, 0x1010106e101000, 0x2020205e202000,
    0x4040403e404000, 0x8080807e808000, 0x1017e01010100, 0x2027c02020200, 0x4047a04040400,
    0x8087608080800, 0x10106e10101000, 0x20205e20202000, 0x40403e40404000, 0x80807e80808000,
    0x17e0101010100, 0x27c0202020200, 0x47a0404040400, 0x8760808080800, 0x106e1010101000,
    0x205e2020202000, 0x403e4040404000, 0x807e8080808000, 0x7e010101010100, 0x7c020202020200,
    0x7a040404040400, 0x76080808080800, 0x6e101010101000, 0x5e202020202000, 0x3e404040404000,
    0x7e808080808000, 0x7e01010101010100, 0x7c02020202020200, 0x7a04040404040400,
    0x7608080808080800, 0x6e10101010101000, 0x5e20202020202000, 0x3e40404040404000,
    0x7e80808080808000,
];

pub const BISHOP_MASK: [u64; 64] = [
    0x40201008040200, 0x402010080400, 0x4020100A00, 0x40221400, 0x2442800, 0x204085000,
    0x20408102000, 0x2040810204000, 0x20100804020000, 0x40201008040000, 0x4020100A0000,
    0x4022140000, 0x244280000, 0x20408500000, 0x2040810200000, 0x4081020400000, 0x10080402000200,
    0x20100804000400, 0x4020100A000A00, 0x402214001400, 0x24428002800, 0x2040850005000,
    0x4081020002000, 0x8102040004000, 0x8040200020400, 0x10080400040800, 0x20100A000A1000,
    0x40221400142200, 0x2442800284400, 0x4085000500800, 0x8102000201000, 0x10204000402000,
    0x4020002040800, 0x8040004081000, 0x100A000A102000, 0x22140014224000, 0x44280028440200,
    0x8500050080400, 0x10200020100800, 0x20400040201000, 0x2000204081000, 0x4000408102000,
    0xA000A10204000, 0x14001422400000, 0x28002844020000, 0x50005008040200, 0x20002010080400,
    0x40004020100800, 0x20408102000, 0x40810204000, 0xA1020400000, 0x142240000000, 0x284402000000,
    0x500804020000, 0x201008040200, 0x402010080400, 0x2040810204000, 0x4081020400000,
    0xA102040000000, 0x14224000000000, 0x28440200000000, 0x50080402000000, 0x20100804020000,
    0x40201008040200,
];

pub const SQ_NAMES: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

/*
0: up left
1: up
2: up right
3: right
4: down right
5: down
6: down left
7: left
*/
pub const RAYS: [[u64; 65]; 8] = [
    [
        0x0, 0x100, 0x10200, 0x1020400, 0x102040800, 0x10204081000, 0x1020408102000,
        0x102040810204000, 0x0, 0x10000, 0x1020000, 0x102040000, 0x10204080000, 0x1020408100000,
        0x102040810200000, 0x204081020400000, 0x0, 0x1000000, 0x102000000, 0x10204000000,
        0x1020408000000, 0x102040810000000, 0x204081020000000, 0x408102040000000, 0x0, 0x100000000,
        0x10200000000, 0x1020400000000, 0x102040800000000, 0x204081000000000, 0x408102000000000,
        0x810204000000000, 0x0, 0x10000000000, 0x1020000000000, 0x102040000000000,
        0x204080000000000, 0x408100000000000, 0x810200000000000, 0x1020400000000000, 0x0,
        0x1000000000000, 0x102000000000000, 0x204000000000000, 0x408000000000000, 0x810000000000000,
        0x1020000000000000, 0x2040000000000000, 0x0, 0x100000000000000, 0x200000000000000,
        0x400000000000000, 0x800000000000000, 0x1000000000000000, 0x2000000000000000,
        0x4000000000000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0,
    ],
    [
        0x101010101010100, 0x202020202020200, 0x404040404040400, 0x808080808080800,
        0x1010101010101000, 0x2020202020202000, 0x4040404040404000, 0x8080808080808000,
        0x101010101010000, 0x202020202020000, 0x404040404040000, 0x808080808080000,
        0x1010101010100000, 0x2020202020200000, 0x4040404040400000, 0x8080808080800000,
        0x101010101000000, 0x202020202000000, 0x404040404000000, 0x808080808000000,
        0x1010101010000000, 0x2020202020000000, 0x4040404040000000, 0x8080808080000000,
        0x101010100000000, 0x202020200000000, 0x404040400000000, 0x808080800000000,
        0x1010101000000000, 0x2020202000000000, 0x4040404000000000, 0x8080808000000000,
        0x101010000000000, 0x202020000000000, 0x404040000000000, 0x808080000000000,
        0x1010100000000000, 0x2020200000000000, 0x4040400000000000, 0x8080800000000000,
        0x101000000000000, 0x202000000000000, 0x404000000000000, 0x808000000000000,
        0x1010000000000000, 0x2020000000000000, 0x4040000000000000, 0x8080000000000000,
        0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000,
        0x1000000000000000, 0x2000000000000000, 0x4000000000000000, 0x8000000000000000,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0,
    ],
    [
        0x8040201008040200, 0x80402010080400, 0x804020100800, 0x8040201000, 0x80402000, 0x804000,
        0x8000, 0x0, 0x4020100804020000, 0x8040201008040000, 0x80402010080000, 0x804020100000,
        0x8040200000, 0x80400000, 0x800000, 0x0, 0x2010080402000000, 0x4020100804000000,
        0x8040201008000000, 0x80402010000000, 0x804020000000, 0x8040000000, 0x80000000, 0x0,
        0x1008040200000000, 0x2010080400000000, 0x4020100800000000, 0x8040201000000000,
        0x80402000000000, 0x804000000000, 0x8000000000, 0x0, 0x804020000000000, 0x1008040000000000,
        0x2010080000000000, 0x4020100000000000, 0x8040200000000000, 0x80400000000000,
        0x800000000000, 0x0, 0x402000000000000, 0x804000000000000, 0x1008000000000000,
        0x2010000000000000, 0x4020000000000000, 0x8040000000000000, 0x80000000000000, 0x0,
        0x200000000000000, 0x400000000000000, 0x800000000000000, 0x1000000000000000,
        0x2000000000000000, 0x4000000000000000, 0x8000000000000000, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
        0x0, 0x0, 0x0, 0,
    ],
    [
        0xFE, 0xFC, 0xF8, 0xF0, 0xE0, 0xC0, 0x80, 0x0, 0xFE00, 0xFC00, 0xF800, 0xF000, 0xE000,
        0xC000, 0x8000, 0x0, 0xFE0000, 0xFC0000, 0xF80000, 0xF00000, 0xE00000, 0xC00000, 0x800000,
        0x0, 0xFE000000, 0xFC000000, 0xF8000000, 0xF0000000, 0xE0000000, 0xC0000000, 0x80000000,
        0x0, 0xFE00000000, 0xFC00000000, 0xF800000000, 0xF000000000, 0xE000000000, 0xC000000000,
        0x8000000000, 0x0, 0xFE0000000000, 0xFC0000000000, 0xF80000000000, 0xF00000000000,
        0xE00000000000, 0xC00000000000, 0x800000000000, 0x0, 0xFE000000000000, 0xFC000000000000,
        0xF8000000000000, 0xF0000000000000, 0xE0000000000000, 0xC0000000000000, 0x80000000000000,
        0x0, 0xFE00000000000000, 0xFC00000000000000, 0xF800000000000000, 0xF000000000000000,
        0xE000000000000000, 0xC000000000000000, 0x8000000000000000, 0x0, 0,
    ],
    [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x0, 0x204,
        0x408, 0x810, 0x1020, 0x2040, 0x4080, 0x8000, 0x0, 0x20408, 0x40810, 0x81020, 0x102040,
        0x204080, 0x408000, 0x800000, 0x0, 0x2040810, 0x4081020, 0x8102040, 0x10204080, 0x20408000,
        0x40800000, 0x80000000, 0x0, 0x204081020, 0x408102040, 0x810204080, 0x1020408000,
        0x2040800000, 0x4080000000, 0x8000000000, 0x0, 0x20408102040, 0x40810204080, 0x81020408000,
        0x102040800000, 0x204080000000, 0x408000000000, 0x800000000000, 0x0, 0x2040810204080,
        0x4081020408000, 0x8102040800000, 0x10204080000000, 0x20408000000000, 0x40800000000000,
        0x80000000000000, 0x0, 0,
    ],
    [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x101,
        0x202, 0x404, 0x808, 0x1010, 0x2020, 0x4040, 0x8080, 0x10101, 0x20202, 0x40404, 0x80808,
        0x101010, 0x202020, 0x404040, 0x808080, 0x1010101, 0x2020202, 0x4040404, 0x8080808,
        0x10101010, 0x20202020, 0x40404040, 0x80808080, 0x101010101, 0x202020202, 0x404040404,
        0x808080808, 0x1010101010, 0x2020202020, 0x4040404040, 0x8080808080, 0x10101010101,
        0x20202020202, 0x40404040404, 0x80808080808, 0x101010101010, 0x202020202020,
        0x404040404040, 0x808080808080, 0x1010101010101, 0x2020202020202, 0x4040404040404,
        0x8080808080808, 0x10101010101010, 0x20202020202020, 0x40404040404040, 0x80808080808080, 0,
    ],
    [
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x0,
        0x100, 0x201, 0x402, 0x804, 0x1008, 0x2010, 0x4020, 0x0, 0x10000, 0x20100, 0x40201, 0x80402,
        0x100804, 0x201008, 0x402010, 0x0, 0x1000000, 0x2010000, 0x4020100, 0x8040201, 0x10080402,
        0x20100804, 0x40201008, 0x0, 0x100000000, 0x201000000, 0x402010000, 0x804020100,
        0x1008040201, 0x2010080402, 0x4020100804, 0x0, 0x10000000000, 0x20100000000, 0x40201000000,
        0x80402010000, 0x100804020100, 0x201008040201, 0x402010080402, 0x0, 0x1000000000000,
        0x2010000000000, 0x4020100000000, 0x8040201000000, 0x10080402010000, 0x20100804020100,
        0x40201008040201, 0,
    ],
    [
        0x0, 0x1, 0x3, 0x7, 0xF, 0x1F, 0x3F, 0x7F, 0x0, 0x100, 0x300, 0x700, 0xF00, 0x1F00, 0x3F00,
        0x7F00, 0x0, 0x10000, 0x30000, 0x70000, 0xF0000, 0x1F0000, 0x3F0000, 0x7F0000, 0x0,
        0x1000000, 0x3000000, 0x7000000, 0xF000000, 0x1F000000, 0x3F000000, 0x7F000000, 0x0,
        0x100000000, 0x300000000, 0x700000000, 0xF00000000, 0x1F00000000, 0x3F00000000,
        0x7F00000000, 0x0, 0x10000000000, 0x30000000000, 0x70000000000, 0xF0000000000,
        0x1F0000000000, 0x3F0000000000, 0x7F0000000000, 0x0, 0x1000000000000, 0x3000000000000,
        0x7000000000000, 0xF000000000000, 0x1F000000000000, 0x3F000000000000, 0x7F000000000000, 0x0,
        0x100000000000000, 0x300000000000000, 0x700000000000000, 0xF00000000000000,
        0x1F00000000000000, 0x3F00000000000000, 0x7F00000000000000, 0,
    ],
];

// all PST are considered from whites perspective
pub const PST: [&[i8]; 14] = [
    &WPAWN_PT, &BPAWN_PT, &WKNIGHT_PT, &BKNIGHT_PT, &WROOK_PT, &BROOK_PT, &WBISHOP_PT, &BBISHOP_PT,
    &WQUEEN_PT, &BQUEEN_PT, &WKING_MID_PT, &BKING_MID_PT, &WKING_END_PT, &BKING_END_PT,
];

pub const WPAWN_PT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, -20, -20, 10, 10, 5, 5, -5, -10, 0, 0, -10, -5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, 5, 10, 25, 25, 10, 5, 5, 10, 10, 20, 30, 30, 20, 10, 10, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const WKNIGHT_PT: [i8; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 5, 5, 0, -20, -40, -30, 5, 10, 15, 15, 10, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 10, 15, 15, 10, 0, -30, -40, -20, 0, 0, 0, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
];

pub const WROOK_PT: [i8; 64] = [
    0, 0, 0, 5, 5, 0, 0, 0, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, 5, 10, 10, 10, 10, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const WBISHOP_PT: [i8; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20, -10, 5, 0, 0, 0, 0, 5, -10, -10, 10, 10, 10, 10, 10, 10, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 5, 10, 10, 5, 0, -10, -10, 0, 0, 0, 0, 0, 0, -10, -20, -10, -10, -10, -10, -10, -10, -20,
];

pub const WQUEEN_PT: [i8; 64] = [
    -20, -10, -10, -5, -5, -10, -10, -20,
    -10, 0, 0, 0, 0, 0, 0, -10,
    -10, 0, 5, 5, 5, 5, 0, -10,
    -5, 0, 5, 5, 5, 5, 0, -5,
    -5, 0, 5, 5, 5, 5, 0, 0,
    -10, 5, 5, 5, 5, 5, 0, -10,
    -10, 0, 5, 0, 0, 0, 0, -10,
    -20, -10, -10, -5, -5, -10, -10, -20,
];

pub const WKING_MID_PT: [i8; 64] = [
    20, 30, 10, 0, 0, 10, 30, 20, 20, 20, 0, 0, 0, 0, 20, 20, -10, -20, -20, -20, -20, -20, -20, -10, -20, -30, -30, -40, -40, -30, -30, -20, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30,
];

pub const WKING_END_PT: [i8; 64] = [
    -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30, -30, -30, -30, -30, -50,
];

pub const BPAWN_PT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -50, -50, -50, -50, -50, -50, -50, -50, -10, -10, -20, -30, -30, -20, -10, -10, -5, -5, -10, -25, -25, -10, -5, -5, 0, 0, 0, -20, -20, 0, 0, 0, -5, 5, 10, 0, 0, 10, 5, -5, -5, -10, -10, 20, 20, -10, -10, -5, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub const BKNIGHT_PT: [i8; 64] = [
    50, 40, 30, 30, 30, 30, 40, 50, 40, 20, 0, 0, 0, 0, 20, 40, 30, 0, -10, -15, -15, -10, 0, 30, 30, -5, -15, -20, -20, -15, -5, 30, 30, 0, -15, -20, -20, -15, 0, 30, 30, -5, -10, -15, -15, -10, -5, 30, 40, 20, 0, -5, -5, 0, 20, 40, 50, 40, 30, 30, 30, 30, 40, 50,
];

pub const BROOK_PT: [i8; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, -5, -10, -10, -10, -10, -10, -10, -5, 5, 0, 0, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, -5, -5, 0, 0, 0,
];

pub const BBISHOP_PT: [i8; 64] = [
    20, 10, 10, 10, 10, 10, 10, 20, 10, 0, 0, 0, 0, 0, 0, 10, 10, 0, -5, -10, -10, -5, 0, 10, 10, -5, -5, -10, -10, -5, -5, 10, 10, 0, -10, -10, -10, -10, 0, 10, 10, -10, -10, -10, -10, -10, -10, 10, 10, -5, 0, 0, 0, 0, -5, 10, 20, 10, 10, 10, 10, 10, 10, 20,
];

pub const BQUEEN_PT: [i8; 64] = [
    20, 10, 10, 5, 5, 10, 10, 20,
    10, 0, -5, 0, 0, 0, 0, 10,
    10, -5, -5, -5, -5, -5, 0, 10,
    5, 0, -5, -5, -5, -5, 0, 0,
    5, 0, -5, -5, -5, -5, 0, 5,
    10, 0, -5, -5, -5, -5, 0, 10,
    10, 0, 0, 0, 0, 0, 0, 10,
    20, 10, 10, 5, 5, 10, 10, 20,
];

pub const BKING_MID_PT: [i8; 64] = [
    30, 40, 40, 50, 50, 40, 40, 30, 30, 40, 40, 50, 50, 40, 40, 30, 30, 40, 40, 50, 50, 40, 40, 30, 30, 40, 40, 50, 50, 40, 40, 30, 20, 30, 30, 40, 40, 30, 30, 20, 10, 20, 20, 20, 20, 20, 20, 10, -20, -20, 0, 0, 0, 0, -20, -20, -20, -30, -10, 0, 0, -10, -30, -20,
];

pub const BKING_END_PT: [i8; 64] = [
    50, 30, 30, 30, 30, 30, 30, 50, 30, 30, 0, 0, 0, 0, 30, 30, 30, 10, -20, -30, -30, -20, 10, 30, 30, 10, -30, -40, -40, -30, 10, 30, 30, 10, -30, -40, -40, -30, 10, 30, 30, 10, -20, -30, -30, -20, 10, 30, 30, 20, 10, 0, 0, 10, 20, 30, 50, 40, 30, 20, 20, 30, 40, 50,
];

#[test]
fn print_black_pst() {
    println!("pub const BPAWN_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WPAWN_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BKNIGHT_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WKNIGHT_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BROOK_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WROOK_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BBISHOP_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WBISHOP_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BQUEEN_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WQUEEN_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BKING_MID_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WKING_MID_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");

    println!("pub const BKING_END_PT: [i8;64] = [");
    for i in (0..8).rev() {
        for sq in WKING_END_PT.iter().skip(i*8).take(8) {
            print!("{sq}, ");
        }
    }
    println!("\n];");
}