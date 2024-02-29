#!/usr/bin/env sh

cargo build --release

RUST_LOG=info,debug ./target/release/loony