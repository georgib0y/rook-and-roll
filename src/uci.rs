use std::{io};
use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::Arc;
use crate::{Board, Move};
use log::{info};
use crate::lazy_smp::lazy_smp;
use crate::moves::{NULL_MOVE, PrevMoves};
use crate::search::{iterative_deepening};
use crate::tt::{TT, TTable, AtomicTTable};
use crate::tt_entry::{Entry, NoEntry, TTEntry};
use crate::uci::UciCommand::{Go, IsReady, Position, Quit, UciInfo, Ucinewgame};

enum UciCommand {
    Ucinewgame,
    UciInfo,
    IsReady,
    Position(String),
    Go(String),
    Quit
}

impl UciCommand {
    fn new(line: &str) -> Result<UciCommand, ()> {
        let (command, args) = line.split_at(line.find(' ')
            .unwrap_or(line.len()));
        
        match command.trim() {
            "ucinewgame" => Ok(Ucinewgame),
            "uci" => Ok(UciInfo),
            "isready" => Ok(IsReady),
            "position" => Ok(Position(args.to_string())),
            "go" => Ok(Go(args.to_string())),
            "quit" => Ok(Quit),
            _ => Err(())
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
    Self: BestMoveFinder
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
        num_threads: usize
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

            io::stdin().read_line(&mut buffer).expect("Uci input failed");
                info!(target: "input", "{buffer}");

            let Ok(command) = UciCommand::new(&buffer) else { continue };

            match command {
                Ucinewgame => self.ucinewgame(),
                UciInfo => self.uci_info(),
                IsReady => self.is_ready(),
                Position(args) => self.position(&args),
                Go(args) => self.go(&args),
                Quit => break
            }
        }
    }

    fn uci_info(&self) {
        let out = format!("id name {}\nid author {}\nuciok", self.bot_name, self.author);
        info!(target: "output", "{out}");
        println!("{out}");
    }
    
    fn is_ready(&self) {
        info!(target: "output", "readyok");
        println!("readyok");
    }

    pub(crate) fn position(&mut self, buffer: &str) {
        let mut board = if buffer.contains("fen") {
            // split after postition fen ...
            let mut fen = buffer.split_once("fen ").unwrap().1;
            // if there are extra moves afterwards (fen ... moves e3g8 f4f4 ... split before then
            if fen.contains("moves") {
                fen = fen.split_once(" moves").unwrap().0;
            }
            // println!("{fen}");
            Board::new_fen(fen).unwrap()
        } else {
            // if "startpos"
            Board::new()
        };

        let mut prev_moves = PrevMoves::new();

        if buffer.contains("moves") {
            let moves: Vec<&str> = buffer
                .trim()
                .split_once("moves ")
                .unwrap() .1
                .split(' ')
                .collect();

            for m in moves {
                let mv = Move::new_from_text(m, &board);
                board = board.copy_make(mv);
                let hash = board.hash;
                prev_moves.add(hash);
            }
        }

        self.prev_moves = Some(prev_moves);
        self.board = Some(board);
    }

    pub(crate) fn go(&mut self, _args: &str) {
        let best_move = self.find_best_move().unwrap_or(NULL_MOVE);
        let out = format!("bestmove {}", best_move.as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");
    }

    pub(crate) fn ucinewgame(&mut self) {
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
            self.prev_moves.take().unwrap()
        )
    }
}

impl BestMoveFinder for GameState<Arc<AtomicTTable>> {
    fn find_best_move(&mut self) -> Option<Move> {
        lazy_smp(
            self.board.take().unwrap(),
            Arc::clone(&self.tt),
            self.prev_moves.take().unwrap(),
            self.num_threads
        )
    }
}