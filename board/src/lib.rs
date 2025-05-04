pub use bitboard::*;
use consts::*;
use enum_indexed::*;

mod consts;
mod enum_indexed;
pub mod magic_table;
mod move_gen;
mod moves;

#[derive(Clone, Debug)]
pub struct Board {
    pieces: ColorIndexed<Bitboard>,
    bitboards: PieceIndexed<Bitboard>,
    squares: [Option<Piece>; 64],
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
            to_move: WHITE,
        }
    }

    pub fn display(&self) {
        for rank in RANK_LIST.into_iter().rev() {
            for file in FILE_LIST {
                let square = Square::new(file, rank);
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