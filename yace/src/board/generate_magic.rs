use std::cell::RefCell;

use crate::board::bitboard::*;
use crate::board::magic_table::*;

use rand::prelude::*;

const NTRIES: usize = 10000000;
const DO_BISHOP: bool = true;
const DO_ROOK: bool = false;
thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::rng());
}

fn random_magic() -> u64 {
    RNG.with_borrow_mut(|c| c.next_u64())
    & RNG.with_borrow_mut(|c| c.next_u64())
    & RNG.with_borrow_mut(|c| c.next_u64())
}

fn find_magic_factor(sq: Square, is_bishop: bool) -> usize {
    let relevant_mask = if is_bishop {bishop_relevant_mask(sq)} else {rook_relevant_mask(sq)};
    let occupancy_bits = occupancy_list(relevant_mask);
    let nbits = relevant_mask.count_ones() as usize;

    for _ in 0..NTRIES {
        let mut attack_map = vec![EMPTY; 1 << nbits];
        let mut occupied = vec![false; 1 << nbits];

        let magic_factor = random_magic();
        let magic = Magic::new(magic_factor, nbits as u32, relevant_mask, EMPTY, 0);

        if (0..(1 << nbits)).all(|i| {
            let mut occupancy = EMPTY;
            for (k, mask) in occupancy_bits.iter().enumerate() {
                if i & (1 << k) != 0 {
                    occupancy |= mask;
                }
            }

            let attack = if is_bishop {bishop_full_attack(sq, occupancy)} else {rook_full_attack(sq, occupancy)};
            let index = magic.index(occupancy);

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
        println!("{:#018x},", find_magic_factor(sq, DO_BISHOP));
    }
    println!("Rook magic");
    for sq in 0..64 {
        println!("{:#018x},", find_magic_factor(sq, DO_ROOK));
    }
}
