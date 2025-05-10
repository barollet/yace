use crate::board::*;

// Evaluation is relative to side to move

// Piece square table are shown from black perspective (because it starts with A1 and they are symmetric)
static PAWN_SQUARE_TABLE: [i16; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10,-20,-20, 10, 10,  5,
    5, -5,-10,  0,  0,-10, -5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
    0,  0,  0,  0,  0,  0,  0,  0
];

static KNIGHT_SQUARE_TABLE: [i16; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

static BISHOP_SQUARE_TABLE: [i16; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

static ROOK_SQUARE_TABLE: [i16; 64] = [
    0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    5, 10, 10, 10, 10, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0,
];

static QUEEN_SQUARE_TABLE: [i16; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

static KING_SQUARE_TABLE_MIDDLE: [i16; 64] = [
    20, 30, 10,  0,  0, 10, 30, 20,
    20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

static KING_SQUARE_TABLE_END: [i16; 64] = [
    -50,-30,-30,-30,-30,-30,-30,-50,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -50,-40,-30,-20,-20,-30,-40,-50,
];

// This is from white point of view
#[derive(Debug, Clone)]
pub struct IncrementalEval {
    material_evaluation: i16,
    position_evaluation: i16,
}

fn piece_positional_value(piece: Piece, mut sq: Square, color: Color) -> i16 {
    if color == BLACK {
        sq = sq.vertical_symmetry();
    }
    match piece {
        Piece::Pawn => PAWN_SQUARE_TABLE[sq as usize],
        Piece::Knight => KNIGHT_SQUARE_TABLE[sq as usize],
        Piece::Bishop => BISHOP_SQUARE_TABLE[sq as usize],
        Piece::Rook => ROOK_SQUARE_TABLE[sq as usize],
        Piece::Queen => QUEEN_SQUARE_TABLE[sq as usize],
        Piece::King => KING_SQUARE_TABLE_MIDDLE[sq as usize], // TODO handle endgame
    }
}

impl IncrementalEval {
    pub fn new() -> Self {
        IncrementalEval { material_evaluation: 0, position_evaluation: 0 }
    }

    pub fn add_piece(&mut self, piece: Piece, sq: Square, color: Color) {
        if color == WHITE {
            self.material_evaluation += piece.value() as i16;
            self.position_evaluation += piece_positional_value(piece, sq, color);
        } else {
            self.material_evaluation -= piece.value() as i16;
            self.position_evaluation -= piece_positional_value(piece, sq, color);
        }
    }

    pub fn remove_piece(&mut self, piece: Piece, sq: Square, color: Color) {
        if color == WHITE {
            self.material_evaluation -= piece.value() as i16;
            self.position_evaluation -= piece_positional_value(piece, sq, color);
        } else {
            self.material_evaluation += piece.value() as i16;
            self.position_evaluation += piece_positional_value(piece, sq, color);
        }
    }

    pub fn score(&self, pov: Color) -> i16 {
        let white_score = self.material_evaluation + self.position_evaluation;
        if pov == WHITE {
            white_score
        } else {
            -white_score
        }
    }
}