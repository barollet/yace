use std::sync::LazyLock;

use arrayvec::ArrayVec;
use bit_iter::BitIter;

use super::{magic_table::bishop_attack, magic_table::rook_attack, *};

static KNIGHT_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_knight_attack);
static KING_ATTACK: LazyLock<[Bitboard; 64]> = LazyLock::new(initialize_king_attack);
static EP_FROM_SQUARES: LazyLock<[Bitboard; 16]> = LazyLock::new(initialize_ep_from_squares);
static EMPTY_CASTLING_SQUARES: LazyLock<[Bitboard; 4]> = LazyLock::new(initialize_castle_empty_squares);
static CHECK_CASTLE_SQUARES: LazyLock<[Bitboard; 4]> = LazyLock::new(initialize_castle_check_squares);
static KING_CASTLING_DEST: LazyLock<[Square; 4]> = LazyLock::new(initialize_king_castle_dest);
pub static ROOK_CASTLING_START: LazyLock<[Square; 4]> = LazyLock::new(initialize_rook_castle_start);
pub static ROOK_CASTLING_DEST: LazyLock<[Square; 4]> = LazyLock::new(initialize_rook_castle_dest);

const QUIET: u8 = 0;
const CAPTURE: u8 = 1;
const EVASION: u8 = 2;
const NON_EVASION: u8 = 3;

pub const MAX_MOVE_NUMBER: usize = 256;

impl Board {
    pub fn legal_move_gen(&self) -> ArrayVec<Move, MAX_MOVE_NUMBER> {
        if self.to_move == WHITE {
            MoveGenerator::new(self).generate::<WHITE>()
        } else {
            MoveGenerator::new(self).generate::<BLACK>()
        }
    }

    fn bishop_attack(&self, sq: Square) -> Bitboard {
        bishop_attack(sq, self.occupancy())
    }

    fn rook_attack(&self, sq: Square) -> Bitboard {
        rook_attack(sq, self.occupancy())
    }

    pub fn square_attacked_by<const MY_COLOR: bool>(&self, sq: Square) -> Bitboard {
        self.square_attacked_by_with_occ::<MY_COLOR>(sq, self.occupancy())
    }

    pub fn square_attacked_by_with_occ<const MY_COLOR: bool>(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop_attack(sq, occupancy) & self.pieces[!MY_COLOR] & (self.bitboards[BISHOP] | self.bitboards[QUEEN])
        | rook_attack(sq, occupancy) & self.pieces[!MY_COLOR] & (self.bitboards[ROOK] | self.bitboards[QUEEN])
        | KNIGHT_ATTACK[sq as usize] & self.pieces[!MY_COLOR] & self.bitboards[KNIGHT]
        | KING_ATTACK[sq as usize] & self.pieces[!MY_COLOR] & self.bitboards[KING]
        | self.pieces[!MY_COLOR] & self.bitboards[PAWN] & sq.forward_left::<MY_COLOR>().map_or(EMPTY, Square::as_bitboard)
        | self.pieces[!MY_COLOR] & self.bitboards[PAWN] & sq.forward_right::<MY_COLOR>().map_or(EMPTY, Square::as_bitboard)
    }

    pub fn square_full_attacked_by(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop_attack(sq, occupancy) & (self.bitboards[BISHOP] | self.bitboards[QUEEN])
        | rook_attack(sq, occupancy) & (self.bitboards[ROOK] | self.bitboards[QUEEN])
        | KNIGHT_ATTACK[sq as usize] & self.bitboards[KNIGHT]
        | KING_ATTACK[sq as usize] & self.bitboards[KING]
        |    (sq.forward_left::<WHITE>().map_or(EMPTY, Square::as_bitboard)
            | sq.forward_right::<WHITE>().map_or(EMPTY, Square::as_bitboard)
            | sq.forward_left::<BLACK>().map_or(EMPTY, Square::as_bitboard)
            | sq.forward_right::<BLACK>().map_or(EMPTY, Square::as_bitboard)) & self.bitboards[PAWN]
    }

    pub fn square_xray_update(&self, sq: Square, occupancy: Bitboard) -> Bitboard {
        bishop_attack(sq, occupancy) & (self.bitboards[BISHOP] | self.bitboards[QUEEN])
        | rook_attack(sq, occupancy) & (self.bitboards[ROOK] | self.bitboards[QUEEN])
    }

    fn pinned_pieces(&self) -> Bitboard {
        let king_square = self.king_square(self.to_move);
        let enemy_pieces = self.pieces[!self.to_move];

        let mut pinned = EMPTY;

        let snipers = rook_attack(king_square, EMPTY) & (self.bitboards[ROOK] | self.bitboards[QUEEN]) & enemy_pieces
            | bishop_attack(king_square, EMPTY) & (self.bitboards[BISHOP] | self.bitboards[QUEEN]) & enemy_pieces;

        for start_square in BitIter::from(snipers) {
            let line = Bitboard::between(start_square as Square, king_square).unset(start_square as Square) & self.occupancy();

            if line.count_ones() == 1 {
                pinned |= line;
            }
        }
        pinned
    }

    pub fn perft<const IS_ROOT: bool>(&mut self, depth: usize) -> usize {
        if depth == 1 {
            let moves = self.legal_move_gen();
            if IS_ROOT {
                for m in &moves {
                    println!("{}{}: {}", m.from().debug(), m.to().debug(), 1);
                }
            }
            moves.len()
        } else {
            let hash = self.zobrist_hash;
            let mut count = 0;
            for to_play in self.legal_move_gen() {
                let ext_move = self.make(to_play);
                let local_count = self.perft::<false>(depth-1);
                if IS_ROOT {
                    println!("{}{}: {}", to_play.from().debug(), to_play.to().debug(), local_count);
                }
                count += local_count;
                self.unmake(ext_move);
            }
            assert_eq!(hash, self.zobrist_hash); // TODO remove this assertion at some point
            count
        }
    }
}

struct MoveGenerator<'a> {
    board: &'a Board,
    moves: ArrayVec<Move, MAX_MOVE_NUMBER>,
}

impl<'a> MoveGenerator<'a> {
    fn new(board: &'a Board) -> Self {
        Self {
            board,
            moves: ArrayVec::new(),
        }
    }

    fn generate<const COLOR: bool>(mut self) -> ArrayVec<Move, MAX_MOVE_NUMBER> {
        let king_square = self.board.king_square(self.board.to_move);
        if self.board.square_attacked_by::<COLOR>(king_square) != 0 {
            self.pseudo_legal_movegen::<COLOR, EVASION>();
        } else {
            self.pseudo_legal_movegen::<COLOR, NON_EVASION>();
        }

        let pinned = self.board.pinned_pieces();
        self.moves.retain(|m| {
            if m.infos() == MoveInfo::EnPassantCapture {
                let ep_target = self.board.ep_target.unwrap();
                let occupancy = (self.board.pieces[WHITE] | self.board.pieces[BLACK]).unset(m.from()).unset(ep_target).set(m.to());
                // if the attack is made by the en passant target, it doesn't count
                if self.board.square_attacked_by_with_occ::<COLOR>(king_square, occupancy) != EMPTY 
                && king_square.forward_left::<COLOR>() != self.board.ep_target && king_square.forward_right::<COLOR>() != self.board.ep_target {
                    return false
                }
            }

            if pinned.has(m.from()) {
                // if the piece is pinned, the king must be on the line of its movement
                // if the movement is not a line then the bitboard is empty
                return Bitboard::line(m.from(), m.to()).has(king_square)
            }

            if m.infos() == MoveInfo::KingCastle {
                for sq in BitIter::from(CHECK_CASTLE_SQUARES[CastlingRights::index(COLOR, KINGSIDE)]) {
                    if self.board.square_attacked_by::<COLOR>(sq as Square) != EMPTY {
                        return false
                    }
                }
            } else if m.infos() == MoveInfo::QueenCastle {
                for sq in BitIter::from(CHECK_CASTLE_SQUARES[CastlingRights::index(COLOR, QUEENSIDE)]) {
                    if self.board.square_attacked_by::<COLOR>(sq as Square) != EMPTY {
                        return false
                    }
                }
            }

            if let Some(KING) = self.board.squares[m.from() as usize] {
                let occupancy = (self.board.pieces[WHITE] | self.board.pieces[BLACK]).unset(king_square);
                return self.board.square_attacked_by_with_occ::<COLOR>(m.to(), occupancy) == EMPTY
            }

            true
        });

        self.moves
    }

    fn pseudo_legal_movegen<const COLOR: bool, const KIND: u8>(&mut self) {
        let target = if KIND == QUIET {
            !(self.board.pieces[WHITE] | self.board.pieces[BLACK])
        } else if KIND == CAPTURE {
            self.board.pieces[!COLOR]
        } else if KIND == NON_EVASION {
            !self.board.pieces[COLOR]
        } else { // EVASION
            let checker = self.board.checkers::<COLOR>().lsb();
            Bitboard::between(checker, self.board.king_square(COLOR))
        };

        // If this is not double check we can generate piece moves
        if !(KIND == EVASION && self.board.checkers::<COLOR>().count_ones() > 1) {
            self.generate_pawn_moves::<COLOR, KIND>(target);

            for sq in BitIter::from(self.board.pieces[COLOR]) {
                let piece: Piece = self.board.squares[sq].unwrap();
                if piece == KNIGHT {
                    self.generate_piece_moves::<KNIGHT_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == BISHOP {
                    self.generate_piece_moves::<BISHOP_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == ROOK {
                    self.generate_piece_moves::<ROOK_ORDINAL, COLOR>(sq as Square, target);
                } else if piece == QUEEN {
                    self.generate_piece_moves::<QUEEN_ORDINAL, COLOR>(sq as Square, target);
                }
            }
        }

        let target = if KIND != EVASION {target} else {!self.board.pieces[COLOR]};
        self.generate_piece_moves::<KING_ORDINAL, COLOR>(self.board.king_square(self.board.to_move), target);

        // Castling
        if KIND == NON_EVASION || KIND == QUIET {
            let all_pieces = self.board.pieces[WHITE] | self.board.pieces[BLACK];
            let king_square = self.board.king_square(self.board.to_move);
            let castle_index = CastlingRights::index(COLOR, KINGSIDE);
            if self.board.castling_rights.has(COLOR, KINGSIDE) && EMPTY_CASTLING_SQUARES[castle_index] & all_pieces == EMPTY {
                self.moves.push(Move::new_base(king_square, KING_CASTLING_DEST[castle_index]).with_infos(MoveInfo::KingCastle));
            }
            let castle_index = CastlingRights::index(COLOR, QUEENSIDE);
            if self.board.castling_rights.has(COLOR, QUEENSIDE) && EMPTY_CASTLING_SQUARES[CastlingRights::index(COLOR, QUEENSIDE)] & all_pieces == EMPTY {
                self.moves.push(Move::new_base(king_square, KING_CASTLING_DEST[castle_index]).with_infos(MoveInfo::QueenCastle));
            }
        }
    }

    // Shouldn't be used on pawns
    fn generate_piece_moves<const PIECE: u8, const COLOR: bool>(&mut self, sq: Square, target: Bitboard) {
        let piece_moves = self.move_list::<PIECE, COLOR>(sq);
        let empty = !(self.board.pieces[COLOR] | self.board.pieces[!COLOR]);

        for dest_square in BitIter::from(piece_moves & target & self.board.pieces[!COLOR]) {
            self.moves.push(Move::new_base(sq, dest_square as Square).with_infos(MoveInfo::Capture));
        }
    
        for dest_sq in BitIter::from(piece_moves & target & empty) {
            self.moves.push(Move::new_base(sq, dest_sq as Square));
        }
        
    }

    // Shouldn't be used on pawns
    fn move_list<const PIECE: u8, const COLOR: bool>(&self, sq: Square) -> Bitboard {
        match PIECE {
            KNIGHT_ORDINAL => KNIGHT_ATTACK[sq as usize],
            BISHOP_ORDINAL => self.board.bishop_attack(sq),
            ROOK_ORDINAL => self.board.rook_attack(sq),
            QUEEN_ORDINAL => self.board.bishop_attack(sq) | self.board.rook_attack(sq),
            KING_ORDINAL => KING_ATTACK[sq as usize],

            _ => panic!("Invalid piece for capture list"),
        }
    }

    fn generate_pawn_moves<const COLOR: bool, const KIND: u8>(&mut self, target: Bitboard) {
        let empty = !(self.board.pieces[COLOR] | self.board.pieces[!COLOR]);
        let pawns = self.board.bitboards[PAWN] & self.board.pieces[COLOR];

        let promotion_rank = if COLOR == WHITE {RANK8} else {RANK1};
        
        // Captures
        if KIND != QUIET {
            // Simple capture + promotion capture
            for dest_square in BitIter::from(pawns.forward_left::<COLOR>() & self.board.pieces[!COLOR] & target) {
                let dest_square = dest_square as Square;
                let from_square = dest_square.backward_left::<COLOR>().unwrap();
                if promotion_rank.has(dest_square) {
                    self.make_promotion::<KIND, true>(from_square, dest_square);
                } else {
                    self.moves.push(Move::new_base(from_square, dest_square as Square).with_infos(MoveInfo::Capture));
                }
            }
            
            for dest_square in BitIter::from(pawns.forward_right::<COLOR>() & self.board.pieces[!COLOR] & target) {
                let dest_square = dest_square as Square;
                let from_square = dest_square.backward_right::<COLOR>().unwrap();
                if promotion_rank.has(dest_square) {
                    self.make_promotion::<KIND, true>(from_square, dest_square);
                } else {
                    self.moves.push(Move::new_base(from_square, dest_square as Square).with_infos(MoveInfo::Capture));
                }
            }
            
            // En passant
            if let Some(ep_target) = self.board.ep_target {
                let ep_dest = ep_target.forward::<COLOR>();
                if self.board.squares[ep_dest as usize].is_none() && (target.has(ep_dest) | target.has(ep_target)) {
                    let capturing_pawns = EP_FROM_SQUARES[(ep_target - A4) as usize] & self.board.bitboards[PAWN] & self.board.pieces[COLOR];
                    for from_square in BitIter::from(capturing_pawns) {
                        self.moves.push(Move::new_base(from_square as Square, ep_dest).with_infos(MoveInfo::EnPassantCapture));
                    }
                }
            }
        }

        // Simple push and double push
        let base_rank = if COLOR == WHITE {RANK3} else {RANK6};
        let push_dest = pawns.forward::<COLOR>() & empty;
        let double_push_dest = (push_dest & base_rank).forward::<COLOR>() & empty & target;

        if KIND != CAPTURE {
            // simple
            for dest_square in BitIter::from(push_dest & target) {
                let dest_square = dest_square as Square;
                let from_square = dest_square.backward::<COLOR>();
                if promotion_rank.has(dest_square) {
                    self.make_promotion::<KIND, false>(from_square, dest_square);
                } else {
                    self.moves.push(Move::new_base(from_square, dest_square));
                }
            }
    
            // double
            for dest_square in BitIter::from(double_push_dest) {
                let dest_square = dest_square as Square;
                self.moves.push(Move::new_base(dest_square.backward::<COLOR>().backward::<COLOR>(), dest_square).with_infos(MoveInfo::DoublePawnPush));
            }
        }
    }
    
    fn make_promotion<const KIND: u8, const CAPTURING: bool>(&mut self, from: Square, to: Square) {
        if KIND != QUIET {
            if CAPTURING {
                self.moves.push(Move::new_base(from, to).with_infos(MoveInfo::CapturePromotion(QUEEN)));
            } else {
                self.moves.push(Move::new_base(from, to).with_infos(MoveInfo::Promotion(QUEEN)));
            }
        }

        if KIND != CAPTURE {
            for piece in [KNIGHT, BISHOP, ROOK] {
                if CAPTURING {
                    self.moves.push(Move::new_base(from, to).with_infos(MoveInfo::CapturePromotion(piece)));
                } else {
                    self.moves.push(Move::new_base(from, to).with_infos(MoveInfo::Promotion(piece)));
                }
            }
        }
    }
}

fn initialize_knight_attack() -> [Bitboard; 64] {
    let mut knight_attack = [0; 64];
    for square in 0..64 {
        let mut attack = EMPTY;
        let f = square.file();
        let r = square.rank();

        if f > 0 && r > 1 {
            attack = attack.set(Square::new(f-1, r-2));
        }
        if f > 1 && r > 0 {
            attack = attack.set(Square::new(f-2, r-1));
        }
        if f < 7 && r < 6 {
            attack = attack.set(Square::new(f+1, r+2));
        }
        if f < 6 && r < 7 {
            attack = attack.set(Square::new(f+2, r+1));
        }
        if f > 0 && r < 6 {
            attack = attack.set(Square::new(f-1, r+2));
        }
        if f < 7 && r > 1 {
            attack = attack.set(Square::new(f+1, r-2));
        }
        if f > 1 && r < 7 {
            attack = attack.set(Square::new(f-2, r+1));
        }
        if f < 6 && r > 0 {
            attack = attack.set(Square::new(f+2, r-1));
        }

        knight_attack[square as usize] = attack;
    }
    knight_attack
}

fn initialize_king_attack() -> [Bitboard; 64] {
    let mut king_attack = [0; 64];
    for square in 0..64 {
        let mut attack = EMPTY;
        let f = square.file();
        let r = square.rank();

        if f > 0 {
            attack = attack.set(Square::new(f-1, r));
            if r > 0 {
                attack = attack.set(Square::new(f-1, r-1));
            }
            if r < 7 {
                attack = attack.set(Square::new(f-1, r+1));
            }
        }
        if f < 7 {
            attack = attack.set(Square::new(f+1, r));
            if r > 0 {
                attack = attack.set(Square::new(f+1, r-1));
            }
            if r < 7 {
                attack = attack.set(Square::new(f+1, r+1));
            }
        }
        if r > 0 {
            attack = attack.set(Square::new(f, r-1));
        }
        if r < 7 {
            attack = attack.set(Square::new(f, r+1));
        }

        king_attack[square as usize] = attack;
    }
    king_attack
}

fn initialize_ep_from_squares() -> [Bitboard; 16] {
    let mut ep_from = [EMPTY; 16];
    for rank in [4, 5] {
        for file in FILE_LIST {
            let square = square_from_name(file, rank);
            let mut bb = EMPTY;
            if square.file() != A {
                bb = bb.set(square-1);
            }
            if square.file() != H {
                bb = bb.set(square+1);
            }
            ep_from[(square-A4) as usize] = bb;
        }
    }
    ep_from
}

fn initialize_castle_empty_squares() -> [Bitboard; 4] {
    let mut castle_empty = [EMPTY; 4];

    // white kingside
    castle_empty[CastlingRights::index(WHITE, KINGSIDE)] = EMPTY.set(F1).set(G1);
    // white queenside
    castle_empty[CastlingRights::index(WHITE, QUEENSIDE)] = EMPTY.set(B1).set(C1).set(D1);
    // black kingside
    castle_empty[CastlingRights::index(BLACK, KINGSIDE)] = EMPTY.set(F8).set(G8);
    // black queenside
    castle_empty[CastlingRights::index(BLACK, QUEENSIDE)] = EMPTY.set(B8).set(C8).set(D8);

    castle_empty
}

fn initialize_castle_check_squares() -> [Bitboard; 4] {
    let mut checks = [EMPTY; 4];

    // white kingside
    checks[CastlingRights::index(WHITE, KINGSIDE)] = EMPTY.set(F1).set(G1);
    // white queenside
    checks[CastlingRights::index(WHITE, QUEENSIDE)] = EMPTY.set(C1).set(D1);
    // black kingside
    checks[CastlingRights::index(BLACK, KINGSIDE)] = EMPTY.set(F8).set(G8);
    // black queenside
    checks[CastlingRights::index(BLACK, QUEENSIDE)] = EMPTY.set(C8).set(D8);

    checks
}

fn initialize_king_castle_dest() -> [Square; 4] {
    let mut king_dest = [A1; 4];

    // white kingside
    king_dest[CastlingRights::index(WHITE, KINGSIDE)] = G1;
    // white queenside
    king_dest[CastlingRights::index(WHITE, QUEENSIDE)] = C1;
    // black kingside
    king_dest[CastlingRights::index(BLACK, KINGSIDE)] = G8;
    // black queenside
    king_dest[CastlingRights::index(BLACK, QUEENSIDE)] = C8;

    king_dest
}

fn initialize_rook_castle_dest() -> [Square; 4] {
    let mut rook_dest = [A1; 4];

    // white kingside
    rook_dest[CastlingRights::index(WHITE, KINGSIDE)] = F1;
    // white queenside
    rook_dest[CastlingRights::index(WHITE, QUEENSIDE)] = D1;
    // black kingside
    rook_dest[CastlingRights::index(BLACK, KINGSIDE)] = F8;
    // black queenside
    rook_dest[CastlingRights::index(BLACK, QUEENSIDE)] = D8;

    rook_dest
}

fn initialize_rook_castle_start() -> [Square; 4] {
    let mut rook_start = [A1; 4];

    // white kingside
    rook_start[CastlingRights::index(WHITE, KINGSIDE)] = H1;
    // white queenside
    rook_start[CastlingRights::index(WHITE, QUEENSIDE)] = A1;
    // black kingside
    rook_start[CastlingRights::index(BLACK, KINGSIDE)] = H8;
    // black queenside
    rook_start[CastlingRights::index(BLACK, QUEENSIDE)] = A8;

    rook_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "perft"]
    fn perft_base() {
        // startpos
        let mut board = Board::new();
        assert_eq!(board.perft::<true>(1), 20);
        assert_eq!(board.perft::<true>(2), 400);
        assert_eq!(board.perft::<true>(3), 8_902);
        assert_eq!(board.perft::<true>(4), 197_281);
        assert_eq!(board.perft::<true>(5), 4_865_609);
        assert_eq!(board.perft::<true>(6), 119_060_324);

        // pos 2
        let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 14);
        assert_eq!(board.perft::<true>(2), 191);
        assert_eq!(board.perft::<true>(3), 2_812);
        assert_eq!(board.perft::<true>(4), 43_238);
        assert_eq!(board.perft::<true>(5), 674_624);
        assert_eq!(board.perft::<true>(6), 11_030_083);
        assert_eq!(board.perft::<true>(7), 178_633_661);

        // pos 3
        let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 48);
        assert_eq!(board.perft::<true>(2), 2039);
        assert_eq!(board.perft::<true>(3), 97_862);
        assert_eq!(board.perft::<true>(4), 4_085_603);
        assert_eq!(board.perft::<true>(5), 193_690_690);

        // pos 4 and mirrored
        let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 6);
        assert_eq!(board.perft::<true>(2), 264);
        assert_eq!(board.perft::<true>(3), 9_467);
        assert_eq!(board.perft::<true>(4), 422_333);
        assert_eq!(board.perft::<true>(5), 15_833_292);
        assert_eq!(board.perft::<true>(6), 706_045_033);

        let mut board = Board::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 6);
        assert_eq!(board.perft::<true>(2), 264);
        assert_eq!(board.perft::<true>(3), 9_467);
        assert_eq!(board.perft::<true>(4), 422_333);
        assert_eq!(board.perft::<true>(5), 15_833_292);
        assert_eq!(board.perft::<true>(6), 706_045_033);

        // pos 5
        let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 44);
        assert_eq!(board.perft::<true>(2), 1_486);
        assert_eq!(board.perft::<true>(3), 62_379);
        assert_eq!(board.perft::<true>(4), 2_103_487);
        assert_eq!(board.perft::<true>(5), 89_941_194);

        // pos 6
        let mut board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").expect("Invalid fen");
        assert_eq!(board.perft::<true>(1), 46);
        assert_eq!(board.perft::<true>(2), 2_079);
        assert_eq!(board.perft::<true>(3), 89_890);
        assert_eq!(board.perft::<true>(4), 3_894_594);
        assert_eq!(board.perft::<true>(5), 164_075_551);
    }
}