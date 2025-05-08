use board::Board;

mod move_ordering;
mod search;

fn main() {
    benchmark_perft();
}

fn benchmark_perft() {
    let mut board = Board::new();
    board.perft::<false>(6);

    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").expect("Invalid fen");
    board.perft::<false>(7);

    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ").expect("Invalid fen");
    board.perft::<false>(5);

    let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").expect("Invalid fen");
    board.perft::<false>(6);

    let mut board = Board::from_fen("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1").expect("Invalid fen");
    board.perft::<false>(6);

    let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").expect("Invalid fen");
    board.perft::<false>(5);

    let mut board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").expect("Invalid fen");
    board.perft::<false>(5);
}
