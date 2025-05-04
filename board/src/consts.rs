use enum_ext::enum_extend;

pub type Color = bool;

pub const WHITE: Color = false;
pub const BLACK: Color = true;

#[derive(Debug, Copy, PartialEq)]
#[enum_extend(IntType = "u8")]
pub enum Piece {
    Queen = 0,
    Rook = 1,
    Bishop = 2,
    Knight = 3,
    Pawn = 4,
    King = 5,
}

pub const PAWN: Piece = Piece::Pawn;
pub const KNIGHT: Piece = Piece::Knight;
pub const BISHOP: Piece = Piece::Bishop;
pub const ROOK: Piece = Piece::Rook;
pub const QUEEN: Piece = Piece::Queen;
pub const KING: Piece = Piece::King;

impl From<Piece> for usize {
    fn from(value: Piece) -> Self {
        value.as_u8() as usize
    }
}

impl From<Piece> for char {
    fn from(value: Piece) -> Self {
        match value {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }
}