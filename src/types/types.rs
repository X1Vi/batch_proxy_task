use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use reqwest::Client;



#[derive(Clone)]
pub struct AppConfig {
    pub max_batch_size: usize,
    pub max_wait_time_ms: u64,
    pub backend_url: String,
}

pub struct AppState {
    pub batch_tx: mpsc::Sender<BatchRequest>,
    pub config: AppConfig,
    pub client: Client,
}

#[derive(Debug, Deserialize)]
pub struct UserInput {
    pub inputs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmbedResult {
    pub embedding: Vec<f32>,
}

pub struct BatchRequest {
    pub input: String,
    pub resp_tx: oneshot::Sender<Result<EmbedResult, String>>,
}