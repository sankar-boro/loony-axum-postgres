# ----------- Stage 1: Builder -----------
FROM rust:1.85.0 AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source and build again
COPY . .
RUN cargo build --release

# ---------- Runtime Stage ----------
FROM debian:bookworm-slim

# Install system dependencies if needed (like OpenSSL for Diesel or Reqwest)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/loony-api /app/loony-api

# Set environment variables (optional)
# ENV RUST_LOG=info
# ENV DATABASE_URL=postgres://postgres:postgres@db:5432/postgres

# Expose the port your Rust app listens on
# EXPOSE 8000

# Start app
ENTRYPOINT ["./loony-api"]
