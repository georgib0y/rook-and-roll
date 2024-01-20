use crate::board::Board;
use crate::game_state::{CanSearch, GameState};
use crate::moves::{Move, PrevMoves};
use crate::searcher::SearchError;
use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, UciNewGame};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::BufRead;

pub const AUTHOR: &str = "George";
pub const BOT_NAME: &str = "RookNRoll";

#[derive(Debug)]
pub enum InvalidUciCommand {
    UnknownCommand(String),
    InvalidPositionCommand(String),
}

impl Display for InvalidUciCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            InvalidUciCommand::UnknownCommand(cmd) => format!("Unknown UCI command: {cmd}"),
            InvalidUciCommand::InvalidPositionCommand(pos) => format!("Invalid position: {pos}"),
        };

        writeln!(f, "{}", s)
    }
}

impl Error for InvalidUciCommand {}

pub enum UciCommand {
    UciNewGame,
    UciInfo,
    IsReady,
    Position {
        fen: Option<String>,
        moves: Vec<String>,
    },
    Go(String),
    Quit,
}

impl UciCommand {
    pub fn new(line: &str) -> Result<UciCommand, InvalidUciCommand> {
        let (command, args) = line.split_at(line.find(' ').unwrap_or(line.len()));

        Ok(match command.trim() {
            "ucinewgame" => UciNewGame,
            "uci" => UciInfo,
            "isready" => IsReady,
            "position" => UciCommand::new_pos_command(args)?,
            "go" => Go(args.to_string()),
            "quit" => Quit,
            _ => Err(InvalidUciCommand::UnknownCommand(line.into()))?,
        })
    }

    fn new_pos_command(position_args: &str) -> Result<UciCommand, InvalidUciCommand> {
        let mut pos_args = position_args.trim().split("moves");
        let pos_str = pos_args
            .next()
            .ok_or(InvalidUciCommand::InvalidPositionCommand(
                position_args.into(),
            ))?;

        let moves_str = pos_args.next();

        dbg!(pos_str, moves_str);

        let fen = if pos_str.starts_with("startpos") {
            None
        } else if pos_str.starts_with("fen") {
            let (_, fen) =
                pos_str
                    .split_once(' ')
                    .ok_or(InvalidUciCommand::InvalidPositionCommand(
                        position_args.into(),
                    ))?;
            Some(fen.to_string())
        } else {
            Err(InvalidUciCommand::InvalidPositionCommand(
                position_args.into(),
            ))?
        };

        dbg!(&fen);

        let moves: Vec<String> = if let Some(moves_str) = moves_str {
            if moves_str.is_empty() {
                Vec::new()
            } else {
                moves_str
                    .trim()
                    .split(' ')
                    .map(|s| s.trim().into())
                    .collect()
            }
        } else {
            Vec::new()
        };

        dbg!(&moves);

        Ok(UciCommand::Position { fen, moves })
    }
}

pub trait Uci: CanSearch {
    fn start(&mut self) {
        loop {
            let command = match self.next_command() {
                Ok(command) => command,
                Err(err) => {
                    eprintln!("Unknown Command: {err}");
                    continue;
                }
            };

            if let Quit = command {
                eprintln!("Bye");
                return;
            }

            if let Some(out) = self.do_command(command) {
                println!("{}", out);
            }
        }
    }

    fn do_command(&mut self, command: UciCommand) -> Option<String> {
        match command {
            UciNewGame => {
                self.handle_new_game();
                None
            }
            UciInfo => Some(self.handle_uci_info()),
            IsReady => Some(self.handle_is_ready()),
            Position { fen, moves } => {
                self.handle_position_command(fen, moves);
                None
            }
            Go(_) => self.handle_search().ok(),
            _ => None,
        }
    }

    fn next_command(&mut self) -> Result<UciCommand, InvalidUciCommand> {
        let mut buffer = String::new();
        let mut stdin = io::stdin().lock();
        stdin.read_line(&mut buffer).expect("Uci input failed");
        UciCommand::new(&buffer)
    }

    fn handle_uci_info(&mut self) -> String {
        format!("id name {AUTHOR}\nid author {BOT_NAME}\nuciok")
    }

    fn handle_new_game(&mut self) {
        self.new_game();
    }

    fn handle_is_ready(&mut self) -> String {
        while !self.get_is_ready() {} // hang till ready
        "readyok".into()
    }

    fn get_is_ready(&mut self) -> bool;

    fn handle_position_command(&mut self, fen: Option<String>, moves: Vec<String>) {
        let mut board = Board::new();

        if let Some(fen) = fen {
            match Board::new_fen(&fen) {
                Ok(b) => board = b,
                Err(err) => {
                    eprintln!("Unkown fen: {fen}, {err}");
                    return;
                }
            };
        }

        let mut prev_moves = PrevMoves::new();

        for m_str in moves {
            let m = Move::new_from_text(&m_str, &board);
            board = board.copy_make(m);
            prev_moves.add(board.hash());
        }

        self.set_pos(board, prev_moves);
    }

    fn set_pos(&mut self, board: Board, prev_moves: PrevMoves);

    fn handle_search(&mut self) -> Result<String, SearchError> {
        self.go()
            .map(|(_, best_move)| format!("bestmove {}", best_move.as_uci_string()))
    }
}

impl<T> Uci for GameState<T>
where
    GameState<T>: CanSearch,
{
    fn get_is_ready(&mut self) -> bool {
        self.is_ready()
    }

    fn set_pos(&mut self, board: Board, prev_moves: PrevMoves) {
        self.set_position(board, prev_moves)
    }
}
