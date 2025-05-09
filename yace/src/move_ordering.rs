use std::cmp::max;

use arrayvec::ArrayVec;
use crate::board::*;

// TODO pinned pieces are still able to capture here
pub fn static_exchange_evaluation(board: &Board, from: Square, to: Square) -> i16 {
    let mut gain: ArrayVec<i16, 32> = ArrayVec::new();
    let mut occupancy = board.occupancy();
    let mut capturing = Some((from.as_bitboard(), board.squares[from as usize].unwrap()));
    let mut playing_color = board.to_move;

    let mut both_color_attackers = board.square_full_attacked_by(to, occupancy);
    let mut alread_seen_attackers = EMPTY;

    let may_cause_xray = board.bitboards[PAWN] | board.bitboards[BISHOP] | board.bitboards[ROOK] | board.bitboards[QUEEN];

    let target_piece = board.squares[to as usize].unwrap();
    gain.push(target_piece.value() as i16);

    while let Some((from_bb, from_piece)) = capturing {
        gain.push(from_piece.value() as i16 - gain.last().unwrap());
        alread_seen_attackers |= from_bb;

        playing_color = !playing_color;
        occupancy &= !from_bb;
        both_color_attackers &= !from_bb;
        if may_cause_xray & from_bb != EMPTY {
            both_color_attackers |= board.square_xray_update(to, occupancy) & !alread_seen_attackers;
        }

        capturing = least_valuable_attacker(board, both_color_attackers, playing_color);
    }
    
    for i in (1..gain.len()-1).rev() {
        gain[i-1] = -max(-gain[i-1], gain[i]);
    }

    *gain.first().unwrap()
}

fn least_valuable_attacker(board: &Board, both_color_attacker: Bitboard, attacking_color: Color) -> Option<(Bitboard, Piece)> {
    let attackers = both_color_attacker & board.pieces[attacking_color];
    for piece in [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING] {
        let piece_atteckers = attackers & board.bitboards[piece];
        if piece_atteckers != EMPTY {
            return Some((piece_atteckers.lsb().as_bitboard(), piece));
        }
    }
    None
}

fn move_sorting_key(board: &Board, m: Move, last_moved_piece: Square) -> (u8, i16) {
    let see = static_exchange_evaluation(board, m.from(), m.to());
    let capture = matches!(m.infos(), MoveInfo::Capture | MoveInfo::CapturePromotion(_));
    let from_piece = board.squares[m.from() as usize].unwrap();


    // recapture
    if m.to() == last_moved_piece {
        (1, from_piece.value() as i16)
    }
    // SEE > 0
    else if see > 0 {
        (2, -see)
    }
    // SEE = 0
    else if capture && see == 0 {
        (3, from_piece.value() as i16)
    }
    // SEE < 0
    else if see < 0 {
        (4, -see)
    }
    // Quiet
    else {
        (5,  from_piece.value() as i16)
    }
}

pub fn order_moves(board: &Board, moves: &mut ArrayVec<Move, MAX_MOVE_NUMBER>, last_moved_piece: Square) {
    moves.sort_by_cached_key(|&m| move_sorting_key(board, m, last_moved_piece));
}


#[cfg(test)]
mod tests {
    use crate::board::*;
    use super::*;

    #[test]
    fn test_see() {
        let board = Board::from_fen("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - -").expect("Invalid fen");
        assert_eq!(static_exchange_evaluation(&board, E1, E5), 100);

        let board = Board::from_fen("1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - -").expect("Invalid fen");
        assert_eq!(static_exchange_evaluation(&board, D3, E5), -225);
    }

}