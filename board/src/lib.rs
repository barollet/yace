pub use bitboard::*;
use consts::*;
use enum_indexed::*;
use moves::{ExtendedMove, Move, MoveInfo};

pub mod bitboard;
mod consts;
mod enum_indexed;
pub mod magic_table;
mod move_gen;
mod moves;
mod square;

#[derive(Clone, Debug)]
pub struct Board {
    pieces: ColorIndexed<Bitboard>,
    bitboards: PieceIndexed<Bitboard>,
    squares: [Option<Piece>; 64],
    ep_target: Option<Square>,
    to_move: Color,
}

impl Board {
    pub fn new() -> Self {
        let mut pieces = ColorIndexed::new();
        pieces[WHITE] = RANK1 | RANK2;
        pieces[BLACK] = RANK7 | RANK8;

        let mut bitboards = PieceIndexed::new();
        bitboards[PAWN] = RANK2 | RANK7;
        bitboards[KNIGHT] = B1.as_bitboard() | G1.as_bitboard() | B8.as_bitboard() | G8.as_bitboard();
        bitboards[BISHOP] = C1.as_bitboard() | F1.as_bitboard() | C8.as_bitboard() | F8.as_bitboard();
        bitboards[ROOK] = A1.as_bitboard() | H1.as_bitboard() | A8.as_bitboard() | H8.as_bitboard();
        bitboards[QUEEN] = D1.as_bitboard() | D8.as_bitboard();
        bitboards[KING] = E1.as_bitboard() | E8.as_bitboard();

        let mut squares = [None; 64];
        let backrank = [ROOK, KNIGHT, BISHOP, QUEEN, KING, BISHOP, KNIGHT, ROOK];
        for (&file, piece) in FILE_LIST.iter().zip(backrank) {
            squares[square_from_name(file, 2) as usize] = Some(PAWN);
            squares[square_from_name(file, 7) as usize] = Some(PAWN);
            squares[square_from_name(file, 1) as usize] = Some(piece);
            squares[square_from_name(file, 8) as usize] = Some(piece);
        }
        
        Board {
            pieces,
            bitboards,
            squares,
            ep_target: None,
            to_move: WHITE,
        }
    }

    // Move must be legal
    pub fn make(&mut self, to_play: Move) -> ExtendedMove {
        let past_ep_state = self.ep_target;

        // For en passant the captured piece is not set to pawn as we already know it
        let captured_piece = match to_play.infos() {
            MoveInfo::Capture | MoveInfo::CapturePromotion(_) => {
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

        let piece: Piece = self.remove_piece(to_play.from(), self.to_move);
        self.add_piece(piece, to_play.to(), self.to_move);

        // Setting ep state on double push
        if to_play.infos() == MoveInfo::DoublePawnPush {
            self.ep_target = Some(to_play.to())
        } else {
            self.ep_target = None
        }

        self.to_move = !self.to_move;

        ExtendedMove::new_base(to_play, captured_piece, past_ep_state)
    }

    pub fn unmake(&mut self, ext_move: ExtendedMove) {
        self.to_move = !self.to_move;
        self.ep_target = ext_move.infos().past_epstate();

        let piece: Piece = self.remove_piece(ext_move.base_move().to(), self.to_move);
        self.add_piece(piece, ext_move.base_move().from(), self.to_move);

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
    }

    fn remove_piece(&mut self, sq: Square, color: Color) -> Piece {
        let piece = self.squares[sq as usize].unwrap();
        self.squares[sq as usize] = None;
        self.bitboards[piece] &= !sq.as_bitboard();
        self.pieces[color] &= !sq.as_bitboard();
        piece
    }

    fn king_square(&self, color: Color) -> Square {
        (self.bitboards[KING] & self.pieces[color]).lsb()
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

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}