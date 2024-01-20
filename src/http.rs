use std::sync::Arc;

use crate::{
    game_state::GameState,
    uci::{InvalidUciCommand, Uci, UciCommand},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Router};
use tokio::sync::Mutex;

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
