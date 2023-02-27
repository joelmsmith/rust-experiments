pub mod attack;
pub mod bitboard;
pub mod moves;
pub mod position;
pub mod square;
pub mod things;

use crate::moves::*;
use crate::square::*;
use position::Position;
use std::time::Instant;

fn perft(depth: u32, pos: &mut Position) -> usize {
    let mut move_generator = MoveGen::new(*pos);
    move_generator.gen_legal_moves();

    if depth == 1 {
        return move_generator.moves.len();
    }

    let mut nodes: usize = 0;
    for mv in move_generator.moves {
        let undo = pos.make_move(mv);
        nodes += perft(depth - 1, pos);
        pos.unmake_move(undo);
    }
    nodes
}

fn divide(depth: u32, pos: &mut Position) -> usize {
    let mut total_nodes: usize = 0;
    let mut move_generator = MoveGen::new(*pos);
    move_generator.gen_legal_moves();

    println!("node count, depth=={}", depth);
    let start = Instant::now();
    if depth == 1 {
        total_nodes = move_generator.moves.len();
    } else if depth > 1 {
        for mv in move_generator.moves {
            let src = mv_get_src(mv);
            let dst = mv_get_dst(mv);

            let undo = pos.make_move(mv);
            let nodes = perft(depth - 1, pos);
            total_nodes += nodes;
            pos.unmake_move(undo);

            println!("{}{}:  {}", sq_to_str(src), sq_to_str(dst), nodes);
        }
    }
    let usec = 1 + start.elapsed().as_micros();
    let sec = usec as f64 / 1000000.;
    let milli = usec as f64 / 1000.;
    let knps = (total_nodes as f64 / sec as f64) / 1000.;

    println!(
        "{} nodes in {} ms; {} knps\n",
        total_nodes, milli, knps as u64
    );
    total_nodes
}

fn main() {
    attack::init();
    bitboard::init();

    let position1: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let position2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let position3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let position4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    let position5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let position6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

    let mut pos = Position::new();
    pos.from_fen(position1);
    pos.debug();
    assert!(119060324 == divide(6, &mut pos));

    pos.from_fen(position2);
    pos.debug();
    assert!(193690690 == divide(5, &mut pos));

    pos.from_fen(position3);
    pos.debug();
    assert!(178633661 == divide(7, &mut pos));

    pos.from_fen(position4);
    pos.debug();
    assert!(15833292 == divide(5, &mut pos));

    pos.from_fen(position5);
    pos.debug();
    assert!(89941194 == divide(5, &mut pos));

    pos.from_fen(position6);
    pos.debug();
    assert!(6923051137 == divide(6, &mut pos));
}
