use std::cell::RefCell;

use bitboard::*;

use rand::prelude::*;

const NTRIES: usize = 10000000;
const BISHOP: bool = true;
const ROOK: bool = false;
thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::rng());
}

fn random_magic() -> u64 {
    RNG.with_borrow_mut(|c| c.next_u64())
    & RNG.with_borrow_mut(|c| c.next_u64())
    & RNG.with_borrow_mut(|c| c.next_u64())
}

fn bishop_relevant_mask(sq: Square) -> Bitboard {
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

fn rook_relevant_mask(sq: Square) -> Bitboard {
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

fn bishop_attack(sq: Square, occupancy: Bitboard) -> Bitboard {
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

fn rook_attack(sq: Square, occupancy: Bitboard) -> Bitboard {
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

fn occupancy_list(mut relevant_mask: Bitboard) -> Vec<Bitboard> {
    let mut output = Vec::with_capacity(relevant_mask.count_ones() as usize);

    while relevant_mask != 0 {
        output.push(!(relevant_mask - 1) & relevant_mask);
        relevant_mask &= relevant_mask - 1;
    }

    output
}

fn find_magic_factor(sq: Square, is_bishop: bool) -> usize {
    let relevant_mask = if is_bishop {bishop_relevant_mask(sq)} else {rook_relevant_mask(sq)};
    let occupancy_bits = occupancy_list(relevant_mask);
    let nbits = relevant_mask.count_ones() as usize;

    for _ in 0..NTRIES {
        let mut attack_map = vec![EMPTY; 1 << nbits];
        let mut occupied = vec![false; 1 << nbits];

        let magic_factor = random_magic();

        if (0..(1 << nbits)).all(|i| {
            let mut occupancy = EMPTY;
            for (k, mask) in occupancy_bits.iter().enumerate() {
                if i & (1 << k) != 0 {
                    occupancy |= mask;
                }
            }

            let attack = if is_bishop {bishop_attack(sq, occupancy)} else {rook_attack(sq, occupancy)};
            let index = ((occupancy.overflowing_mul(magic_factor)).0 >> (64-nbits)) as usize;

            if !occupied[index] {
                occupied[index] = true;
                attack_map[index] = attack;

                true
            } else {
                attack_map[index] == attack
            }
        }) {
            return magic_factor as usize
        }
    }

    println!("Couldn't find magic for square {}", sq.debug());
    0
}

fn main() {
    println!("Bishop magic");
    for sq in 0..64 {
        println!("{:#018x},", find_magic_factor(sq, BISHOP));
    }
    println!("Rook magic");
    for sq in 0..64 {
        println!("{:#018x},", find_magic_factor(sq, ROOK));
    }
}
