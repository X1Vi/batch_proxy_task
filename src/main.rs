use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use reqwest::{header::HOST, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::{
    pin,
    sync::{mpsc, oneshot},
};

mod types;
mod views;

use crate::types::types::{AppConfig, AppState, BatchRequest, UserInput, EmbedResult};
use crate::views::views::embed_handler;


const MAX_BATCH_SIZE: usize = 8;
const MAX_WAIT_TIME : u64 = 50;
const PORT : &str = "3000";
const IP : &str = "0.0.0.0";

const HUGGING_FACE_ADDRESS: &str = "embeddings-server";
const HUGGING_FACE_PORT : &str = "80";

#[tokio::main]
async fn main() {
    // Setup configuration and state
    let config = setup_config();
    let client = Client::new();
    let (batch_tx, batch_rx) = mpsc::channel::<BatchRequest>(100);

    // Spawn batch processor task
    spawn_batch_processor(config.clone(), client.clone(), batch_rx);

    let app_state = Arc::new(AppState {
        batch_tx,
        config,
        client,
    });

    // Run the HTTP server
    run_server(app_state).await;
}

// Function to setup application configuration
fn setup_config() -> AppConfig {
    let hugging_face_host_link = format!("http://{}:{}", HUGGING_FACE_ADDRESS, HUGGING_FACE_PORT);
    AppConfig {
        max_batch_size: MAX_BATCH_SIZE,
        max_wait_time_ms: MAX_WAIT_TIME,
        backend_url: hugging_face_host_link.to_string(),
    }
}

// Function to spawn the background task that processes batch requests
fn spawn_batch_processor(config: AppConfig, client: Client, mut batch_rx: mpsc::Receiver<BatchRequest>) {
    tokio::spawn(async move {
        loop {
            match collect_batch(&config, &mut batch_rx).await {
                Some((requests, responders)) => {
                    process_batch(&config, &client, requests, responders).await;
                }
                None => break, // Channel closed
            }
        }
    });
}

/// Collects a batch of requests up to max_batch_size or max_wait_time_ms
async fn collect_batch(
    config: &AppConfig,
    batch_rx: &mut mpsc::Receiver<BatchRequest>,
) -> Option<(Vec<String>, Vec<oneshot::Sender<Result<EmbedResult, String>>>)> {
    let mut requests = Vec::with_capacity(config.max_batch_size);
    let mut responders = Vec::with_capacity(config.max_batch_size);

    // Receive the first request, if none then return None meaning channel closed
    let first = batch_rx.recv().await?;
    requests.push(first.input);
    responders.push(first.resp_tx);

    let timer = tokio::time::sleep(Duration::from_millis(config.max_wait_time_ms));
    pin!(timer);

    // Collect requests until batch is full or timer expires
    while requests.len() < config.max_batch_size {
        tokio::select! {
            Some(req) = batch_rx.recv() => {
                requests.push(req.input);
                responders.push(req.resp_tx);

                if requests.len() == config.max_batch_size {
                    break;
                }
            }
            _ = &mut timer => {
                break;
            }
        }
    }

    Some((requests, responders))
}

/// Sends the batch to backend, parses response, and sends results back to requesters
async fn process_batch(
    config: &AppConfig,
    client: &Client,
    requests: Vec<String>,
    responders: Vec<oneshot::Sender<Result<EmbedResult, String>>>,
) {
    let batch_payload = json!({
        "inputs": requests,
    });

    let batch_start = std::time::Instant::now();

    let resp_result = client
        .post(&config.backend_url)
        .json(&batch_payload)
        .send()
        .await;

    match resp_result {
        Ok(mut resp) => {
            println!("Backend response status: {}", resp.status());

            let text = resp.text().await.unwrap_or_else(|_| "<Failed to read body>".to_string());
            println!("Backend response body:\n{}", text);

            match serde_json::from_str::<Vec<Vec<f32>>>(&text) {
                Ok(embeddings_vec) => {
                    for (idx, (resp_tx, embedding)) in responders.into_iter().zip(embeddings_vec).enumerate() {
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

    let batch_end = std::time::Instant::now();
    println!(
        "Total batch processing time: {:.4?}",
        batch_end.duration_since(batch_start)
    );
}

// Function to start the HTTP server
async fn run_server(app_state: Arc<AppState>) {
    let app = Router::new()
        .route("/embed", post(embed_handler))
        .with_state(app_state);
    let host_link = format!("{}:{}", IP, PORT.to_string());
    let listener = TcpListener::bind(host_link).await.unwrap();
    println!("Listening on http://0.0.0.0:3000 ...");
    axum::serve(listener, app).await.unwrap();
}
