#!/usr/bin/env sh

cargo build --release

USERNAME="sankar"
HOME="/home/sankar"
TMP_UPLOADS_DIRECTORY="$HOME/.tmp_uploads"
DOCUMENT_UPLOADS_DIRECTORY="$HOME/.doc_uploads"
USER_UPLOADS_DIR="$HOME/.user_uploads"
ALLOW_ORIGIN="http://localhost:3000"
SECRET_KEY="lorem_ipsum_dolor_isset"

if [ ! -d "$TMP_UPLOADS_DIRECTORY" ]; then
    mkdir $TMP_UPLOADS_DIRECTORY
fi
if [ ! -d "$DOCUMENT_UPLOADS_DIRECTORY" ]; then
    mkdir $DOCUMENT_UPLOADS_DIRECTORY
fi
if [ ! -d "$USER_UPLOADS_DIR" ]; then
    mkdir $USER_UPLOADS_DIR
fi

RUST_LOG=info \
HOST=localhost \
PORT=5002 \
PASSWORD="$USERNAME" \
PG_HOST=localhost \
PG_USER="$USERNAME" \
PG_DBNAME="$USERNAME" \
PG_PASSWORD="$USERNAME" \
ALLOW_ORIGIN="$ALLOW_ORIGIN" \
SECRET_KEY="$SECRET_KEY" \
FILE_UPLOADS_TMP="$TMP_UPLOADS_DIRECTORY" \
FILE_UPLOADS_DOC="$DOCUMENT_UPLOADS_DIRECTORY" \
FILE_UPLOADS_USER="$USER_UPLOADS_DIR" \
./target/release/loony