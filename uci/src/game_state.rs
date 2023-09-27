use crate::uci::UciCommand::*;
use crate::uci::UciCommand::{Go, Position};
use crate::uci::{PositionCommandType, UciCommand};
use chess::board::board::Board;
use chess::movegen::moves::{Move, PrevMoves, NULL_MOVE};
use chess::search::lazy_smp::lazy_smp;
use chess::search::searchers::iterative_deepening;
use chess::search::tt::{AtomicTTable, TTable, TT};
use log::info;
use std::io;
use std::sync::Arc;

pub struct GameState<T: TT> {
    author: String,
    bot_name: String,
    tt: T,
    board: Option<Board>,
    prev_moves: Option<PrevMoves>,
    num_threads: usize,
}

impl<T: TT> GameState<T>
where
    Self: BestMoveFinder,
{
    pub fn new_single_thread(author: &str, bot_name: &str) -> GameState<TTable> {
        GameState {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            tt: TTable::new(),
            board: Some(Board::new()),
            prev_moves: Some(PrevMoves::new()),
            num_threads: 1,
        }
    }

    pub fn new_multi_threaded(
        author: &str,
        bot_name: &str,
        num_threads: usize,
    ) -> GameState<Arc<AtomicTTable>> {
        GameState {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            tt: AtomicTTable::new(),
            board: Some(Board::new()),
            prev_moves: Some(PrevMoves::new()),
            num_threads,
        }
    }

    pub fn start(&mut self) {
        loop {
            let mut buffer = String::new();

            io::stdin()
                .read_line(&mut buffer)
                .expect("Uci input failed");
            info!(target: "input", "{buffer}");

            let command = match UciCommand::new(&buffer) {
                Ok(command) => command,
                Err(err) => {
                    println!("{err}");
                    continue;
                }
            };

            match command {
                UciNewGame => self.ucinewgame(),
                UciInfo => self.uci_info(),
                IsReady => self.is_ready(),
                Position(pos) => self.position(pos),
                Go(args) => self.go(&args),
                Quit => break,
            }
        }
    }

    pub fn ucinewgame(&mut self) {
        self.tt.clear();
        self.board = Some(Board::new());
        self.prev_moves = Some(PrevMoves::new());
    }

    fn uci_info(&self) {
        let out = format!(
            "id name {}\nid author {}\nuciok",
            self.bot_name, self.author
        );
        info!(target: "output", "{out}");
        println!("{out}");
    }

    fn is_ready(&self) {
        info!(target: "output", "readyok");
        println!("readyok");
    }

    pub fn position(&mut self, position_type: PositionCommandType) {
        let mut prev_moves = PrevMoves::new();

        let board = match position_type {
            PositionCommandType::Startpos => Board::new(),
            PositionCommandType::Fen { fen, moves } => {
                let mut board = match Board::new_fen(&fen) {
                    Ok(board) => board,
                    Err(err) => {
                        println!("{err}");
                        return;
                    }
                };

                for m in moves {
                    let mv = Move::new_from_text(&m, &board);
                    board = board.copy_make(mv);
                    prev_moves.add(board.hash);
                }

                board
            }
        };

        self.prev_moves = Some(prev_moves);
        self.board = Some(board);
    }

    pub fn go(&mut self, _args: &str) {
        let best_move = self.find_best_move().unwrap_or(NULL_MOVE);
        let out = format!("bestmove {}", best_move.as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");
    }
}

pub trait BestMoveFinder {
    fn find_best_move(&mut self) -> Option<Move>;
}

impl BestMoveFinder for GameState<TTable> {
    fn find_best_move(&mut self) -> Option<Move> {
        iterative_deepening(
            self.board.take().unwrap(),
            &self.tt,
            self.prev_moves.take().unwrap(),
        )
    }
}

impl BestMoveFinder for GameState<Arc<AtomicTTable>> {
    fn find_best_move(&mut self) -> Option<Move> {
        lazy_smp(
            self.board.take().unwrap(),
            Arc::clone(&self.tt),
            self.prev_moves.take().unwrap(),
            self.num_threads,
        )
    }
}
