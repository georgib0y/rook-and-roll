use std::collections::HashMap;
use simplelog::*;

use crate::eval::MAT_SCORES;
use crate::{Search, Board, Move, SeqTT, MoveTables};
use log::{error, info};
use std::io;
use std::io::{BufWriter, Stdout, Write};
use crate::moves::{KillerMoves, PrevMoves};

// TODO maybe uci is a bit of a misleading name for the struct now that it contains more info
pub struct Uci {
    author: String,
    bot_name: String,
    tt: SeqTT,
    mt: MoveTables,
    km: KillerMoves,
    board: Board,
    prev_moves: PrevMoves,
}

impl Uci {
    pub fn new(author: &str, bot_name: &str) -> Uci {
        Uci {
            author: String::from(author),
            bot_name: String::from(bot_name),
            tt: SeqTT::new(),
            km: KillerMoves::new(),
            mt: MoveTables::new(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
        }
    }

    pub fn start(&mut self) {
        let mut buffer = String::new();

        loop {
            io::stdin()
                .read_line(&mut buffer)
                .expect("Uci input failed");

            info!(target: "input", "{buffer}");

            if buffer.starts_with("ucinewgame") {
                self.tt = SeqTT::new();
                self.board = Board::new();
            } else if buffer.starts_with("uci") {
                let out = format!(
                    "id name {}\nid author {}\nuciok",
                    self.bot_name, self.author
                );
                info!(target: "output", "{out}");
                println!("{}", out);
            } else if buffer.starts_with("isready") {
                info!(target: "output", "readyok");
                println!("readyok");
            } else if buffer.starts_with("position") {
                self.position(&buffer);
            } else if buffer.starts_with("go") {
                self.go();
            } else if buffer.starts_with("quit") {
                break;
            } else if buffer.starts_with("print board") || buffer.starts_with("pb") {
                println!("{}", self.board);
            }

            buffer.clear();
        }
    }

    fn position(&mut self, buffer: &str) {
        self.board = if buffer.contains("fen") {
            // split after postition fen ...
            let mut fen = buffer.split_once("fen ").unwrap().1;
            // if there are extra moves afterwards (fen ... moves e3g8 f4f4 ... split before then
            if fen.contains("moves") {
                fen = fen.split_once(" moves").unwrap().0;
            }
            // println!("{fen}");
            Board::new_fen(fen)
        } else {
            // if "startpos"
            Board::new()
        };

        if buffer.contains("moves") {
            let moves: Vec<&str> = buffer
                .trim()
                .split_once("moves ")
                .unwrap()
                .1
                .split(' ')
                .collect();

            for m in moves {
                let mv = Move::new_from_text(m, &self.board);
                self.board = self.board.copy_make(mv);
            }
        }
    }

    fn go(&mut self) {
        info!(target: "output", "info string starting search");
        println!("info string starting search");

        let best_move = Search::new(&self.mt, &mut self.tt, &mut self.km, &mut self.prev_moves)
            .iterative_deepening(&self.board);

        if best_move.is_none() {
            error!(target: "panic", "Did not find a best move")
        }

        let out = format!("bestmove {}", best_move.unwrap().as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");

    }
}
