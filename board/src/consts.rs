use enum_ext::enum_extend;

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

pub const KNIGHT_ORDINAL: u8 = KNIGHT.ordinal() as u8;
pub const BISHOP_ORDINAL: u8 = BISHOP.ordinal() as u8;
pub const ROOK_ORDINAL: u8 = ROOK.ordinal() as u8;
pub const QUEEN_ORDINAL: u8 = QUEEN.ordinal() as u8;
pub const KING_ORDINAL: u8 = KING.ordinal() as u8;

impl Piece {
    pub const fn value(&self) -> u16 {
        match self {
            Piece::Pawn => 100,
            Piece::Knight => 325,
            Piece::Bishop => 325,
            Piece::Rook => 500,
            Piece::Queen => 1000,
            Piece::King => 5000,
        }
    }
}

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