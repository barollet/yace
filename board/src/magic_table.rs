use std::{array, sync::LazyLock, fmt::Debug};

use bitboard::*;

pub struct Magic {
    factor: u64,
    shift: u8,
    pre_mask: Bitboard,
    post_mask: Bitboard,
    offset: usize,
}

static ROOK_MAGIC: LazyLock<[Magic; 64]> = LazyLock::new(build_rook_magic);
static BISHOP_MAGIC: LazyLock<[Magic; 64]> = LazyLock::new(build_bishop_magic);
static ATTACK_TABLE: LazyLock<[Bitboard; TOTAL_TABLE_SIZE]> = LazyLock::new(build_magic_table);

const BISHOP_FACTORS: [u64; 64] = [
    0x0070cc1000420022,
    0x0004100401002030,
    0x002800c400840290,
    0x1e044100a2000080,
    0x1805114002021a80,
    0x0809882440000445,
    0x00250c110d400808,
    0x1c04a02904104000,
    0x0908200730090308,
    0x0000050404140424,
    0x16b1100102003098,
    0x0400444400802002,
    0x20000202100000e7,
    0x000819040240a100,
    0x0052008201104022,
    0x10080020a2101001,
    0x0604004010042122,
    0x40604008880118c0,
    0x8004080800440018,
    0x0104020824220000,
    0x0001000090400000,
    0x40d14002004a201a,
    0x70021004d8120800,
    0x2802020040620810,
    0x0893080220a01400,
    0x0058144020031600,
    0x0404480004080010,
    0x0226410008009100,
    0x0010048004002100,
    0x8008020000208400,
    0x0000840008a40444,
    0x8000802022021222,
    0x1008041002400200,
    0x0002020340200801,
    0x8002010100500540,
    0x0006010040040040,
    0x00421284000200a0,
    0x0009010100820050,
    0x0502180102884c08,
    0x8002808288010409,
    0x0808040520009400,
    0x6904008410200402,
    0x80080c0048000408,
    0x6000804208000083,
    0x0042500202010194,
    0x4605103000c00180,
    0x0020090901240204,
    0x0402080901000020,
    0x000a020104411000,
    0x0081820149200000,
    0x1800846205100000,
    0x0080200242060108,
    0x0008a50803040420,
    0x04800520040500a0,
    0x0044a82208020020,
    0x0818020082020000,
    0x204864020a100205,
    0x5242052404040d00,
    0x0000108100809000,
    0x0008040004c20206,
    0x08008000a0142400,
    0x0000006060020491,
    0x0001204401160c00,
    0x10c0040802204450,
];

const ROOK_FACTORS: [u64; 64] = [
    0x0180004002201080,
    0x1040012008100440,
    0x0080200088801000,
    0x1480100008000482,
    0x02001008450a0020,
    0x0100010002040008,
    0x0480050002000c80,
    0x8500021040208100,
    0x16458000c0008e20,
    0x0000802000804000,
    0x181200314080e200,
    0x1200801000080282,
    0x0002002006001028,
    0x2001000b00080400,
    0x1002002802000514,
    0x00810001000a8066,
    0x5500b08002884000,
    0x2110004000200148,
    0x0000808020011002,
    0x00400a0012004022,
    0x000206002e000a20,
    0x1400808002000400,
    0x120c040022011028,
    0x0041020000840443,
    0x4400401480008260,
    0x023000474001a000,
    0x0202900880200180,
    0x1240100080080080,
    0x0005011100040800,
    0x0132008080020400,
    0x00000a0400010830,
    0x0000008e00040041,
    0x1000a24000800083,
    0x00008a4000802000,
    0x0510410111002000,
    0x0100802805801000,
    0x2008004004040020,
    0x0002005022000c09,
    0x40009006a4000108,
    0x000970830a002044,
    0x0000400020818009,
    0x0810004020004000,
    0x0201d00020008080,
    0x8008482200420010,
    0x0009240801010030,
    0x205400800a008004,
    0x4000900208040051,
    0x0142040840820003,
    0x4000800220411900,
    0x0008400020048280,
    0x03200080a0300480,
    0x2428480180100080,
    0x0028008084000880,
    0x4011801400020080,
    0x220a000428210200,
    0x0a00800900044080,
    0xc8e0114102008062,
    0x0114225101804001,
    0x0000200041021009,
    0x4405000820900005,
    0x0482001408502002,
    0x0101008204000813,
    0x4001100802010284,
    0x0001000600802041,
];

pub fn bishop_attack(sq: Square, occupancy: Bitboard) -> Bitboard {
    let magic = &BISHOP_MAGIC[sq as usize];
    ATTACK_TABLE[magic.index(occupancy)] & magic.post_mask
}

pub fn rook_attack(sq:Square, occupancy: Bitboard) -> Bitboard {
    let magic = &ROOK_MAGIC[sq as usize];
    ATTACK_TABLE[magic.index(occupancy)] & magic.post_mask
}

impl Magic {
    pub fn new(factor: u64, nbits: u32, pre_mask: Bitboard, post_mask: Bitboard, offset: usize) -> Self {
        Self { factor, shift: 64 - nbits as u8, pre_mask, post_mask, offset }
    }
    pub fn index(&self, occupancy: Bitboard) -> usize {
        self.offset + ((occupancy & self.pre_mask).overflowing_mul(self.factor).0 >> self.shift) as usize
    }
}

const ROOK_SHARING: [usize; 64] = [
    0,  1,  2,  3,  4,  5,  6,  7,
    1,  0,  3,  2,  5,  4,  7,  6,
    8,  9, 10, 11, 12, 13, 14, 15,
    9,  8, 11, 10, 13, 12, 15, 14,
   16, 17, 18, 19, 20, 21, 22, 23,
   17, 16, 19, 18, 21, 20, 23, 22,
   24, 25, 26, 27, 28, 29, 30, 31,
   25, 24, 27, 26, 29, 28, 31, 30,
];

const ROOK_SHARED_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 12, 11, 11, 11, 11, 12, 11,
    11, 11, 10, 10, 10, 10, 11, 11,
    11, 11, 10, 10, 10, 10, 11, 11,
    11, 11, 10, 10, 10, 10, 11, 11,
    11, 11, 10, 10, 10, 10, 11, 11,
    11, 12, 11, 11, 11, 11, 12, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

const BISHOP_SHARING: [usize; 64] = [
    0,  2,  4,  4,  4,  4, 12, 14,
    0,  2,  5,  5,  5,  5, 12, 14,
    0,  2,  6,  6,  6,  6, 12, 14,
    0,  2,  7,  7,  7,  7, 12, 14,
    1,  3,  8,  8,  8,  8, 13, 15,
    1,  3,  9,  9,  9,  9, 13, 15,
    1,  3, 10, 10, 10, 10, 13, 15,
    1,  3, 11, 11, 11, 11, 13, 15,
];

const BISHOP_SHARED_BITS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    6, 5, 5, 5, 5, 5, 5, 6,
    6, 5, 7, 7, 7, 7, 5, 6,
    6, 5, 9, 9, 9, 9, 5, 6,
    6, 5, 9, 9, 9, 9, 5, 6,
    6, 5, 7, 7, 7, 7, 5, 6,
    6, 5, 5, 5, 5, 5, 5, 6,
    6, 5, 5, 5, 5, 5, 5, 6
];

const ROOK_TABLE_SIZE: usize = 65536;
const BISHOP_TABLE_SIZE: usize = 1792;
const TOTAL_TABLE_SIZE: usize = ROOK_TABLE_SIZE + BISHOP_TABLE_SIZE;

static ROOK_OFFSETS: LazyLock<[usize; 64]> = LazyLock::new(build_rook_offsets);
static BISHOP_OFFSETS: LazyLock<[usize; 64]> = LazyLock::new(build_bishop_offsets);

fn build_rook_magic() -> [Magic; 64] {
    array::from_fn(|sq| {
        let pre_mask = rook_relevant_mask(sq as Square);
        Magic::new(ROOK_FACTORS[sq], pre_mask.count_ones(), pre_mask, rook_full_attack(sq as Square, EMPTY), ROOK_OFFSETS[sq])
    })
}

fn build_bishop_magic() -> [Magic; 64] {
    array::from_fn(|sq| {
        let pre_mask = bishop_relevant_mask(sq as Square);
        Magic::new(BISHOP_FACTORS[sq], pre_mask.count_ones(), pre_mask, bishop_full_attack(sq as Square, EMPTY), BISHOP_OFFSETS[sq])
    })
}

macro_rules! build_offset {
    ($sharing: ident, $bits: ident, $max_share: expr, $start: expr) => {
        {
            let mut offsets = [0; 64];
            let mut current_offset = $start;

            for share_number in 0..$max_share {
                let mut squares = $sharing.iter().enumerate()
                    .filter_map(move |(sq, &n)| if n == share_number {Some(sq as Square)} else {None}).peekable();
                let &first_square = squares.peek().unwrap();
                for sq in squares {
                    offsets[sq as usize] = current_offset;
                }
                current_offset += 1 << $bits[first_square as usize];
            }

            offsets
        }
    };
}

fn build_rook_offsets() -> [usize; 64] {
    build_offset!(ROOK_SHARING, ROOK_SHARED_BITS, 32, 0)
}

fn build_bishop_offsets() -> [usize; 64] {
    build_offset!(BISHOP_SHARING, BISHOP_SHARED_BITS, 16, ROOK_TABLE_SIZE)
}

macro_rules! fill_attack {
    ($magic_table: ident, $mask_function: ident, $attack_function: ident, $magic: ident) => {
        for sq in 0..64 {
            let nbits = $mask_function(sq).count_ones();
            let occupancy_bits = occupancy_list($mask_function(sq));
    
            for i in 0..(1 << nbits) {
                let mut occupancy: u64 = EMPTY;
                for (k, mask) in occupancy_bits.iter().enumerate() {
                    if i & (1 << k) != 0 {
                        occupancy |= mask;
                    }
                }
    
                $magic_table[$magic[sq as usize].index(occupancy)] |= $attack_function(sq, occupancy);
            }
        }
    };
}

fn build_magic_table() -> [Bitboard; TOTAL_TABLE_SIZE] {
    let mut magic_table = [EMPTY; TOTAL_TABLE_SIZE];

    // rooks
    fill_attack!(magic_table, rook_relevant_mask, rook_full_attack, ROOK_MAGIC);

    // bishops
    fill_attack!(magic_table, bishop_relevant_mask, bishop_full_attack, BISHOP_MAGIC);

    magic_table
}

pub fn bishop_relevant_mask(sq: Square) -> Bitboard {
    let mut f = sq.file();
    let mut r = sq.rank();
    let mut mask = EMPTY;
    while f < 6 && r < 6 {
        f += 1;
        r += 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f > 1 && r > 1 {
        f -= 1;
        r -= 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f < 6 && r > 1 {
        f += 1;
        r -= 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f > 1 && r < 6 {
        f -= 1;
        r += 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    mask
}

pub fn rook_relevant_mask(sq: Square) -> Bitboard {
    let mut f = sq.file();
    let r = sq.rank();
    let mut mask = EMPTY;
    while f < 6 {
        f += 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    while f > 1 {
        f -= 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let f = sq.file();
    let mut r = sq.rank();
    while r > 1 {
        r -= 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    let mut r = sq.rank();
    while r < 6 {
        r += 1;
        mask |= Square::new(f, r).to_bitboard();
    }
    mask
}

pub fn bishop_full_attack(sq: Square, occupancy: Bitboard) -> Bitboard {
    let mut attack = EMPTY;

    let mut f = sq.file();
    let mut r = sq.rank();
    while f < 7 && r < 7 && !occupancy.has(Square::new(f, r)) {
        f += 1;
        r += 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f > 0 && r > 0 && !occupancy.has(Square::new(f, r)) {
        f -= 1;
        r -= 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f < 7 && r > 0 && !occupancy.has(Square::new(f, r)) {
        f += 1;
        r -= 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    let mut r = sq.rank();
    while f > 0 && r < 7 && !occupancy.has(Square::new(f, r)) {
        f -= 1;
        r += 1;
        attack |= Square::new(f, r).to_bitboard();
    }

    attack
}

pub fn rook_full_attack(sq: Square, occupancy: Bitboard) -> Bitboard {
    let mut attack = EMPTY;

    let mut f: u8 = sq.file();
    let r = sq.rank();
    while f < 7 && !occupancy.has(Square::new(f, r)) {
        f += 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let mut f = sq.file();
    while f > 0 && !occupancy.has(Square::new(f, r)) {
        f -= 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let f = sq.file();
    let mut r = sq.rank();
    while r > 0 && !occupancy.has(Square::new(f, r)) {
        r -= 1;
        attack |= Square::new(f, r).to_bitboard();
    }
    let mut r = sq.rank();
    while r < 7 && !occupancy.has(Square::new(f, r)) {
        r += 1;
        attack |= Square::new(f, r).to_bitboard();
    }

    attack
}

pub fn occupancy_list(mut relevant_mask: Bitboard) -> Vec<Bitboard> {
    let mut output = Vec::with_capacity(relevant_mask.count_ones() as usize);

    while relevant_mask != 0 {
        output.push(!(relevant_mask - 1) & relevant_mask);
        relevant_mask &= relevant_mask - 1;
    }

    output
}

impl Debug for Magic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Magic").field("factor", &self.factor).field("shift", &self.shift).field("offset", &self.offset).finish()?;
        writeln!(f, "\npre_mask:\n{}", self.pre_mask.to_string())?;
        writeln!(f, "post_mask:\n{}", self.post_mask.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_attack {
        ($mask_function: ident, $table_attack_function: ident, $attack_function: ident) => {
            for sq in 0..64 {
                let nbits = $mask_function(sq).count_ones();
                let occupancy_bits = occupancy_list($mask_function(sq));
        
                for i in 0..(1 << nbits) {
                    let mut occupancy: u64 = EMPTY;
                    for (k, mask) in occupancy_bits.iter().enumerate() {
                        if i & (1 << k) != 0 {
                            occupancy |= mask;
                        }
                    }
    
                    assert_eq!($table_attack_function(sq, occupancy), $attack_function(sq, occupancy))
                }
            }
        };
    }

    #[test]
    fn test_magic_factors() {
        test_attack!(rook_relevant_mask, rook_attack, rook_full_attack);
        test_attack!(bishop_relevant_mask, bishop_attack, bishop_full_attack);
    }
}