use crate::evaluation::IncrementalEval;

pub use self::piece::*;
pub use self::bitboard::*;
pub use self::move_gen::*;
pub use self::moves::*;
pub use self::square::*;

use enum_indexed::*;
use zobrist::ZobristHash;
use zobrist::ZobristHasher;

pub mod bitboard;
pub mod piece;
pub mod fen;
pub mod square;
pub mod magic_table;
pub mod move_gen;
pub mod moves;

mod zobrist;
mod generate_magic;
mod enum_indexed;

pub type CastlingRights = u8;

#[derive(Clone, Debug)]
pub struct Board {
    pub pieces: ColorIndexed<Bitboard>,
    pub bitboards: PieceIndexed<Bitboard>,
    pub squares: [Option<Piece>; 64],
    castling_rights: CastlingRights,
    ep_target: Option<Square>,
    pub to_move: Color,

    pub evaluation: IncrementalEval,
    pub zobrist_hash: ZobristHash,
}

pub type CastlingSide = bool;
pub const KINGSIDE: CastlingSide = false;
pub const QUEENSIDE: CastlingSide = true;

impl Board {
    pub fn new() -> Self {
        let mut board = Self::empty();

        let backrank = [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK];
        for (&file, piece) in FILE_LIST.iter().zip(backrank) {
            board.add_piece(PAWN, square_from_name(file, 2), WHITE);
            board.add_piece(PAWN, square_from_name(file, 7), BLACK);
            board.add_piece(piece, square_from_name(file, 1), WHITE);
            board.add_piece(piece, square_from_name(file, 8), BLACK);
        }
        
        board
    }

    pub fn empty() -> Self {
        Board {
            pieces: ColorIndexed::new(),
            bitboards: PieceIndexed::new(),
            squares: [None; 64],
            ep_target: None,
            castling_rights: CastlingRights::new(),
            to_move: WHITE,

            evaluation: IncrementalEval::new(),
            zobrist_hash: ZobristHasher::new_hash(),
        }
    }

    // Move must be legal
    pub fn make(&mut self, to_play: Move) -> ExtendedMove {
        let past_ep_state = self.ep_target;
        let past_castle = self.castling_rights;
        self.zobrist_hash.handle_castling(past_castle);

        // For en passant the captured piece is not set to pawn as we already know it
        let captured_piece = match to_play.infos() {
            MoveInfo::Capture | MoveInfo::CapturePromotion(_) => {
                // remove castling rights if ending on a rook starting square
                if to_play.to() == ROOK_CASTLING_START[CastlingRights::index(!self.to_move, KINGSIDE)] {
                    self.castling_rights.remove(!self.to_move, KINGSIDE);
                } else if to_play.to() == ROOK_CASTLING_START[CastlingRights::index(!self.to_move, QUEENSIDE)] {
                    self.castling_rights.remove(!self.to_move, QUEENSIDE);
                }

                Some(self.remove_piece(to_play.to(), !self.to_move))
            },
            _ => None
        };

        if to_play.infos() == MoveInfo::EnPassantCapture {
            if self.to_move == WHITE {
                self.remove_piece(to_play.to().backward::<WHITE>(), BLACK);
            } else {
                self.remove_piece(to_play.to().backward::<BLACK>(), WHITE);
            }
        }


        // Actual movement
        match to_play.infos() {
            MoveInfo::Promotion(prom_piece) | MoveInfo::CapturePromotion(prom_piece) => {
                self.remove_piece(to_play.from(), self.to_move);
                self.add_piece(prom_piece, to_play.to(), self.to_move);
            },
            _ => {
                self.move_piece(to_play.from(), to_play.to(), self.to_move);
            }
        };

        // Move rook on castling
        match to_play.infos() {
            MoveInfo::KingCastle => {
                let index = CastlingRights::index(self.to_move, KINGSIDE);
                self.move_piece(ROOK_CASTLING_START[index], ROOK_CASTLING_DEST[index], self.to_move);
            },
            MoveInfo::QueenCastle => {
                let index = CastlingRights::index(self.to_move, QUEENSIDE);
                self.move_piece(ROOK_CASTLING_START[index], ROOK_CASTLING_DEST[index], self.to_move);
            },
            _ => (),
        }
        
        // If king moves both castling rights are removed, only one if this is a rook
        // A rook being captured removes the castling right
        if self.squares[to_play.to() as usize] == Some(KING) {
            self.castling_rights.remove(self.to_move, KINGSIDE);
            self.castling_rights.remove(self.to_move, QUEENSIDE);
        } else if to_play.from() == ROOK_CASTLING_START[CastlingRights::index(self.to_move, KINGSIDE)] {
            self.castling_rights.remove(self.to_move, KINGSIDE);
        } else if to_play.from() == ROOK_CASTLING_START[CastlingRights::index(self.to_move, QUEENSIDE)] {
            self.castling_rights.remove(self.to_move, QUEENSIDE);
        }

        // Setting ep state on double push
        self.zobrist_hash.handle_ep(self.ep_target);
        if to_play.infos() == MoveInfo::DoublePawnPush {
            self.ep_target = Some(to_play.to());
        } else {
            self.ep_target = None;
        }
        self.zobrist_hash.handle_ep(self.ep_target);

        self.zobrist_hash.handle_castling(self.castling_rights);

        self.to_move = !self.to_move;
        self.zobrist_hash.handle_side_to_move();

        ExtendedMove::new_base(to_play, captured_piece, past_ep_state, past_castle)
    }

    pub fn unmake(&mut self, ext_move: ExtendedMove) {
        self.to_move = !self.to_move;
        self.zobrist_hash.handle_side_to_move();

        // Remove and restore
        self.zobrist_hash.handle_ep(self.ep_target);
        self.ep_target = ext_move.infos().past_epstate();
        self.zobrist_hash.handle_ep(self.ep_target);

        self.zobrist_hash.handle_castling(self.castling_rights);
        self.castling_rights = ext_move.infos().past_castle();
        self.zobrist_hash.handle_castling(self.castling_rights);

        // Restore old rook position on castle
        match ext_move.base_move().infos() {
            MoveInfo::KingCastle => {
                let index = CastlingRights::index(self.to_move, KINGSIDE);
                self.move_piece(ROOK_CASTLING_DEST[index], ROOK_CASTLING_START[index], self.to_move);
            },
            MoveInfo::QueenCastle => {
                let index = CastlingRights::index(self.to_move, QUEENSIDE);
                self.move_piece(ROOK_CASTLING_DEST[index], ROOK_CASTLING_START[index], self.to_move);
            },
            _ => (),
        }

        // Actual movement
        match ext_move.base_move().infos() {
            MoveInfo::Promotion(_) | MoveInfo::CapturePromotion(_) => {
                self.remove_piece(ext_move.base_move().to(), self.to_move);
                self.add_piece(PAWN, ext_move.base_move().from(), self.to_move);
            },
            _ => {
                self.move_piece(ext_move.base_move().to(), ext_move.base_move().from(), self.to_move);
            }
        };


        if ext_move.base_move().infos() == MoveInfo::EnPassantCapture {
            if self.to_move == WHITE {
                self.add_piece(PAWN, ext_move.base_move().to().backward::<WHITE>(), BLACK);
            } else {
                self.add_piece(PAWN, ext_move.base_move().to().backward::<BLACK>(), WHITE);
            }
        }

        match ext_move.base_move().infos() {
            MoveInfo::Capture | MoveInfo::CapturePromotion(_) => {
                self.add_piece(ext_move.infos().captured_piece().unwrap(), ext_move.base_move().to(), !self.to_move);
            },
            _ => {}
        }
    }

    fn add_piece(&mut self, piece: Piece, sq: Square, color: Color) {
        self.squares[sq as usize] = Some(piece);
        self.bitboards[piece] |= sq.as_bitboard();
        self.pieces[color] |= sq.as_bitboard();

        self.evaluation.add_piece(piece, sq, color);
        self.zobrist_hash.handle_piece(sq, piece, color);
    }

    fn remove_piece(&mut self, sq: Square, color: Color) -> Piece {
        let piece = self.squares[sq as usize].unwrap();
        self.squares[sq as usize] = None;
        self.bitboards[piece] &= !sq.as_bitboard();
        self.pieces[color] &= !sq.as_bitboard();

        self.evaluation.remove_piece(piece, sq, color);
        self.zobrist_hash.handle_piece(sq, piece, color);

        piece
    }

    fn move_piece(&mut self, from: Square, to: Square, color: Color) {
        let piece = self.squares[from as usize].unwrap();
        self.squares[to as usize] = self.squares[from as usize];
        self.squares[from as usize] = None;

        self.bitboards[piece] &= !from.as_bitboard();
        self.bitboards[piece] |= to.as_bitboard();

        self.pieces[color] &= !from.as_bitboard();        
        self.pieces[color] |= to.as_bitboard();

        self.evaluation.move_piece(piece, from, to, color);
        self.zobrist_hash.handle_piece(from, piece, color);
        self.zobrist_hash.handle_piece(to, piece, color);
    }

    fn king_square(&self, color: Color) -> Square {
        (self.bitboards[KING] & self.pieces[color]).lsb()
    }

    pub fn occupancy(&self) -> Bitboard {
        self.pieces[WHITE] | self.pieces[BLACK]
    }

    fn checkers<const COLOR: bool>(&self) -> Bitboard {
        self.square_attacked_by::<COLOR>(self.king_square(COLOR))
    }

    pub fn display(&self) {
        for rank in RANK_LIST.into_iter().rev() {
            for file in FILE_LIST {
                let square = square_from_name(file, rank);
                if let Some(piece) = self.squares[square as usize] {
                    if self.pieces[WHITE].has(square) {
                        print!("{}", char::from(piece).to_uppercase())
                    } else {
                        print!("{}", char::from(piece))
                    }
                } else {
                    print!(".")
                }
            }
            println!()
        }
    }
}

pub trait CastlingRightsExt {
    fn new() -> Self;
    fn index(color: Color, side: CastlingSide) -> usize;
    fn remove(&mut self, color: Color, side: CastlingSide);
    fn restore(&mut self, color: Color, side: CastlingSide);
    fn has(&self, color: Color, side: CastlingSide) -> bool;
}

impl CastlingRightsExt for CastlingRights {
    fn new() -> Self {
        0xf
    }

    fn remove(&mut self, color: Color, side: CastlingSide) {
        let mask = 1 << Self::index(color, side);
        *self &= !mask;
    }
    
    fn restore(&mut self, color: Color, side: CastlingSide) {
        let mask = 1 << Self::index(color, side);
        *self |= mask;
    }
    
    fn has(&self, color: Color, side: CastlingSide) -> bool {
        let mask = 1 << Self::index(color, side);
        self & mask != 0
    }
    
    #[inline]
    fn index(color: Color, side: CastlingSide) -> usize {
        2*color as usize + side as usize
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castle() {
        let mut castling = CastlingRights::new();

        assert!(castling.has(WHITE, KINGSIDE));
        assert!(castling.has(WHITE, QUEENSIDE));
        assert!(castling.has(BLACK, KINGSIDE));
        assert!(castling.has(BLACK, QUEENSIDE));

        castling.remove(WHITE, QUEENSIDE);
        assert!(castling.has(WHITE, KINGSIDE));
        assert!(!castling.has(WHITE, QUEENSIDE));
        assert!(castling.has(BLACK, KINGSIDE));
        assert!(castling.has(BLACK, QUEENSIDE));

        castling.remove(BLACK, KINGSIDE);
        assert!(castling.has(WHITE, KINGSIDE));
        assert!(!castling.has(WHITE, QUEENSIDE));
        assert!(!castling.has(BLACK, KINGSIDE));
        assert!(castling.has(BLACK, QUEENSIDE));

        castling.restore(WHITE, QUEENSIDE);
        assert!(castling.has(WHITE, KINGSIDE));
        assert!(castling.has(WHITE, QUEENSIDE));
        assert!(!castling.has(BLACK, KINGSIDE));
        assert!(castling.has(BLACK, QUEENSIDE));
    }
}