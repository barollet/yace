use std::sync::LazyLock;

use bit_iter::BitIter;
use bitboard::*;

use crate::{consts::*, magic_table::{bishop_attack, rook_attack}, moves::*, Board};

static KNIGHT_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_knight_attack);
static KING_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_king_attack);

const QUIET: u8 = 0;
const CAPTURE: u8 = 1;
const EVASION: u8 = 2;
const NON_EVASION: u8 = 3;
const ALL: u8 = 4;

impl Board {
    pub fn legal_move_gen(&self) -> Vec<ExtendedMove> {
        if self.to_move == WHITE {
            MoveGenerator::new::<WHITE, ALL>(self).moves
        } else {
            MoveGenerator::new::<BLACK, ALL>(self).moves
        }
    }
}

struct MoveGenerator<'a> {
    board: &'a Board,
    moves: Vec<ExtendedMove>
}

impl<'a> MoveGenerator<'a> {
    fn new<const COLOR: bool, const KIND: u8>(board: &'a Board) -> Self {
        let mut move_generator = Self {
            board,
            moves: Vec::with_capacity(20),
        };

        for sq in BitIter::from(board.pieces[COLOR]) {
            let piece = board.squares[sq].unwrap();
            if piece == PAWN {
                move_generator.generate_pawn_moves::<COLOR, KIND>(sq as u8);
            } else if piece == KNIGHT {
                move_generator.generate_piece_moves::<KNIGHT_ORDINAL, COLOR, KIND>(sq as u8);
            } else if piece == BISHOP {
                move_generator.generate_piece_moves::<BISHOP_ORDINAL, COLOR, KIND>(sq as u8);
            } else if piece == ROOK {
                move_generator.generate_piece_moves::<ROOK_ORDINAL, COLOR, KIND>(sq as u8);
            } else if piece == QUEEN {
                move_generator.generate_piece_moves::<QUEEN_ORDINAL, COLOR, KIND>(sq as u8);
            } else if piece == KING {
                move_generator.generate_piece_moves::<KING_ORDINAL, COLOR, KIND>(sq as u8);
            }
        }

        move_generator
    }

    // Shouldn't be used on pawns
    fn generate_piece_moves<const PIECE: u8, const COLOR: bool, const KIND: u8>(&mut self, sq: Square) {
        let piece_moves = self.move_list::<PIECE, COLOR>(sq);

        if KIND == CAPTURE || KIND == NON_EVASION || KIND == ALL || ( KIND == EVASION && PIECE == KING_ORDINAL) {
            for dest_sq in BitIter::from(piece_moves & self.board.pieces[!COLOR]) {
                self.moves.push(
                    ExtendedMove::new_base(
                        Move::new_base(sq.into(), dest_sq).with_infos(MoveInfo::Capture),
                        self.board.squares[dest_sq])
                )
            }
        }

        if KIND == QUIET || KIND == NON_EVASION || KIND == ALL {
            for dest_sq in BitIter::from(piece_moves & !(self.board.pieces[COLOR] | self.board.pieces[!COLOR])) {
                self.moves.push(
                    ExtendedMove::new_base(
                        Move::new_base(sq.into(), dest_sq),
                        None)
                )
            }
        }
    }

    // Shouldn't be used on pawns
    fn move_list<const PIECE: u8, const COLOR: bool>(&self, sq: Square) -> Bitboard {
        let occupancy = self.board.pieces[WHITE] | self.board.pieces[BLACK];
        match PIECE {
            KNIGHT_ORDINAL => KNIGHT_ATTACK[sq as usize],
            BISHOP_ORDINAL => bishop_attack(sq, occupancy),
            ROOK_ORDINAL => rook_attack(sq, occupancy),
            QUEEN_ORDINAL => bishop_attack(sq, occupancy) | rook_attack(sq, occupancy),
            KING_ORDINAL => KING_ATTACK[sq as usize],

            _ => panic!("Invalid piece for capture list"),
        }
    }

    fn generate_pawn_moves<const COLOR: bool, const KIND: u8>(&mut self, sq: Square) {
        let all_pieces = self.board.pieces[COLOR] | self.board.pieces[!COLOR];
        // Simple push
        let forward_dest = sq.forward::<COLOR>();
        if KIND == QUIET || KIND == NON_EVASION || KIND == ALL && !all_pieces.has(forward_dest) {
            self.moves.push(ExtendedMove::new_base(Move::new_base(sq as usize, forward_dest as usize), None));

            let base_rank = if COLOR == WHITE {RANK2} else {RANK7};
            let double_push_dest = forward_dest.forward::<COLOR>();
            if base_rank.has(sq) && !all_pieces.has(double_push_dest) {
                self.moves.push(
                    ExtendedMove::new_base(
                        Move::new_base(sq as usize, double_push_dest as usize).with_infos(MoveInfo::DoublePawnPush),
                        None));
            }
        }

        // TODO Capture
        // TODO Promotion
        // TODO EN PASSANT
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

fn initialize_king_attack() -> [Bitboard; 64] {
    let mut king_attack = [0; 64];
    for square in 0..64 {
        let mut attack = EMPTY;
        let f = square.file();
        let r = square.rank();

        if f > 0 {
            attack = attack.set(Square::new(f-1, r));
            if r > 0 {
                attack = attack.set(Square::new(f-1, r-1));
            }
            if r < 7 {
                attack = attack.set(Square::new(f-1, r+1));
            }
        }
        if f < 7 {
            attack = attack.set(Square::new(f+1, r));
            if r > 0 {
                attack = attack.set(Square::new(f+1, r-1));
            }
            if r < 7 {
                attack = attack.set(Square::new(f+1, r+1));
            }
        }
        if r > 0 {
            attack = attack.set(Square::new(f, r-1));
        }
        if r < 7 {
            attack = attack.set(Square::new(f, r+1));
        }

        king_attack[square as usize] = attack;
    }
    king_attack
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft1() {
        let board = Board::new();
        dbg!(board.legal_move_gen());
        assert_eq!(board.legal_move_gen().len(), 20);
    }
}