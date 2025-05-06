use std::sync::LazyLock;

use bit_iter::BitIter;
use crate::bitboard::*;

use crate::{consts::*, magic_table::{bishop_attack, rook_attack}, moves::*, Board};

static KNIGHT_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_knight_attack);
static KING_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_king_attack);
static EP_FROM_SQUARES: LazyLock<[Bitboard; 16]> = LazyLock::new(initialize_ep_from_squares);

const QUIET: u8 = 0;
const CAPTURE: u8 = 1;
const EVASION: u8 = 2;
const NON_EVASION: u8 = 3;

impl Board {
    pub fn legal_move_gen(&self) -> Vec<Move> {
        if self.to_move == WHITE {
            MoveGenerator::new(self).generate::<WHITE>()
        } else {
            MoveGenerator::new(self).generate::<BLACK>()
        }
    }

    fn bishop_attack(&self, sq: Square) -> Bitboard {
        let occupancy = self.pieces[WHITE] | self.pieces[BLACK];
        bishop_attack(sq, occupancy)
    }

    fn rook_attack(&self, sq: Square) -> Bitboard {
        let occupancy = self.pieces[WHITE] | self.pieces[BLACK];
        rook_attack(sq, occupancy)
    }

    pub fn square_attacked_by<const MY_COLOR: bool>(&self, sq: Square) -> Bitboard {
        self.square_attacked_by_with_occ::<MY_COLOR>(sq, self.pieces[WHITE] | self.pieces[BLACK])
    }

    fn square_attacked_by_with_occ<const MY_COLOR: bool>(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop_attack(sq, occupancy) & self.pieces[!MY_COLOR] & (self.bitboards[BISHOP] | self.bitboards[QUEEN])
        | rook_attack(sq, occupancy) & self.pieces[!MY_COLOR] & (self.bitboards[ROOK] | self.bitboards[QUEEN])
        | KNIGHT_ATTACK[sq as usize] & self.pieces[!MY_COLOR] & self.bitboards[KNIGHT]
        | self.pieces[!MY_COLOR] & self.bitboards[PAWN] & sq.forward_left::<MY_COLOR>().map_or(EMPTY, Square::as_bitboard)
        | self.pieces[!MY_COLOR] & self.bitboards[PAWN] & sq.forward_right::<MY_COLOR>().map_or(EMPTY, Square::as_bitboard)
    }

    fn pinned_pieces(&self) -> Bitboard {
        let king_square = self.king_square(self.to_move);
        let enemy_pieces = self.pieces[!self.to_move];

        let mut pinned = EMPTY;

        let snipers = rook_attack(king_square, EMPTY) & (self.bitboards[ROOK] | self.bitboards[QUEEN]) & enemy_pieces
            | bishop_attack(king_square, EMPTY) & (self.bitboards[BISHOP] | self.bitboards[QUEEN]) & enemy_pieces;

        for start_square in BitIter::from(snipers) {
            let line = Bitboard::between(start_square as Square, king_square).unset(start_square as Square) & (self.pieces[WHITE] | self.pieces[BLACK]);

            if line.count_ones() == 1 {
                pinned |= line;
            }
        }
        pinned
    }
}

struct MoveGenerator<'a> {
    board: &'a Board,
    moves: Vec<Move>
}

impl<'a> MoveGenerator<'a> {
    fn new(board: &'a Board) -> Self {
        Self {
            board,
            moves: Vec::with_capacity(32),
        }
    }

    fn generate<const COLOR: bool>(mut self) -> Vec<Move> {
        let king_square = self.board.king_square(self.board.to_move);
        if self.board.square_attacked_by::<COLOR>(king_square) != 0 {
            self.pseudo_legal_movegen::<COLOR, EVASION>();
        } else {
            self.pseudo_legal_movegen::<COLOR, NON_EVASION>();
        }

        let pinned = self.board.pinned_pieces();
        self.moves.retain(|m| {
            // TODO check castle

            if m.infos() == MoveInfo::EnPassantCapture {
                let ep_target = self.board.ep_target.unwrap();
                let occupancy = (self.board.pieces[WHITE] | self.board.pieces[BLACK]).unset(m.from()).unset(ep_target).set(m.to());
                // if the attack is made by the en passant target, it doesn't count
                if self.board.square_attacked_by_with_occ::<COLOR>(king_square, occupancy) != EMPTY 
                && king_square.forward_left::<COLOR>() != self.board.ep_target && king_square.forward_right::<COLOR>() != self.board.ep_target {
                    return false
                }
            }

            if pinned.has(m.from()) {
                // if the piece is pinned, the king must be on the line of its movement
                // if the movement is not a line then the bitboard is empty
                return line(m.from(), m.to()).has(king_square)
            }

            if let Some(KING) = self.board.squares[m.from() as usize] {
                let occupancy = (self.board.pieces[WHITE] | self.board.pieces[BLACK]).unset(king_square);
                return self.board.square_attacked_by_with_occ::<COLOR>(m.to(), occupancy) == EMPTY
            }
            
            true
        });

        self.moves
    }

    fn pseudo_legal_movegen<const COLOR: bool, const KIND: u8>(&mut self) {
        let target = if KIND == QUIET {
            !(self.board.pieces[WHITE] | self.board.pieces[BLACK])
        } else if KIND == CAPTURE {
            self.board.pieces[!COLOR]
        } else if KIND == NON_EVASION {
            !self.board.pieces[COLOR]
        } else { // EVASION
            let checker = self.board.checkers::<COLOR>().lsb();
            Bitboard::between(checker, self.board.king_square(COLOR))
        };

        // If this is not double check we can generate piece moves
        if !(KIND == EVASION && self.board.checkers::<COLOR>().count_ones() > 1) {
            self.generate_pawn_moves::<COLOR, KIND>(target);

            for sq in BitIter::from(self.board.pieces[COLOR]) {
                let piece: Piece = self.board.squares[sq].unwrap();
                if piece == KNIGHT {
                    self.generate_piece_moves::<KNIGHT_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == BISHOP {
                    self.generate_piece_moves::<BISHOP_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == ROOK {
                    self.generate_piece_moves::<ROOK_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == QUEEN {
                    self.generate_piece_moves::<QUEEN_ORDINAL, COLOR>(sq as Square, target);
                }
            }
        }

        let target = if KIND != EVASION {target} else {!self.board.pieces[COLOR]};
        self.generate_piece_moves::<KING_ORDINAL, COLOR>(self.board.king_square(self.board.to_move), target);

        // TODO Castling
    }

    // Shouldn't be used on pawns
    fn generate_piece_moves<const PIECE: u8, const COLOR: bool>(&mut self, sq: Square, target: Bitboard) {
        let piece_moves = self.move_list::<PIECE, COLOR>(sq);
        let empty = !(self.board.pieces[COLOR] | self.board.pieces[!COLOR]);

        for dest_square in BitIter::from(piece_moves & target & self.board.pieces[!COLOR]) {
            self.moves.push(Move::new_base(sq, dest_square as Square).with_infos(MoveInfo::Capture));
        }
    
        for dest_sq in BitIter::from(piece_moves & target & empty) {
            self.moves.push(Move::new_base(sq, dest_sq as Square));
        }
        
    }

    // Shouldn't be used on pawns
    fn move_list<const PIECE: u8, const COLOR: bool>(&self, sq: Square) -> Bitboard {
        match PIECE {
            KNIGHT_ORDINAL => KNIGHT_ATTACK[sq as usize],
            BISHOP_ORDINAL => self.board.bishop_attack(sq),
            ROOK_ORDINAL => self.board.rook_attack(sq),
            QUEEN_ORDINAL => self.board.bishop_attack(sq) | self.board.rook_attack(sq),
            KING_ORDINAL => KING_ATTACK[sq as usize],

            _ => panic!("Invalid piece for capture list"),
        }
    }

    fn generate_pawn_moves<const COLOR: bool, const KIND: u8>(&mut self, target: Bitboard) {
        let empty = !(self.board.pieces[COLOR] | self.board.pieces[!COLOR]);
        let pawns = self.board.bitboards[PAWN] & self.board.pieces[COLOR];

        // Simple push and double push
        let not_promoting_pawns = pawns & !if COLOR == WHITE {RANK7} else {RANK2};
        let base_rank = if COLOR == WHITE {RANK3} else {RANK6};
        let push_dest = not_promoting_pawns.forward::<COLOR>() & empty;
        let double_push_dest = (push_dest & base_rank).forward::<COLOR>() & empty & target;
        if KIND != CAPTURE {
            for dest_square in BitIter::from(push_dest & target) {
                let dest_square = dest_square as Square;
                self.moves.push(Move::new_base(dest_square.backward::<COLOR>(), dest_square));
            }

            for dest_square in BitIter::from(double_push_dest) {
                let dest_square = dest_square as Square;
                self.moves.push(Move::new_base(dest_square.backward::<COLOR>().backward::<COLOR>(), dest_square).with_infos(MoveInfo::DoublePawnPush));
            }
        } 

        // Simple capture
        if KIND != QUIET {
            for dest_square in BitIter::from(not_promoting_pawns.forward_left::<COLOR>() & self.board.pieces[!COLOR] & target) {
                let dest_square = dest_square as Square;
                self.moves.push(Move::new_base(dest_square.backward_left::<COLOR>().unwrap(), dest_square as Square).with_infos(MoveInfo::Capture));
            }

            for dest_square in BitIter::from(not_promoting_pawns.forward_right::<COLOR>() & self.board.pieces[!COLOR] & target) {
                let dest_square = dest_square as Square;
                self.moves.push(Move::new_base(dest_square.backward_right::<COLOR>().unwrap(), dest_square as Square).with_infos(MoveInfo::Capture));
            }

            // En passant
            if let Some(ep_target) = self.board.ep_target {
                let ep_dest = ep_target.forward::<COLOR>();
                if self.board.squares[ep_dest as usize].is_none() && (target.has(ep_dest) | target.has(ep_target)) {
                    let capturing_pawns = EP_FROM_SQUARES[(ep_target - A4) as usize] & self.board.bitboards[PAWN] & self.board.pieces[COLOR];
                    for from_square in BitIter::from(capturing_pawns) {
                        self.moves.push(Move::new_base(from_square as Square, ep_dest).with_infos(MoveInfo::EnPassantCapture));
                    }
                }
            }
        }

        // TODO Promotion

    }
}

// line is computed with intersection of two bishop attacks
// If the two squares are not on a line, the bitboard is empty
fn line(sq1: Square, sq2: Square) -> Bitboard {
    if sq1.rank() == sq2.rank() {
        RANKS[sq1.rank() as usize]
    } else if sq1.file() == sq2.file() {
        FILES[sq1.file() as usize]
    } else if (sq1.file() - sq2.file()).abs() == (sq1.rank() - sq2.rank()).abs() {
        (bishop_attack(sq1, EMPTY) & bishop_attack(sq2, EMPTY)).set(sq1).set(sq2)
    } else { EMPTY }
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

fn initialize_ep_from_squares() -> [Bitboard; 16] {
    let mut ep_from = [EMPTY; 16];
    for rank in [4, 5] {
        for file in FILE_LIST {
            let square = square_from_name(file, rank);
            let mut bb = EMPTY;
            if square.file() != A {
                bb = bb.set(square-1);
            }
            if square.file() != H {
                bb = bb.set(square+1);
            }
            ep_from[(square-A4) as usize] = bb;
        }
    }
    ep_from
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perft<const IS_ROOT: bool>(board: &mut Board, depth: usize) -> usize {
        if depth == 1 {
            let moves = board.legal_move_gen();
            if IS_ROOT {
                for m in &moves {
                    println!("{}{}: {}", m.from().debug(), m.to().debug(), 1);
                }
            }
            moves.len()
        } else {
            let mut count = 0;
            for to_play in board.legal_move_gen() {
                let ext_move = board.make(to_play);
                let local_count = perft::<false>(board, depth-1);
                if IS_ROOT {
                    println!("{}{}: {}", to_play.from().debug(), to_play.to().debug(), local_count);
                }
                count += local_count;
                board.unmake(ext_move);
            }
            count
        }
    }

    #[test]
    fn perft_base() {
        let mut board = Board::new();
        assert_eq!(perft::<true>(&mut board, 1), 20);
        assert_eq!(perft::<true>(&mut board, 2), 400);
        assert_eq!(perft::<true>(&mut board, 3), 8_902);
        assert_eq!(perft::<true>(&mut board, 4), 197_281);
        assert_eq!(perft::<true>(&mut board, 5), 4_865_609);
    }

    #[test]
    #[ignore]
    fn perft_base_expensive() {
        let mut board: Board = Board::new();
        assert_eq!(perft::<true>(&mut board, 6), 119_060_324);
    }

    #[test]
    fn perft_pos3() {
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("Invalid fen");
        //assert_eq!(perft::<true>(&mut board, 1), 14);
        //assert_eq!(perft::<true>(&mut board, 2), 191);
        //assert_eq!(perft::<true>(&mut board, 3), 2_812);
        //assert_eq!(perft::<true>(&mut board, 4), 43_238);
        assert_eq!(perft::<true>(&mut board, 5), 674_624);
    }
}