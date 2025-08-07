#!/bin/bash

set -e

NETWORK_NAME="batch-proxy-net"

echo "[+] Creating Docker network if it doesn't exist..."
docker network inspect $NETWORK_NAME >/dev/null 2>&1 || \
  docker network create $NETWORK_NAME

echo "[+] Building local batch-proxy image if not already built..."
docker build -t batch-proxy .

echo "[+] Starting batch-proxy server..."
docker run -d --rm \
  --network $NETWORK_NAME \
  --name batch-proxy \
  -p 3000:3000 \
  batch-proxy

echo "[+] Starting Hugging Face embedding server..."
docker run --rm \
  --network $NETWORK_NAME \
  --name embeddings-server \
  -p 8080:80 \
  --pull always \
  ghcr.io/huggingface/text-embeddings-inference:cpu-latest \
  --model-id sentence-transformers/all-MiniLM-L6-v2 &

sleep 5
