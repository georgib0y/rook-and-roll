use crate::game_state::UciGameState;
use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, UciNewGame};
use chess::board::board::Board;
use chess::movegen::moves::{Move, PrevMoves};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, LineWriter, Stdout, Write};
use std::{io, process};

const AUTHOR: &'static str = "George";
const BOT_NAME: &'static str = "Rustinator: Rook and Rool";

pub struct UciPositionCommand {
    fen: Option<String>,
    moves: Vec<String>,
}

impl UciPositionCommand {
    pub fn new_from_pos_args(position_args: &str) -> Result<UciPositionCommand, InvalidUciCommand> {
        let mut pos_args = position_args.trim().split("moves");
        let pos_str = pos_args.next().ok_or(InvalidUciCommand::new(&format!(
            "Invalid position: {position_args}"
        )))?;

        let moves_str = pos_args.next();

        dbg!(pos_str, moves_str);

        let fen = if pos_str.starts_with("startpos") {
            None
        } else if pos_str.starts_with("fen") {
            let (_, fen) = pos_str
                .split_once(' ')
                .ok_or(InvalidUciCommand::new(&format!(
                    "Invalid position: {position_args}"
                )))?;
            Some(fen.to_string())
        } else {
            Err(InvalidUciCommand::new(&format!(
                "Invalid position: {position_args}"
            )))?
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

        Ok(UciPositionCommand { fen, moves })
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
        writeln!(f, "Invalid Uci Command: {}", self.command)
    }
}

impl Error for InvalidUciCommand {}

pub enum UciCommand {
    UciNewGame,
    UciInfo,
    IsReady,
    Position(UciPositionCommand),
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
                UciPositionCommand::new_from_pos_args(args)
                    .map_err(|_| InvalidUciCommand::new(line))?,
            )),
            "go" => Ok(Go(args.to_string())),
            "quit" => Ok(Quit),
            _ => Err(InvalidUciCommand::new(line)),
        }
    }
}

pub struct Uci<T: UciGameState> {
    game_state: T,
    uci_writer_out: UciWriter,
}

impl<T: UciGameState> Uci<T> {
    pub fn new(game_state: T) -> Uci<T> {
        Uci {
            game_state,
            uci_writer_out: UciWriter::new(),
        }
    }

    pub fn new_with_out_file(game_state: T) -> Uci<T> {
        Uci {
            game_state,
            uci_writer_out: UciWriter::new_with_file(),
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = match self.next_command() {
                Ok(command) => command,
                Err(err) => {
                    writeln!(self.uci_writer_out, "Unknown Command: {err}").unwrap();
                    continue;
                }
            };

            match command {
                UciNewGame => self.game_state.new_game(),
                UciInfo => writeln!(
                    &mut self.uci_writer_out,
                    "id name {AUTHOR}\nid author {BOT_NAME}\nuciok"
                )
                .unwrap(),
                IsReady => self.game_state.is_ready(&mut self.uci_writer_out).unwrap(),
                Position(pos) => self.handle_position_command(pos),
                Go(_) => self.handle_search(),
                Quit => {
                    writeln!(&mut self.uci_writer_out, "Bye").unwrap();
                    return;
                }
            }
        }
    }

    fn next_command(&mut self) -> Result<UciCommand, Box<dyn Error>> {
        let mut buffer = String::new();
        let mut stdin = io::stdin().lock();

        stdin.read_line(&mut buffer).expect("Uci input failed");

        self.uci_writer_out.log_input(&buffer)?;
        UciCommand::new(&buffer).map_err(|err| err.into())
    }

    fn handle_position_command(&mut self, command: UciPositionCommand) {
        let mut board = Board::new();

        if let Some(fen) = command.fen {
            match Board::new_fen(&fen) {
                Ok(b) => board = b,
                Err(err) => {
                    writeln!(&mut self.uci_writer_out, "Unkown fen: {fen}, {err}").unwrap();
                    return;
                }
            };
        }

        let mut prev_moves = PrevMoves::new();

        for m_str in command.moves {
            let m = Move::new_from_text(&m_str, &board);
            board = board.copy_make(m);
            prev_moves.add(board.hash());
        }

        self.game_state.set_position(board, prev_moves);
    }

    fn handle_search(&mut self) {
        let best_move = match self.game_state.go(&mut self.uci_writer_out) {
            Ok((_, best_move)) => best_move,
            Err(err) => {
                writeln!(&mut self.uci_writer_out, "Could not get best move: {err}");
                return;
            }
        };

        writeln!(
            &mut self.uci_writer_out,
            "bestmove {}",
            best_move.as_uci_string()
        )
        .unwrap();
    }
}

pub struct UciWriter {
    stdout: LineWriter<Stdout>,
    file: Option<LineWriter<File>>,
}

//TODO change this to env variable
const LOG_DIR: &'static str = "/home/george/Nextcloud/progs/rookandroll/logs";

impl UciWriter {
    pub fn new() -> UciWriter {
        UciWriter {
            stdout: LineWriter::new(io::stdout()),
            file: None,
        }
    }

    pub fn new_with_file() -> UciWriter {
        let path = format!("{}/{}_log.log", LOG_DIR, process::id());

        UciWriter {
            stdout: LineWriter::new(io::stdout()),
            file: Some(LineWriter::new(File::create(path).unwrap())),
        }
    }

    pub fn log_input(&mut self, input: &str) -> io::Result<()> {
        match self.file.as_mut() {
            None => Ok(()),
            Some(file) => writeln!(file, "=== {} ===", input.trim()),
        }
    }
}

impl Write for UciWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(file) = self.file.as_mut() {
            file.write_all(buf)?;
        }

        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }

        self.stdout.flush()
    }
}
