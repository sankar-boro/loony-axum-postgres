#!/usr/bin/env sh

cargo build --release

RUST_LOG=info \
HOST=localhost \
PORT=5002 \
PASSWORD=sankar \
PG_HOST=localhost \
PG_USER=sankar \
PG_DBNAME=sankar \
PB_PASSWORD=sankar \
ALLOW_ORIGIN=http://localhost:3000 \
SECRET_KEY=lorem_ipsum_dolor_isset \
./target/release/loony