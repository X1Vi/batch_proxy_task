# 1. Build stage
FROM rust:1.88-slim as builder

# Install required packages
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential ca-certificates

# Create a new empty shell project
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -r src

# Copy actual source
COPY . .

# Build actual project
RUN cargo build --release

# 2. Runtime stage
FROM debian:bookworm-slim

# Install OpenSSL 3 runtime libs
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary
COPY --from=builder /app/target/release/batch_proxy_task /usr/local/bin/batch_proxy_task

# Expose port (change if needed)
EXPOSE 3000

# Run the binary
CMD ["batch_proxy_task"]
