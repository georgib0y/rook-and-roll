use std::io;
use std::io::{Error, ErrorKind, Read};
use crate::{Board, Move, SeqTT};
use log::{info};
use std::process::exit;
use std::sync::Arc;
use crate::moves::{KillerMoves, PrevMoves};
use crate::search::iterative_deepening;
use crate::smp::lazy_smp;
use crate::tt::ParaTT;
use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, Ucinewgame};

pub struct GameState {
    author: String,
    bot_name: String,
    board: Board,
    tt: SeqTT,
    km: KillerMoves,
    prev_moves: Option<PrevMoves>
}

impl GameState {
    pub fn new(author: &str, bot_name: &str) -> GameState {
        GameState {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            board: Board::new(),
            tt: SeqTT::new(),
            km: KillerMoves::new(),
            prev_moves: None
        }
    }
}

impl Uci for GameState {
    fn ucinewgame(&mut self) {
        self.tt.clear();
        self.board = Board::new();
        self.km = KillerMoves::new();
        self.prev_moves = None
    }

    fn find_best_move(&mut self) -> Option<Move> {
        iterative_deepening( &self.board, &mut self.tt, &mut self.km, self.prev_moves.take().unwrap() )
    }

    fn author(&self) -> &str { &self.author }

    fn bot_name(&self) -> & str { &self.bot_name }

    fn board(&mut self) -> &mut Board { &mut self.board }

    fn clear_prev_moves(&mut self) { self.prev_moves = Some(PrevMoves::new()) }

    fn prev_moves(&mut self) -> &mut PrevMoves { self.prev_moves.as_mut().unwrap() }
}

pub struct GameStateMT {
    author: String,
    bot_name: String,
    board: Board,
    tt: Arc<ParaTT>,
    km: KillerMoves,
    prev_moves: Option<PrevMoves>,
    num_threads: usize
}

impl GameStateMT {
    pub fn new(author: &str, bot_name: &str, num_threads: usize) -> GameStateMT {
        GameStateMT {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            board: Board::new(),
            tt: Arc::new(ParaTT::new()),
            km: KillerMoves::new(),
            prev_moves: None,
            num_threads
        }
    }
}

impl Uci for GameStateMT {
    fn ucinewgame(&mut self) {
        self.board = Board::new();
        self.tt.clear();
        self.km = KillerMoves::new();
        self.prev_moves = None
    }

    fn find_best_move(&mut self) -> Option<Move> {
        lazy_smp(
            &self.board,
            Arc::clone(&self.tt),
            self.prev_moves.take().unwrap(),
            self.num_threads
        )
    }

    fn author(&self) -> &str { &self.author }

    fn bot_name(&self) -> &str { &self.bot_name }

    fn board(&mut self) -> &mut Board { &mut self.board }

    fn clear_prev_moves(&mut self) { self.prev_moves = Some(PrevMoves::new()) }

    fn prev_moves(&mut self) -> &mut PrevMoves { self.prev_moves.as_mut().unwrap() }
}

enum UciCommand <'a> {
    Ucinewgame,
    UciInfo,
    IsReady,
    Position(&'a str),
    Go(&'a str),
    Quit
}

impl <'a> UciCommand <'a> {
    fn new(line: &'a str) -> Result<UciCommand<'a>, Error> {
        let (command, args) = line.split_at(line.find(' ')
            .unwrap_or(line.len()));
        
        match command.trim() {
            "ucinewgame" => Ok(Ucinewgame),
            "uci" => Ok(UciInfo),
            "isready" => Ok(IsReady),
            "position" => Ok(Position(args)),
            "go" => Ok(Go(args)),
            "quit" => Ok(Quit),
            _ => Err(Error::new(
                ErrorKind::InvalidInput, 
                format!("command: \"{command}\" not found")
                )
            )
        }
    }
}

pub trait Uci {
    fn start(&mut self) {
        let mut buffer = String::new();

        loop {
            io::stdin().read_line(&mut buffer).expect("Uci input failed");
            info!(target: "input", "{buffer}");
            
            let command = match UciCommand::new(&buffer) {
                Ok(command) => command,
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    continue;
                }
            };

            match command {
                Ucinewgame => self.ucinewgame(),
                UciInfo => self.uci_info(),
                IsReady => self.is_ready(),
                Position(args) => self.position(args),
                Go(args) => self.go(args),
                Quit => break
            }

            buffer.clear();
        }
    }

    fn ucinewgame(&mut self);
    
    fn uci_info(&self) {
        let out = format!("id name {}\nid author {}\nuciok", self.bot_name(), self.author());
        info!(target: "output", "{out}");
        println!("{out}");
    }
    
    fn is_ready(&self) {
        info!(target: "output", "readyok");
        println!("readyok");
    }

    fn position(&mut self, buffer: &str) {
        *self.board() = if buffer.contains("fen") {
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

        self.clear_prev_moves();

        if buffer.contains("moves") {
            let moves: Vec<&str> = buffer
                .trim()
                .split_once("moves ")
                .unwrap()
                .1
                .split(' ')
                .collect();

            for m in moves {
                let mv = Move::new_from_text(m, &self.board());
                *self.board() = self.board().copy_make(mv);
                let hash = self.board().hash;
                self.prev_moves().add(hash);
            }
        }
    }

    fn go(&mut self, _args: &str) {
        let best_move = self.find_best_move().expect("Did not find a best move");

        let out = format!("bestmove {}", best_move.as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");
    }

    fn find_best_move(&mut self) -> Option<Move>;

    fn author(&self) -> &str;

    fn bot_name(&self) -> &str;

    fn board(&mut self) -> &mut Board;

    fn clear_prev_moves(&mut self);

    fn prev_moves(&mut self) -> &mut PrevMoves;
}
