use crate::game_state::{BestMoveFinder, GameState};
use crate::uci::PositionCommandType;
use chess::board::board::PIECE_NAMES;
use chess::movegen::move_info::SQ_NAMES;
use chess::movegen::moves::Move;
use chess::search::old_tt::TTable;
use std::fs::read_to_string;

#[test]
fn wac_tests() {
    chess::init();
    let wacs = read_to_string("./wac").unwrap();
    let mut unmatched_ids = Vec::new();
    let num_tests = 20;
    let mut tested = 0;

    let mut game_state = GameState::<TTable>::new_single_thread("", "");

    for line in wacs.lines().take(num_tests) {
        if line.trim().is_empty() {
            break;
        }
        let (position, wac_id) = line.split_once("; ").unwrap();
        // id == WAC.001
        let id = String::from(wac_id.split("\"").skip(1).next().unwrap());

        let (fen, bm) = position.split_once(" bm ").unwrap();
        let pos = PositionCommandType::new_from_pos_args(fen).unwrap();

        game_state.ucinewgame();
        // game_state.position(pos).unwrap();

        let m = game_state.find_best_move().unwrap();
        tested += 1;

        if move_matches_bm(m, bm) {
            println!("{} matches {bm} on wac id {id}\n", m.as_uci_string());
        } else {
            println!("{} does not match {bm} on wac id {id}\n", m.as_uci_string());
            unmatched_ids.push(id);
        }
    }

    println!("failed {}/{tested} tests", unmatched_ids.len());
    dbg!(unmatched_ids);
}

fn move_matches_bm(m: Move, bm: &str) -> bool {
    let piece = PIECE_NAMES[m.piece() as usize].to_uppercase();
    let to = SQ_NAMES[m.to() as usize];

    bm.contains(&piece) && bm.contains(to)
}
