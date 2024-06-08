#!/usr/bin/env sh

cargo build --release

#!/bin/bash

DIRECTORY="/home/sankar/.tmp_uploads"
DIRECTORY1="/home/sankar/.doc_uploads"
DIRECTORY2="/home/sankar/.user_uploads"

if [ ! -d "$DIRECTORY" ]; then
    mkdir $DIRECTORY
fi
if [ ! -d "$DIRECTORY1" ]; then
    mkdir $DIRECTORY1
fi
if [ ! -d "$DIRECTORY2" ]; then
    mkdir $DIRECTORY2
fi

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