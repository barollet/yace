use std::sync::LazyLock;

use bit_iter::BitIter;
use bitboard::*;

use crate::{consts::*, moves::*, Board};

static KNIGHT_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_knight_attack);

impl Board {
    pub fn pseudo_legal_move_gen(&self) -> Vec<Move> {
        MoveGenerator::new(self).moves
    }
}

struct MoveGenerator<'a> {
    board: &'a Board,
    moves: Vec<Move>
}

impl<'a> MoveGenerator<'a> {
    fn new(board: &'a Board) -> Self {
        let mut move_generator = Self {
            board,
            moves: Vec::with_capacity(20),
        };

        move_generator.generate_pawn_moves();
        move_generator.generate_kight_moves();

        move_generator
    }

    fn generate_pawn_moves(&mut self) {
        let me = self.board.to_move;
        let pawns = self.board.pieces[me] & self.board.bitboards[PAWN];
        let pieces = self.board.pieces[me] | self.board.pieces[!me];

        // Simple push
        let simple_dest = if me == WHITE {
            pawns << 8
        } else {
            pawns >> 8
        } & !pieces;

        for dest_square in BitIter::from(simple_dest) {
            let origin_square = if me == WHITE { dest_square - 8 } else { dest_square + 8 };
            self.moves.push(Move::new_base(origin_square, dest_square));
        }

        // Double push
        let double_dest = if me == WHITE {
            (simple_dest & RANK3) << 8
        } else {
            (simple_dest & RANK6) >> 8
        } & !pieces;

        for dest_square in BitIter::from(double_dest) {
            let origin_square = if me == WHITE { dest_square - 16 } else { dest_square + 16 };
            self.moves.push(Move::new_base(origin_square,dest_square).with_infos(MoveInfo::DoublePawnPush));
        }

        // TODO Capture
        // TODO Promotion
        // TODO EN PASSANT
    }

    fn generate_kight_moves(&mut self) {
        let me = self.board.to_move;
        let knights = self.board.bitboards[KNIGHT] & self.board.pieces[me];
        for knight_square in BitIter::from(knights) {
            let attack = KNIGHT_ATTACK[knight_square] & !self.board.pieces[me];
            for dest_square in BitIter::from(attack) {
                let knight_move = Move::new_base(knight_square, dest_square);
                if self.board.pieces[!me].has(dest_square as u8) {
                    self.moves.push(knight_move.with_infos(MoveInfo::Capture));
                } else {
                    self.moves.push(knight_move);
                }
            }
        }
    }
}

fn initialize_knight_attack() -> [Bitboard; 64] {
    let mut knight_attack = [0; 64];
    for square in 0..64 {
        let mut attack = EMPTY;
        let f = square.file();
        let r = square.rank();

        if f > 0 && r > 1 {
            attack = attack.set(Square::new(f-1, r-2));
        }
        if f > 1 && r > 0 {
            attack = attack.set(Square::new(f-2, r-1));
        }
        if f < 7 && r < 6 {
            attack = attack.set(Square::new(f+1, r+2));
        }
        if f < 6 && r < 7 {
            attack = attack.set(Square::new(f+2, r+1));
        }
        if f > 0 && r < 6 {
            attack = attack.set(Square::new(f-1, r+2));
        }
        if f < 7 && r > 1 {
            attack = attack.set(Square::new(f+1, r-2));
        }
        if f > 1 && r < 7 {
            attack = attack.set(Square::new(f-2, r+1));
        }
        if f < 6 && r > 0 {
            attack = attack.set(Square::new(f+2, r-1));
        }

        knight_attack[square as usize] = attack;
    }
    knight_attack
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft1() {
        let board = Board::new();
        assert_eq!(board.pseudo_legal_move_gen().len(), 20);
    }
}