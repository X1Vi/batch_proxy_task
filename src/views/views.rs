use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use tokio::sync::oneshot;

use crate::{AppState, BatchRequest, UserInput, EmbedResult}; // Assuming these are defined in your crate

pub async fn embed_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserInput>,
) -> impl IntoResponse {
    let mut response_vec = Vec::new();

    for input_text in payload.inputs {
        let (resp_tx, resp_rx) = oneshot::channel();

        let batch_req = BatchRequest {
            input: input_text,
            resp_tx,
        };

        if state.batch_tx.send(batch_req).await.is_err() {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "Batch queue closed".to_string(),
            )
                .into_response();
        }

        match resp_rx.await {
            Ok(Ok(embed_result)) => response_vec.push(embed_result),
            Ok(Err(err_msg)) => {
                return (StatusCode::BAD_GATEWAY, err_msg).into_response();
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Batch worker task aborted".to_string(),
                )
                    .into_response();
            }
        }
    }

    Json(response_vec).into_response()
}