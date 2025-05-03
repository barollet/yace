pub use bitboard::*;
use consts::*;
use enum_indexed::{ColorIndexed, PieceIndexed};

mod consts;
mod enum_indexed;
mod move_gen;
mod moves;

#[derive(Clone, Debug, Default)]
pub struct Board {
    pieces: ColorIndexed<Bitboard>,
    bitboards: PieceIndexed<Bitboard>,
    to_move: Color,
}

impl Board {
    pub fn new() -> Self {
        let mut pieces = ColorIndexed::new();
        pieces[WHITE] = RANK1 | RANK2;
        pieces[BLACK] = RANK7 | RANK8;

        let mut bitboards = PieceIndexed::new();
        bitboards[PAWN] = RANK2 | RANK7;
        bitboards[KNIGHT] = B1.to_bitboard() | G1.to_bitboard() | B8.to_bitboard() | G8.to_bitboard();
        bitboards[BISHOP] = C1.to_bitboard() | F1.to_bitboard() | C8.to_bitboard() | F8.to_bitboard();
        bitboards[ROOK] = A1.to_bitboard() | H1.to_bitboard() | A8.to_bitboard() | H8.to_bitboard();
        bitboards[QUEEN] = D1.to_bitboard() | D8.to_bitboard();
        bitboards[KING] = E1.to_bitboard() | E8.to_bitboard();

        Board {
            pieces,
            bitboards,
            to_move: WHITE,
        }
    }

    pub fn display(&self) {
        for rank in RANK_LIST.into_iter().rev() {
            for file in FILE_LIST {
                let square = Square::new(file, rank);
                if (self.pieces[WHITE] | self.pieces[BLACK]).has(square) {
                    let lowercase = self.pieces[BLACK].has(square);
                    let piece = Piece::from_ordinal(
                        self.bitboards.0.iter()
                        .position(|bb| bb.has(square))
                        .expect("No piece found on square"))
                    .expect("Cannot convert this number to a piece");
                    if lowercase {
                        print!("{}", char::from(piece))
                    } else {
                        print!("{}", char::from(piece).to_uppercase())
                    }
                } else {
                    print!(".")
                }
            }
            println!()
        }
    }
}
