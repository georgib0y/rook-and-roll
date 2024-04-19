use thiserror::Error;

use crate::moves::Move;

#[derive(Error, Debug)]
pub enum InvalidFenError {
    #[error("Invalid Pieces")]
    InvalidPieces,
    #[error("Invalid CTM")]
    InvalidCTM,
    #[error("Invalid Castle State")]
    InvalidCastleState,
    #[error("Invalid Ep Square")]
    InvalidEpSquare,
    #[error("Invalid HalfMove")]
    InvalidHalfmove,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("No moves found")]
    NoMove,
    #[error("Failed low")]
    FailLow,
    #[error("Failed high")]
    FailHigh,
}

#[derive(Error, Debug)]
pub enum ArbiterError {
    #[error("Illegal Move: {0}")]
    IllegalMove(Move),
}

#[derive(Error, Debug)]
pub enum InvalidUciCommand {
    #[error("Unknown UCI command: {0}")]
    UnknownCommand(String),
    #[error("Invalid position: {0}")]
    InvalidPositionCommand(String),
}
