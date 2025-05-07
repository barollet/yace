use crate::consts::*;
use crate::*;

impl Board {
    pub fn from_fen(fen_string: &str) -> Option<Self> {
        let mut parts = fen_string.split_ascii_whitespace();
        let pieces = parts.next().unwrap();
        let to_move = parts.next().unwrap();
        let castling = parts.next().unwrap();
        let en_passant = parts.next().unwrap();
        //let _halfmove: &str = parts.next().unwrap();
        //let _fullmove = parts.next().unwrap();

        let mut board = Board::empty();

        for (rank, line) in (0..8).rev().zip(pieces.split('/')) {
            let mut file = 0;
            for char in line.chars() {
                if let Some(n) = char.to_digit(10) {
                    file += n as i8;
                } else {
                    let color = if char.is_uppercase() {WHITE} else {BLACK};
                    let piece = match char.to_ascii_uppercase() {
                        'P' => PAWN,
                        'N' => KNIGHT,
                        'B' => BISHOP,
                        'R' => ROOK,
                        'Q' => QUEEN,
                        'K' => KING,
                        _ => panic!("Invalid piece char")
                    };
                    board.add_piece(piece, Square::new(file, rank), color);
                    file += 1;
                }
            }
        }

        if to_move == "b" {
            board.to_move = BLACK;
        }

        let mut castling_rights = CastlingRights::new();
        if !castling.contains('K') {
            castling_rights.remove(WHITE, KINGSIDE);
        }
        if !castling.contains('Q') {
            castling_rights.remove(WHITE, QUEENSIDE);
        }
        if !castling.contains('k') {
            castling_rights.remove(BLACK, KINGSIDE);
        }
        if !castling.contains('q') {
            castling_rights.remove(BLACK, QUEENSIDE);
        }
        board.castling_rights = castling_rights;

        if en_passant != "-" {
            let mut chars = en_passant.chars();
            let file = chars.next().unwrap();
            let rank = chars.next().unwrap();
            let square = square_from_name(file as i8 - 'a' as i8, rank.to_digit(10).unwrap() as Square);
            board.ep_target = Some(square);
        }

        Some(board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fen() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

        assert_eq!(board.ep_target, Some(E3));

        let board = Board::from_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2 ").unwrap();
        assert_eq!(board.ep_target, None);
    }
}