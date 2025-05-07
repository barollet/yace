use std::fmt::Debug;

use crate::bitboard::*;
use bitfield_struct::bitfield;

use crate::consts::*;

#[bitfield(u16, debug=false)]
pub struct Move {
    #[bits(6, from = std::convert::identity, into = std::convert::identity)]
    pub from: i8,
    #[bits(6, from = std::convert::identity, into = std::convert::identity)]
    pub to: i8,
    #[bits(4)]
    pub infos: MoveInfo,
}

#[bitfield(u32)]
pub struct ExtendedMove {
    #[bits(16)]
    pub base_move: Move,
    #[bits(16)]
    pub infos: ExtMoveInfo,
}

impl Move {
    pub fn new_base(from: Square, to: Square) -> Self {
        Self::new().with_from(from).with_to(to)
    }
}

#[derive(Debug, PartialEq)]
pub enum MoveInfo {
    Quiet,
    DoublePawnPush,
    KingCastle,
    QueenCastle,
    Capture,
    EnPassantCapture,
    Promotion(Piece),
    CapturePromotion(Piece),
}

#[bitfield(u16, debug=false)]
#[derive(Debug, PartialEq)]
pub struct ExtMoveInfo {
    #[bits(3, from = captured_from_bits, into = captured_into_bits)]
    pub captured_piece: Option<Piece>,
    #[bits(5, from = ep_from_bits, into = ep_into_bits)]
    pub past_epstate: Option<Square>,
    #[bits(8)]
    __: u8,
}

#[bitfield(u8)]
struct InfoStruct {
    special0: bool,
    special1: bool,
    capture: bool,
    promotion: bool,
    #[bits(4)]
    __: u8,
}

impl MoveInfo {
    const fn into_bits(self) -> u8 {
        match self {
            MoveInfo::Quiet => 0,
            MoveInfo::DoublePawnPush => InfoStruct::new().with_special0(true).into_bits(),
            MoveInfo::KingCastle => InfoStruct::new().with_special1(true).into_bits(),
            MoveInfo::QueenCastle => InfoStruct::new().with_special1(true).with_special0(true).into_bits(),
            MoveInfo::Capture => InfoStruct::new().with_capture(true).into_bits(),
            MoveInfo::EnPassantCapture => InfoStruct::new().with_capture(true).with_special0(true).into_bits(),

            MoveInfo::Promotion(piece) => 0b1000 + piece.ordinal() as u8,
            MoveInfo::CapturePromotion(piece) => 0b1100 + piece.ordinal() as u8,
        }
    }

    const fn from_bits(value: u8) -> Self {
        let infos = InfoStruct::from_bits(value);

        if infos.promotion() {
            let piece = Piece::from_u8(value & 0b11).unwrap();
            if infos.capture() {
                MoveInfo::CapturePromotion(piece)
            } else {
                MoveInfo::Promotion(piece)
            }
        } else if infos.capture() {
            if infos.special0() {
                MoveInfo::EnPassantCapture
            } else {
                MoveInfo::Capture
            }
        } else {
            match (infos.special1(), infos.special0()) {
                (false, false) => MoveInfo::Quiet,
                (false, true) => MoveInfo::DoublePawnPush,
                (true, false) => MoveInfo::KingCastle,
                (true, true) => MoveInfo::QueenCastle,
            }
        }
    }
}

impl ExtendedMove {
    pub fn new_base(base_move: Move, captured_piece: Option<Piece>, ep_state: Option<Square>) -> Self {
        Self::new().with_base_move(base_move)
            .with_infos(ExtMoveInfo::new().with_captured_piece(captured_piece).with_past_epstate(ep_state))
    }
}

const fn captured_from_bits(value: u8) -> Option<Piece> {
    Piece::from_u8(value &0b111)
}

const fn captured_into_bits(piece: Option<Piece>) -> u8 {
    if let Some(piece) = piece {
        piece.ordinal() as u8
    } else {
        0b111
    }
}

const fn ep_from_bits(value: i8) -> Option<Square> {
    if value & 0b10000 != 0 {
        Some((value & 0b1111) + A4)
    } else {
        None
    }
}

const fn ep_into_bits(ep: Option<Square>) -> i8 {
    if let Some(ep) = ep {
        0b10000 + (ep - A4)
    } else {
        0
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Move").field("from", &self.from().debug()).field("to", &self.to().debug()).field("infos", &self.infos()).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::bitboard::*;
    use crate::consts::*;

    use super::*;

    #[test]
    fn test_infos() {
        assert_eq!(Move::new().with_infos(MoveInfo::Quiet).infos(), MoveInfo::Quiet);
        assert_eq!(Move::new().with_infos(MoveInfo::DoublePawnPush).infos(), MoveInfo::DoublePawnPush);
        assert_eq!(Move::new().with_infos(MoveInfo::KingCastle).infos(), MoveInfo::KingCastle);
        assert_eq!(Move::new().with_infos(MoveInfo::QueenCastle).infos(), MoveInfo::QueenCastle);
        assert_eq!(Move::new().with_infos(MoveInfo::Capture).infos(), MoveInfo::Capture);
        assert_eq!(Move::new().with_infos(MoveInfo::EnPassantCapture).infos(), MoveInfo::EnPassantCapture);

        assert_eq!(Move::new().with_infos(MoveInfo::Promotion(KNIGHT)).infos(), MoveInfo::Promotion(KNIGHT));
        assert_eq!(Move::new().with_infos(MoveInfo::Promotion(BISHOP)).infos(), MoveInfo::Promotion(BISHOP));
        assert_eq!(Move::new().with_infos(MoveInfo::Promotion(ROOK)).infos(), MoveInfo::Promotion(ROOK));
        assert_eq!(Move::new().with_infos(MoveInfo::Promotion(QUEEN)).infos(), MoveInfo::Promotion(QUEEN));

        assert_eq!(Move::new().with_infos(MoveInfo::CapturePromotion(KNIGHT)).infos(), MoveInfo::CapturePromotion(KNIGHT));
        assert_eq!(Move::new().with_infos(MoveInfo::CapturePromotion(BISHOP)).infos(), MoveInfo::CapturePromotion(BISHOP));
        assert_eq!(Move::new().with_infos(MoveInfo::CapturePromotion(ROOK)).infos(), MoveInfo::CapturePromotion(ROOK));
        assert_eq!(Move::new().with_infos(MoveInfo::CapturePromotion(QUEEN)).infos(), MoveInfo::CapturePromotion(QUEEN));
    }

    #[test]
    fn test_from_to() {
        let m = Move::new().with_from(A3).with_to(D7);
        assert_eq!(m.from(), A3);
        assert_eq!(m.to(), D7);
    }
}