use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, UciNewGame};
use std::error::Error;
use std::fmt::{Display, Formatter};

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

#[derive(Debug)]
pub struct InvalidUciCommand {
    command: String,
}

impl InvalidUciCommand {
    pub fn new(command: &str) -> InvalidUciCommand {
        InvalidUciCommand {
            command: command.into(),
        }
    }
}

impl Display for InvalidUciCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid Uci Command: {}", self.command)
    }
}

impl Error for InvalidUciCommand {}

pub enum UciCommand {
    UciNewGame,
    UciInfo,
    IsReady,
    Position(PositionCommandType),
    Go(String),
    Quit,
}

impl UciCommand {
    pub fn new(line: &str) -> Result<UciCommand, InvalidUciCommand> {
        let (command, args) = line.split_at(line.find(' ').unwrap_or(line.len()));

        match command.trim() {
            "ucinewgame" => Ok(UciNewGame),
            "uci" => Ok(UciInfo),
            "isready" => Ok(IsReady),
            "position" => Ok(Position(
                PositionCommandType::new_from_pos_args(args)
                    .map_err(|_| InvalidUciCommand::new(line))?,
            )),
            "go" => Ok(Go(args.to_string())),
            "quit" => Ok(Quit),
            _ => Err(InvalidUciCommand::new(line)),
        }
    }
}
