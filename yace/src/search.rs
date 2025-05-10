use std::i16;

use crate::{board::*, move_ordering::order_moves};

struct PVNode {
    pv_move: Move,
    evaluation: i16,
}

pub struct Searcher<'a> {
    principal_variation: Vec<PVNode>,
    board: &'a mut Board,
}

impl<'a> Searcher<'a> {
    pub fn new(board: &'a mut Board) -> Self {
        Searcher { principal_variation: Vec::with_capacity(32), board, }
    }

    pub fn search(&mut self, depth: u8) -> i16 {
        self.alphabeta(i16::MIN, i16::MAX, depth)
    }

    fn alphabeta(&mut self, mut alpha: i16, beta: i16, depthleft: u8) -> i16 {
        if depthleft == 0 {
            return self.board.evaluation.score(self.board.to_move);
        }

        let mut possible_moves = self.board.legal_move_gen();
        order_moves(self.board, &mut possible_moves, A1);
        let mut max_score = i16::MIN;

        for possible_move in possible_moves {

            // Make -> recursive eval -> unmake
            let ext_move = self.board.make(possible_move);
            let score = -self.alphabeta(-beta, -alpha, depthleft-1);
            self.board.unmake(ext_move);

            // update alpha and best max score
            if score > max_score {
                max_score = alpha;
                if score > alpha {
                    alpha = score;
                }
            }

            // beta cutoff
            if score >= beta {
                return score
            }

        }

        max_score
    }

}

impl PVNode {
    fn new(pv_move: Move, evaluation: i16) -> Self {
        Self { pv_move, evaluation }
    }
}