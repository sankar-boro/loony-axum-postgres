#!/usr/bin/env sh

cargo build --release

USERNAME="sankar"
HOME="/home/sankar"
TMP_UPLOADS="$HOME/.tmp_uploads"
BLOG_UPLOADS="$HOME/.blog_uploads"
BOOK_UPLOADS="$HOME/.book_uploads"
USER_UPLOADS="$HOME/.user_uploads"
ALLOW_ORIGIN="http://localhost:3000"
SECRET_KEY="lorem_ipsum_dolor_isset"

if [ ! -d "$TMP_UPLOADS" ]; then
    mkdir $TMP_UPLOADS
fi
if [ ! -d "$BLOG_UPLOADS" ]; then
    mkdir $BLOG_UPLOADS
fi
if [ ! -d "$BOOK_UPLOADS" ]; then
    mkdir $BOOK_UPLOADS
fi
if [ ! -d "$USER_UPLOADS" ]; then
    mkdir $USER_UPLOADS
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
TMP_UPLOADS="$TMP_UPLOADS" \
BLOG_UPLOADS="$BLOG_UPLOADS" \
BOOK_UPLOADS="$BOOK_UPLOADS" \
USER_UPLOADS="$USER_UPLOADS" \
./target/release/loony