use crate::board::magic_table::bishop_attack;
pub use crate::board::square::*;

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
pub const RANKS: [Bitboard; 8] = [RANK1, RANK2, RANK3, RANK4, RANK5, RANK6, RANK7, RANK8];

pub const FILEA: Bitboard = 0x0101010101010101;
pub const FILEB: Bitboard = FILEA << 1;
pub const FILEC: Bitboard = FILEA << 2;
pub const FILED: Bitboard = FILEA << 3;
pub const FILEE: Bitboard = FILEA << 4;
pub const FILEF: Bitboard = FILEA << 5;
pub const FILEG: Bitboard = FILEA << 6;
pub const FILEH: Bitboard = FILEA << 7;
pub const FILES: [Bitboard; 8] = [FILEA, FILEB, FILEC, FILED, FILEE, FILEF, FILEG, FILEH];

pub trait BitboardExt {
    fn set(self, pos: Square) -> Self;
    fn unset(self, pos: Square) -> Self;
    fn has(self, pos: Square) -> bool;
    fn to_string(self) -> String;
    fn lsb(self) -> Square;
    fn between(sq1: Square, sq2: Square) -> Self;
    fn line(sq1: Square, sq2: Square) -> Self;
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
    
    // Bounds are excluded
    fn between(sq1: Square, sq2: Square) -> Self {
        let mut bb = sq1.as_bitboard();
        let file_diff= sq2.file() - sq1.file();
        let rank_diff = sq2.rank() - sq1.rank();
        if file_diff == 0 || rank_diff == 0 || file_diff.abs() == rank_diff.abs() {
            let file_diff= file_diff.clamp(-1, 1);
            let rank_diff = rank_diff.clamp(-1, 1);
    
            let mut next_sq = sq1;
            while next_sq != sq2 {
                bb |= next_sq.as_bitboard();
                next_sq = Square::new(next_sq.file() + file_diff, next_sq.rank() + rank_diff);
            }
        }

        bb
    }

    // line is computed with intersection of two bishop attacks
    // If the two squares are not on a line, the bitboard is empty
    fn line(sq1: Square, sq2: Square) -> Self {
        if sq1.rank() == sq2.rank() {
            RANKS[sq1.rank() as usize]
        } else if sq1.file() == sq2.file() {
            FILES[sq1.file() as usize]
        } else if (sq1.file() - sq2.file()).abs() == (sq1.rank() - sq2.rank()).abs() {
            (bishop_attack(sq1, EMPTY) & bishop_attack(sq2, EMPTY)).set(sq1).set(sq2)
        } else { EMPTY }
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
