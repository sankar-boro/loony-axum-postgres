#!/usr/bin/env sh

cargo build --release

RUST_LOG=info HOST=localhost PORT=5002 ALLOW_ORIGIN=http://localhost:5000 ./target/release/loony