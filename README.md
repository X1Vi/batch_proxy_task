# Auto-Batching Text Embedding Proxy

This project provides an auto-batching proxy service that forwards input text to a local Hugging Face embedding model served via Docker.

## Setup

1. Run `.setup_docker.sh` to set up the entire project using Docker.

2. Once the service is running, hit the following endpoint:

   POST http://0.0.0.0:3000/embed

   with this JSON body:

   ```json
   {
     "inputs": [
       "string1",
       "string2",
       "strnng3",
       "string4"
     ]
   }
   ```

## Benchmark Logs

| Total Time | Tokenization | Queue Time | Inference Time |
|------------|--------------|------------|----------------|
| 5.151 ms   | 298.6 µs     | 613.8 µs   | 4.070 ms       |
| 4.491 ms   | 166.8 µs     | 446.8 µs   | 3.725 ms       |
| 10.419 ms  | 158.7 µs     | 484.1 µs   | 9.704 ms       |
| 4.770 ms   | 210.0 µs     | 483.2 µs   | 3.876 ms       |

**Batch Embedding Time (entire batch)**: **239 ms**
