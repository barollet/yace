use std::{array, cell::RefCell, sync::LazyLock};

use seeded_random::{Random,Seed};

use super::*;

pub type ZobristHash = u64;

thread_local! {
    static RNG: RefCell<Random> = RefCell::new(Random::from_seed(Seed::unsafe_new(25)));
}

// Indexed by square -> piece -> color
static ZOBRIST_PIECE_VALUES: LazyLock<[PieceIndexed<ColorIndexed<u64>>; 64]> = LazyLock::new(initialize_zobrist_table);
static ZOBRIST_TO_MOVE: LazyLock<u64> = LazyLock::new(random_u64);
static ZOBRIST_EP_FILE: LazyLock<[u64; 8]> = LazyLock::new(|| array::from_fn(|_| random_u64()));
static ZOBRIST_CASTLING: LazyLock<[u64; 16]> = LazyLock::new(|| array::from_fn(|_| random_u64()));

fn random_u64() -> u64 {
    RNG.with_borrow_mut(|c| c.u32() as u64 | ((c.u32() as u64) << 32))
}

fn initialize_zobrist_table() -> [PieceIndexed<ColorIndexed<u64>>; 64] {
    array::from_fn(|_sq| PieceIndexed::from_fn(|_p| ColorIndexed::from_fn(|_c| random_u64())))
}

pub trait ZobristHasher {
    fn new_hash() -> Self;
    fn handle_piece(&mut self, sq: Square, piece: Piece, color: Color);
    fn handle_side_to_move(&mut self);
    fn handle_ep(&mut self, ep_square: Option<Square>);
    fn handle_castling(&mut self, castling: CastlingRights);
}

impl ZobristHasher for ZobristHash {
    fn new_hash() -> Self {
        0
    }

    fn handle_piece(&mut self, sq: Square, piece: Piece, color: Color) {
        *self ^= ZOBRIST_PIECE_VALUES[sq as usize][piece][color]
    }

    fn handle_side_to_move(&mut self) {
        *self ^= *ZOBRIST_TO_MOVE
    }

    fn handle_ep(&mut self, ep_square: Option<Square>) {
        if let Some(sq) = ep_square {
            *self ^= ZOBRIST_EP_FILE[sq.file() as usize]
        }
    }

    fn handle_castling(&mut self, castling: CastlingRights) {
        *self ^= ZOBRIST_CASTLING[castling as usize]
    }
}