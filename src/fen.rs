use crate::board::{gen_hash, Board};
use crate::eval::gen_board_value;
use crate::move_info::SQUARES;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum InvalidFenError {
    InvalidPieces,
    InvalidCTM,
    InvalidCastleState,
    InvalidEpSquare,
    InvalidHalfmove,
}

impl Display for InvalidFenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            InvalidFenError::InvalidPieces => "Invalid Pieces",
            InvalidFenError::InvalidCTM => "Invalid CTM",
            InvalidFenError::InvalidCastleState => "Invalid Castle state",
            InvalidFenError::InvalidEpSquare => "Invalid Ep square",
            InvalidFenError::InvalidHalfmove => "Invalid Halfmove",
        };

        write!(f, "Invalid fen: {msg}")
    }
}

impl Error for InvalidFenError {}

fn piece_from_char(name: char) -> Option<usize> {
    match name {
        'P' => Some(0),
        'p' => Some(1),
        'N' => Some(2),
        'n' => Some(3),
        'R' => Some(4),
        'r' => Some(5),
        'B' => Some(6),
        'b' => Some(7),
        'Q' => Some(8),
        'q' => Some(9),
        'K' => Some(10),
        'k' => Some(11),
        _ => None,
    }
}

// get the amount of squares to increment while iterating through fen rows
fn inc_from_char(name: char) -> Option<usize> {
    match name {
        'P' | 'p' | 'N' | 'n' | 'R' | 'r' | 'B' | 'b' | 'Q' | 'q' | 'K' | 'k' => Some(1),
        '1'..='8' => Some(name as usize - '0' as usize),
        _ => None,
    }
}

fn pieces_from_fen(fen: &str) -> Result<[u64; 12], InvalidFenError> {
    let fen_pieces = fen
        .split(' ')
        .next()
        .ok_or(InvalidFenError::InvalidPieces)?;

    let mut pieces = [0; 12];

    // iterates through each rank of the fen from 1-8
    fen_pieces
        .split('/')
        .collect::<Vec<_>>()
        .iter()
        .rev()
        .enumerate()
        .map(|(i, row)| (i * 8, row))
        .try_for_each(|(mut idx, row)| {
            row.chars().take(8).try_for_each(|sq| {
                if let Some(piece) = piece_from_char(sq) {
                    pieces[piece] ^= SQUARES[idx];
                }

                idx += inc_from_char(sq).ok_or(InvalidFenError::InvalidPieces)?;
                Ok(())
            })
        })?;

    Ok(pieces)
}

fn ctm_from_fen(fen: &str) -> Result<u8, InvalidFenError> {
    let ctm = fen.split(' ').nth(1).ok_or(InvalidFenError::InvalidCTM)?;

    match ctm {
        "w" => Ok(0),
        "b" => Ok(1),
        _ => Err(InvalidFenError::InvalidCTM),
    }
}

fn castle_state_from_fen(fen: &str) -> Result<u8, InvalidFenError> {
    let castle_state_str = fen
        .split(' ')
        .nth(2)
        .ok_or(InvalidFenError::InvalidCastleState)?;

    match castle_state_str {
        "KQkq" => Ok(0b1111),
        "KQk" => Ok(0b1110),
        "KQq" => Ok(0b1101),
        "KQ" => Ok(0b1100),
        "Kkq" => Ok(0b1011),
        "Kk" => Ok(0b1010),
        "Kq" => Ok(0b1001),
        "K" => Ok(0b1000),
        "Qkq" => Ok(0b0111),
        "Qk" => Ok(0b0110),
        "Qq" => Ok(0b0101),
        "Q" => Ok(0b0100),
        "kq" => Ok(0b0011),
        "k" => Ok(0b0010),
        "q" => Ok(0b0001),
        "-" => Ok(0b0000),
        _ => Err(InvalidFenError::InvalidCastleState),
    }
}

fn ep_sq_from_fen(fen: &str) -> Result<u8, InvalidFenError> {
    let ep_sq = fen
        .split(' ')
        .nth(3)
        .ok_or(InvalidFenError::InvalidEpSquare)?;

    if ep_sq.contains('-') {
        return Ok(64);
    }

    // convert file letter to 0-7 value
    let file_char = ep_sq
        .chars()
        .next()
        .ok_or(InvalidFenError::InvalidEpSquare)?;
    if !('a'..='h').contains(&file_char) {
        return Err(InvalidFenError::InvalidEpSquare);
    }

    let file = file_char as usize - 'a' as usize;

    // convert rank to 0-7 value
    let rank_char = ep_sq
        .chars()
        .nth(1)
        .ok_or(InvalidFenError::InvalidEpSquare)?;
    if !(rank_char == '3' || rank_char == '6') {
        return Err(InvalidFenError::InvalidEpSquare);
    }
    let rank = rank_char as usize - '1' as usize;

    Ok((rank * 8 + file) as u8)
}

fn halfmove_from_fen(fen: &str) -> Result<u16, InvalidFenError> {
    let Some(halfmove_str) = fen.split(' ').nth(4) else {
        return Ok(0);
    };

    let halfmove = halfmove_str
        .parse()
        .map_err(|_| InvalidFenError::InvalidHalfmove)?;

    Ok(halfmove)
}

impl Board {
    pub fn new_fen(fen: &str) -> Result<Board, InvalidFenError> {
        let fen = fen.trim();

        let pieces = pieces_from_fen(fen)?;
        let white = pieces[0] | pieces[2] | pieces[4] | pieces[6] | pieces[8] | pieces[10];
        let black = pieces[1] | pieces[3] | pieces[5] | pieces[7] | pieces[9] | pieces[11];

        let mut board = Board {
            pieces,
            util: [white, black, white | black],
            ctm: ctm_from_fen(fen)?,
            castle_state: castle_state_from_fen(fen)?,
            ep: ep_sq_from_fen(fen)?,
            halfmove: halfmove_from_fen(fen)?,
            hash: 0,
            mg_value: 0,
            eg_value: 0,
        };

        // regen the hash after everything is finished
        board.hash = gen_hash(board);
        (board.mg_value, board.eg_value) = gen_board_value(&board);

        Ok(board)
    }
}

#[test]
fn test_fens() -> Result<(), InvalidFenError> {
    crate::init();

    let good_fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2 ",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1",
        "     rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq -      ",
    ];

    for fen in good_fens {
        let board = Board::new_fen(fen)?;
        println!("{}\n{}\n\n", fen, board);
    }

    let bad_fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR KQkq - 0 1",
        "rnbaqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KaQkq c6 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq r5 1 2 ",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - -1 2 ",
    ];

    for fen in bad_fens {
        println!("{}", fen);
        assert!(Board::new_fen(fen).is_err());
    }

    Ok(())
}
