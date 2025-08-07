use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::{
    pin,
    sync::{mpsc, oneshot},
};

#[derive(Clone)]
struct AppConfig {
    max_batch_size: usize,
    max_wait_time_ms: u64,
    backend_url: String,
}

struct AppState {
    batch_tx: mpsc::Sender<BatchRequest>,
    config: AppConfig,
    client: Client,
}

#[derive(Debug, Deserialize)]
struct UserInput {
    inputs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EmbedResult {
    embedding: Vec<f32>,
}

struct BatchRequest {
    input: String,
    resp_tx: oneshot::Sender<Result<EmbedResult, String>>,
}

#[tokio::main]
async fn main() {
    // Configuration
    let config = AppConfig {
        max_batch_size: 8,
        max_wait_time_ms: 50,
        backend_url: "http://localhost:8080/embed".to_string(),
    };

    let client = Client::new();

    let (batch_tx, mut batch_rx) = mpsc::channel::<BatchRequest>(100);

    let config2 = config.clone();
    let client2 = client.clone();

    tokio::spawn(async move {
        loop {
            // Collect requests and responders as usual
            let mut requests = Vec::with_capacity(config2.max_batch_size);
            let mut responders = Vec::with_capacity(config2.max_batch_size);

            let first = batch_rx.recv().await;
            if let Some(req) = first {
                requests.push(req.input);
                responders.push(req.resp_tx);
            } else {
                break;
            }

            let timer = tokio::time::sleep(Duration::from_millis(config2.max_wait_time_ms));
            pin!(timer);

            while requests.len() < config2.max_batch_size {
                tokio::select! {
                    Some(req) = batch_rx.recv() => {
                        requests.push(req.input);
                        responders.push(req.resp_tx);

                        if requests.len() == config2.max_batch_size {
                            break;
                        }
                    }
                    _ = &mut timer => {
                        break;
                    }
                }
            }

            let batch_payload = json!({
                "inputs": requests,
            });

            // Start batch timer
            let batch_start = std::time::Instant::now();

            let resp_result = client2
                .post(&config2.backend_url)
                .json(&batch_payload)
                .send()
                .await;

            match resp_result {
                Ok(mut resp) => {
                    println!("Backend response status: {}", resp.status());

                    let text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "<Failed to read body>".to_string());
                    println!("Backend response body:\n{}", text);

                    match serde_json::from_str::<Vec<Vec<f32>>>(&text) {
                        Ok(embeddings_vec) => {
                            for (idx, (resp_tx, embedding)) in
                                responders.into_iter().zip(embeddings_vec).enumerate()
                            {
                                // Time each individual request send back
                                let req_start = std::time::Instant::now();
                                let _ = resp_tx.send(Ok(EmbedResult { embedding }));
                                let req_end = std::time::Instant::now();
                                println!(
                                    "Request {} response sending time: {:.4?}",
                                    idx + 1,
                                    req_end.duration_since(req_start)
                                );
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to parse backend JSON: {}", e);
                            for resp_tx in responders {
                                let _ = resp_tx.send(Err(err_msg.clone()));
                            }
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("Backend request failed: {}", e);
                    for resp_tx in responders {
                        let _ = resp_tx.send(Err(err_msg.clone()));
                    }
                }
            }

            // Print total batch processing time
            let batch_end = std::time::Instant::now();
            println!(
                "Total batch processing time: {:.4?}",
                batch_end.duration_since(batch_start)
            );
        }
    });

    let app_state = Arc::new(AppState {
        batch_tx,
        config,
        client,
    });

    let app = Router::new()
        .route("/embed", post(embed_handler))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000 ...");
    axum::serve(listener, app).await.unwrap();
}
async fn embed_handler(
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
