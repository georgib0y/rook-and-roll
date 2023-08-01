use crate::fen::InvalidFenError;
use crate::lazy_smp::lazy_smp;
use crate::movegen::MoveList;
use crate::moves::{PrevMoves, NULL_MOVE};
use crate::search::iterative_deepening;
use crate::tt::{AtomicTTable, TTable, TT};
use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, UciNewGame};
use crate::{Board, Move};
use log::info;
use std::io;
use std::sync::Arc;

pub enum PositionCommandType {
    Startpos,
    Fen { fen: String, moves: Vec<String> },
}

impl PositionCommandType {
    pub fn new_from_pos_args(position_args: &str) -> Result<PositionCommandType, ()> {
        let split = position_args.trim().split(' ').collect::<Vec<_>>();

        match split.first() {
            Some(&"startpos") => Ok(PositionCommandType::Startpos),
            Some(&"fen") => {
                let fen = split.get(1).map(|s| s.to_string()).ok_or(())?;

                let moves = split.into_iter().skip(3).map(|s| s.into()).collect();

                Ok(PositionCommandType::Fen { fen, moves })
            }
            Some(_) | None => Err(()),
        }
    }
}

enum UciCommand {
    UciNewGame,
    UciInfo,
    IsReady,
    Position(PositionCommandType),
    Go(String),
    Quit,
}

impl UciCommand {
    fn new(line: &str) -> Result<UciCommand, ()> {
        let (command, args) = line.split_at(line.find(' ').unwrap_or(line.len()));

        match command.trim() {
            "ucinewgame" => Ok(UciNewGame),
            "uci" => Ok(UciInfo),
            "isready" => Ok(IsReady),
            "position" => Ok(Position(PositionCommandType::new_from_pos_args(args)?)),
            "go" => Ok(Go(args.to_string())),
            "quit" => Ok(Quit),
            _ => Err(()),
        }
    }
}

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

            let Ok(command) = UciCommand::new(&buffer) else {
                println!("Unknown command: {}", buffer);
                continue;
            };

            match command {
                UciNewGame => self.ucinewgame(),
                UciInfo => self.uci_info(),
                IsReady => self.is_ready(),
                Position(pos) => {
                    if let Err(err) = self.position(pos) {
                        println!("{}", err)
                    }
                }
                Go(args) => self.go(&args),
                Quit => break,
            }
        }
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

    pub fn position(&mut self, position_type: PositionCommandType) -> Result<(), InvalidFenError> {
        let mut prev_moves = PrevMoves::new();

        let mut board = match position_type {
            PositionCommandType::Startpos => Board::new(),
            PositionCommandType::Fen { fen, moves } => {
                let mut board = Board::new_fen(&fen)?;
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

        Ok(())
    }

    pub fn go(&mut self, _args: &str) {
        let best_move = self.find_best_move().unwrap_or(NULL_MOVE);
        let out = format!("bestmove {}", best_move.as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");
    }

    pub fn ucinewgame(&mut self) {
        self.tt.clear();
        self.board = Some(Board::new());
        self.prev_moves = Some(PrevMoves::new());
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
