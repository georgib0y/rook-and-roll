use std::cell::Cell;
use std::{io, thread};
use std::borrow::BorrowMut;
use std::io::{Error, ErrorKind};
use crate::{Board, Move, SeqTT};
use log::{info};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::task::Poll;
use std::thread::JoinHandle;
use std::time::Duration;
use rayon::max_num_threads;
use crate::moves::{AtomicHistoryTable, HistoryTable, KillerMoves, PrevMoves};
use crate::search::{AbortFlag, iterative_deepening, lazy_smp, Searcher};//, Searches};
use crate::tt::{AtomicTTEntry, Entry, ParaTT, TT, TTable, TTableMT, TTableST, TTEntry};
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

pub type GameStateST = GameState<TTableST>;
pub type GameStateMT = GameState<TTableMT>;


pub struct GameState<T: TT> {
    author: String,
    bot_name: String,
    tt: Option<T>,
    board: Option<Board>,
    prev_moves: Option<PrevMoves>,
    num_threads: usize,
}

impl GameStateST {
    pub fn new_single_threaded(author: &str, bot_name: &str) -> GameStateST {
        GameState {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            tt: Some(TTableST::new_single_threaded()),
            board: Some(Board::new()),
            prev_moves: Some(PrevMoves::new()),
            num_threads: 1,
        }
    }
}

impl GameStateMT {
    pub fn new_multi_threaded(author: &str, bot_name: &str, num_threads: usize) -> GameStateMT {
        GameState {
            author: author.to_string(),
            bot_name: bot_name.to_string(),
            tt: Some(TTable::<AtomicTTEntry>::new_multi_threaded()),
            board: Some(Board::new()),
            prev_moves: Some(PrevMoves::new()),
            num_threads,
        }
    }
}

impl<T: TT> GameState<T>{
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

    pub fn find_best_move(&mut self) -> Move {
        let (tt, best_move) = if self.num_threads <= 1 {
            iterative_deepening(
                self.board.take().unwrap(),
                self.tt.take().unwrap(),
                self.prev_moves.take().unwrap()
            )
        } else {
            lazy_smp(
                self.board.take().unwrap(),
                self.tt.take().unwrap(),
                self.prev_moves.take().unwrap(),
                self.num_threads
            )
        };

        // give ownership of the tt back to the Gamestate
        self.tt = Some(tt);

        best_move.unwrap()
    }

    pub(crate) fn go(&mut self, _args: &str) {
        let best_move = self.find_best_move();
        let out = format!("bestmove {}", best_move.as_uci_string());

        info!(target: "output", "{out}");
        println!("{out}");
    }

    pub(crate) fn ucinewgame(&mut self) {
        if let Some(tt) = self.tt.as_mut() { tt.clear() }
        self.board = Some(Board::new());
        self.prev_moves = Some(PrevMoves::new());
    }
}