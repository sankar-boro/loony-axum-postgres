#!/usr/bin/env sh

# Check if the number of arguments is exactly 2
if [ "$#" -eq 1 ]; then
    if [ "$1" = "--build-backend" ]; then
        echo "Building loony-axum-postgres..."
        cargo build --release
    fi
fi

PORT=8000
USERNAME="sankar"
HOME=$HOME
TMP_UPLOADS="$HOME/.tmp_uploads"
BLOG_UPLOADS="$HOME/.blog_uploads"
BOOK_UPLOADS="$HOME/.book_uploads"
USER_UPLOADS="$HOME/.user_uploads"
ORIGINS="https://sankarboro.com,https://web.sankarboro.com,http://localhost,http://localhost:3000,http://127.0.0.1:8081,http://10.0.2.2:8081"
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
PORT="$PORT" \
PASSWORD="$USERNAME" \
PG_HOST=localhost \
PG_USER="$USERNAME" \
PG_DBNAME="$USERNAME" \
PG_PASSWORD="$USERNAME" \
ORIGINS="$ORIGINS" \
SECRET_KEY="$SECRET_KEY" \
TMP_UPLOADS="$TMP_UPLOADS" \
BLOG_UPLOADS="$BLOG_UPLOADS" \
BOOK_UPLOADS="$BOOK_UPLOADS" \
USER_UPLOADS="$USER_UPLOADS" \
./target/release/loony_axum_postgres
