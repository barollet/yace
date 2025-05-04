pub use crate::square::*;

mod square;

pub type Color = bool;

pub const WHITE: Color = false;
pub const BLACK: Color = true;

pub type Bitboard = u64;

pub const EMPTY: Bitboard = 0;
pub const FULL: Bitboard = !EMPTY;

pub const RANK1: Bitboard = 0xff;
pub const RANK2: Bitboard = RANK1 << 8;
pub const RANK3: Bitboard = RANK1 << 16;
pub const RANK4: Bitboard = RANK1 << 24;
pub const RANK5: Bitboard = RANK1 << 32;
pub const RANK6: Bitboard = RANK1 << 40;
pub const RANK7: Bitboard = RANK1 << 48;
pub const RANK8: Bitboard = RANK1 << 56;

pub const FILEA: Bitboard = 0x0101010101010101;
pub const FILEH: Bitboard = FILEA << 7;

pub trait BitboardExt {
    fn set(self, pos: Square) -> Self;
    fn unset(self, pos: Square) -> Self;
    fn has(self, pos: Square) -> bool;
    fn to_string(self) -> String;
    fn lsb(self) -> Square;
    fn between(sq1: Square, sq2: Square) -> Self;
    fn forward<const COLOR: bool>(self) -> Self;
    fn forward_left<const COLOR: bool>(self) -> Self;
    fn forward_right<const COLOR: bool>(self) -> Self;
    fn display(self);
}

impl BitboardExt for u64 {
    fn set(self, pos: Square) -> Self {
        self | (1 << pos)
    }

    fn unset(self, pos: Square) -> Self {
        self & !(1 << pos)
    }
    
    fn has(self, pos: Square) -> bool {
        self & (1 << pos) != 0
    }

    fn to_string(self) -> String {
        let mut output = String::with_capacity(72);
        for rank in RANK_LIST.into_iter().rev() {
            for file in FILE_LIST {
                if self.has(square_from_name(file, rank)) {
                    output.push('1');
                } else {
                    output.push('0');
                }
            }
            output.push('\n');
        }
        output
    }

    fn display(self) {
        println!("{}", self.to_string())
    }
    
    fn lsb(self) -> Square {
        self.trailing_zeros() as Square
    }
    
    fn between(sq1: Square, sq2: Square) -> Self {
        let mut bb = sq1.as_bitboard();
        let file_diff= (sq2.file() - sq1.file()).clamp(-1, 1);
        let rank_diff = (sq2.rank() - sq1.rank()).clamp(-1, 1);

        let mut sq = sq1;
        while sq != sq2 {
            sq = Square::new(sq.file() + file_diff, sq.rank() + rank_diff);
            bb |= sq.as_bitboard();
        }

        bb
    }
    
    fn forward<const COLOR: bool>(self) -> Self {
        if COLOR == WHITE {
            self << 8
        } else {
            self >> 8
        }
    }
    
    fn forward_left<const COLOR: bool>(self) -> Self {
        if COLOR == WHITE {
            (self & !FILEA) << 7 
        } else {
            (self & !FILEH) >> 7 
        }
    }
    
    fn forward_right<const COLOR: bool>(self) -> Self {
        if COLOR == WHITE {
            (self & !FILEH) << 9 
        } else {
            (self & !FILEA) >> 9 
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let bb = EMPTY;
        assert_eq!(bb.set(5), 0b100000);
        assert_eq!(bb.set(5).set(3), 0b101000);
        assert_eq!(bb.set(0), 0b1);
    }

    #[test]
    fn test_unset() {
        let bb = EMPTY.set(5).set(3);
        assert_eq!(bb.unset(4), bb);
        assert_eq!(bb.unset(5), 0b1000);
    }

    #[test]
    fn test_has() {
        assert!(EMPTY.set(5).set(3).has(5));
        assert!(!EMPTY.set(5).set(3).has(4));
    }

    #[test]
    fn test_display() {
        assert_eq!(EMPTY.set(5).to_string(), "00000000\n00000000\n00000000\n00000000\n00000000\n00000000\n00000000\n00000100\n")
    }

}
