```markdown
# 🧠 Batch Embedding Proxy

Simple Rust proxy that batches embedding requests and forwards them to an inference server.

## ⚙️ Setup

### 1. Run the inference container
This uses HuggingFace's inference server with Nomic's embedding model.

```bash
docker run --rm -it -p 8080:80 --pull always \
ghcr.io/huggingface/text-embeddings-inference:cpu-latest \
--model-id nomic-ai/nomic-embed-text-v1.5
```

### 2. Run the proxy server

```bash
./batch_proxy_task.exe
```

The proxy starts on:  
`http://localhost:3000`

## 🔁 API Usage

**Endpoint:**  
`POST http://localhost:3000/embed`

**Payload format:**

```json
{
  "inputs": ["string1", "string2", "string3"]
}
```

### 🧪 Example:

```bash
curl -X POST http://localhost:3000/embed \
  -H "Content-Type: application/json" \
  -d '{"inputs": ["What is Vector Search?", "Hello, world!"]}'
```

## ⚡ Benchmark Notes

| Type              | Time      |
|-------------------|-----------|
| Single request    | ~5ms avg  |
| Full batch (8x)   | ~265ms    |

## ⚙️ Config (hardcoded in main.rs)

- Max Batch Size: `8`
- Max Wait Time: `50ms`
- Backend URL: `http://localhost:8080/embed`
```
