# 🧠 Batch Embedding Proxy

Simple Rust proxy that batches embedding requests and forwards them to an inference server.

---

## ⚙️ Setup

### 1. Run the inference container

This uses HuggingFace's inference server with Nomic's embedding model.

```bash
docker run --rm -it -p 8080:80 --pull always \
ghcr.io/huggingface/text-embeddings-inference:cpu-latest \
--model-id nomic-ai/nomic-embed-text-v1.5
```

### 2. Clone and run the Rust server

```bash
git clone https://github.com/X1Vi/batch_proxy_task.git
cd batch_proxy_task
cargo run
```

---

## 📦 API

### `POST /embed`

Accepts a JSON payload with an array of input strings. Example:

```json
{
  "inputs": [
    "What is Vector Search?",
    "Hello, world!"
  ]
}
```

Response:

```json
{
  "embeddings": [
    [0.011, 0.005, ...],
    [0.017, 0.002, ...]
  ]
}
```

---

## 🔧 Configuration

You can configure the proxy using environment variables:

- `EMBEDDING_API_URL` (default: `http://localhost:8080`)
- `BATCH_SIZE` (default: `8`)
- `BATCH_TIMEOUT_MS` (default: `200`)

---

## 🚀 Example Request

```bash
curl -X POST http://localhost:3000/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs": ["What is Vector Search?", "Hello, world!"]}'
```

---
