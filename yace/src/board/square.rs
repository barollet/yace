use super::*;

pub const A: i8 = 0;
pub const B: i8 = 1;
pub const C: i8 = 2;
pub const D: i8 = 3;
pub const E: i8 = 4;
pub const F: i8 = 5;
pub const G: i8 = 6;
pub const H: i8 = 7;

pub const FILE_LIST: [i8; 8] = [A, B, C, D, E, F, G, H];
pub const RANK_LIST: [i8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];

pub type Square = i8;

pub const A1: Square = square_from_name(A, 1);
pub const A2: Square = square_from_name(A, 2);
pub const A3: Square = square_from_name(A, 3);
pub const A4: Square = square_from_name(A, 4);
pub const A5: Square = square_from_name(A, 5);
pub const A6: Square = square_from_name(A, 6);
pub const A7: Square = square_from_name(A, 7);
pub const A8: Square = square_from_name(A, 8);
pub const B1: Square = square_from_name(B, 1);
pub const B2: Square = square_from_name(B, 2);
pub const B3: Square = square_from_name(B, 3);
pub const B4: Square = square_from_name(B, 4);
pub const B5: Square = square_from_name(B, 5);
pub const B6: Square = square_from_name(B, 6);
pub const B7: Square = square_from_name(B, 7);
pub const B8: Square = square_from_name(B, 8);
pub const C1: Square = square_from_name(C, 1);
pub const C2: Square = square_from_name(C, 2);
pub const C3: Square = square_from_name(C, 3);
pub const C4: Square = square_from_name(C, 4);
pub const C5: Square = square_from_name(C, 5);
pub const C6: Square = square_from_name(C, 6);
pub const C7: Square = square_from_name(C, 7);
pub const C8: Square = square_from_name(C, 8);
pub const D1: Square = square_from_name(D, 1);
pub const D2: Square = square_from_name(D, 2);
pub const D3: Square = square_from_name(D, 3);
pub const D4: Square = square_from_name(D, 4);
pub const D5: Square = square_from_name(D, 5);
pub const D6: Square = square_from_name(D, 6);
pub const D7: Square = square_from_name(D, 7);
pub const D8: Square = square_from_name(D, 8);
pub const E1: Square = square_from_name(E, 1);
pub const E2: Square = square_from_name(E, 2);
pub const E3: Square = square_from_name(E, 3);
pub const E4: Square = square_from_name(E, 4);
pub const E5: Square = square_from_name(E, 5);
pub const E6: Square = square_from_name(E, 6);
pub const E7: Square = square_from_name(E, 7);
pub const E8: Square = square_from_name(E, 8);
pub const F1: Square = square_from_name(F, 1);
pub const F2: Square = square_from_name(F, 2);
pub const F3: Square = square_from_name(F, 3);
pub const F4: Square = square_from_name(F, 4);
pub const F5: Square = square_from_name(F, 5);
pub const F6: Square = square_from_name(F, 6);
pub const F7: Square = square_from_name(F, 7);
pub const F8: Square = square_from_name(F, 8);
pub const G1: Square = square_from_name(G, 1);
pub const G2: Square = square_from_name(G, 2);
pub const G3: Square = square_from_name(G, 3);
pub const G4: Square = square_from_name(G, 4);
pub const G5: Square = square_from_name(G, 5);
pub const G6: Square = square_from_name(G, 6);
pub const G7: Square = square_from_name(G, 7);
pub const G8: Square = square_from_name(G, 8);
pub const H1: Square = square_from_name(H, 1);
pub const H2: Square = square_from_name(H, 2);
pub const H3: Square = square_from_name(H, 3);
pub const H4: Square = square_from_name(H, 4);
pub const H5: Square = square_from_name(H, 5);
pub const H6: Square = square_from_name(H, 6);
pub const H7: Square = square_from_name(H, 7);
pub const H8: Square = square_from_name(H, 8);

pub const fn square_from_name(file: i8, rank: i8) -> Square {
    8*(rank-1) + file
}

pub trait SquareExt {
    fn new(file: i8, rank: i8) -> Self;
    fn file(self) -> i8;
    fn rank(self) -> i8;
    fn forward<const COLOR: bool>(self) -> Square;
    fn backward<const COLOR: bool>(self) -> Square;
    fn forward_left<const COLOR: bool>(self) -> Option<Square>;
    fn forward_right<const COLOR: bool>(self) -> Option<Square>;
    fn backward_left<const COLOR: bool>(self) -> Option<Square>;
    fn backward_right<const COLOR: bool>(self) -> Option<Square>;
    fn vertical_symmetry(self) -> Square;
    fn as_bitboard(self) -> Bitboard;
    fn debug(self) -> String;
}

impl SquareExt for Square {
    fn new(file: i8, rank: i8) -> Self {
        8*rank+file
    }

    fn as_bitboard(self) -> Bitboard {
        1 << self
    }
    
    fn file(self) -> i8 {
        self % 8
    }
    
    fn rank(self) -> i8 {
        self / 8
    }
    
    fn debug(self) -> String {
        let mut output = String::with_capacity(2);
        output.push(match self.file() {
            0 => 'A',
            1 => 'B',
            2 => 'C',
            3 => 'D',
            4 => 'E',
            5 => 'F',
            6 => 'G',
            7 => 'H',
            _ => panic!("Invalid file for square")
        });
        output.push(char::from_digit(self.rank() as u32+1, 10).expect("Invalid rank for square"));

        output
    }
    
    fn backward<const COLOR: bool>(self) -> Square {
        if COLOR == WHITE {
            self - 8
        } else {
            self + 8
        }
    }

    fn forward<const COLOR: bool>(self) -> Square {
        if COLOR == WHITE {
            self + 8
        } else {
            self - 8
        }
    }
    
    fn backward_left<const COLOR: bool>(self) -> Option<Square> {
        if COLOR == WHITE {
            if self.file() != H && self.rank() > 0 {
                Some(self - 7)
            } else {
                None
            }
        } else if self.file() != A && self.rank() < 7 {
            Some(self + 7)
        } else {
            None
        }
    }
    
    fn backward_right<const COLOR: bool>(self) -> Option<Square> {
        if COLOR == WHITE {
            if self.file() != A && self.rank() > 0 {
                Some(self - 9)
            } else {
                None
            }
        } else if self.file() != H && self.rank() < 7 {
            Some(self + 9)
        } else {
            None
        }
    }
    
    fn forward_left<const COLOR: bool>(self) -> Option<Square> {
        if COLOR == WHITE {
            if self.file() != A && self.rank() < 7 {
                Some(self + 7)
            } else {
                None
            }
        } else if self.file() != H && self.rank() > 0 {
            Some(self - 7)
        } else {
            None
        }
    }
    
    fn forward_right<const COLOR: bool>(self) -> Option<Square> {
        if COLOR == WHITE {
            if self.file() != H && self.rank() < 7 {
                Some(self + 9)
            } else {
                None
            }
        } else if self.file() != A && self.rank() > 0 {
            Some(self - 9)
        } else {
            None
        }
    }

    fn vertical_symmetry(self) -> Square {
        Self::new(self.file(), 7-self.rank())
    }
}
