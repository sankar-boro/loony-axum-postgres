#!/usr/bin/env sh

cargo build --release

RUST_LOG=info \
HOST=localhost \
PORT=5002 \
PASSWORD=sankar \
PG_HOST=localhost \
PG_USER=sankar \
PG_DBNAME=sankar \
PG_PASSWORD=sankar \
ALLOW_ORIGIN=http://localhost:3000 \
SECRET_KEY=lorem_ipsum_dolor_isset \
FILE_UPLOADS_TMP=/home/sankar/.tmp_uploads \
FILE_UPLOADS_DOC=/home/sankar/.doc_uploads \
FILE_UPLOADS_USER=/home/sankar/.user_uploads \
./target/release/loony