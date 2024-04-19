use std::sync::Arc;

use crate::{
    board::Board,
    error::{ArbiterError, InvalidUciCommand},
    game_state::GameState,
    move_list::{StackMoveList, MAX_MOVES},
    movegen::{gen_moves, is_in_check, is_legal_move, moved_into_check},
    moves::{Move, PrevMoves},
    tt::NoTTable,
    uci::{Uci, UciCommand},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Router};
use tokio::sync::Mutex;

struct Arbiter<T: Uci> {
    bot: T,
    board: Board,
    prev_moves: PrevMoves,
}

impl<T: Uci> Arbiter<T> {
    pub fn new() -> Arbiter<GameState<NoTTable>> {
        Arbiter {
            bot: GameState::new_no_tt(),
            board: Board::new(),
            prev_moves: PrevMoves::new(),
        }
    }

    pub fn apply_player_move(&mut self, m: Move) -> Result<(), ArbiterError> {
        let mut ml = StackMoveList::<MAX_MOVES>::new();
        gen_moves(&self.board, &mut ml, is_in_check(&self.board));

        if !ml.contains_move(m)
            || moved_into_check(&self.board, m)
            || !is_legal_move(&self.board, m, &self.prev_moves)
        {
            Err(ArbiterError::IllegalMove(m))?;
        }

        self.board = self.board.copy_make(m);
        self.prev_moves.add(self.board.hash());

        Ok(())
    }
}

pub async fn run_http() {
    // let game_state = Arc::new(Mutex::new(GameState::new()));
    let game_state = Arc::new(Mutex::new(GameState::new_no_tt()));

    let app = Router::new().route("/", post(uci)).with_state(game_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn uci(
    State(state): State<Arc<Mutex<impl Uci>>>,
    cmd_str: String,
) -> Result<String, InvalidUciCommand> {
    Ok(state
        .lock()
        .await
        .do_command(UciCommand::new(&cmd_str)?)
        .unwrap_or(String::new()))
}

impl IntoResponse for InvalidUciCommand {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
