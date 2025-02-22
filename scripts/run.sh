#!/usr/bin/env sh

# Check if the number of arguments is exactly 2
if [ "$#" -eq 1 ]; then
    if [ "$1" = "--build-backend" ]; then
        echo "Building loony-axum-postgres..."
        cargo build --release
    fi
fi

HOME=$HOME
V1_PORT=8000
V1_HOSTNAME=localhost
TMP_UPLOADS="$HOME/.tmp_uploads"
BLOG_UPLOADS="$HOME/.blog_uploads"
BOOK_UPLOADS="$HOME/.book_uploads"
USER_UPLOADS="$HOME/.user_uploads"
V1_PG_HOSTNAME="localhost"
V1_PG_USERNAME="<username>"
V1_PG_DBNAME="<dbname>"
V1_PG_PASSWORD="<password>"
V1_SECRET_KEY="<secret_key>"
V1_ALLOWED_ORIGINS="http://localhost,http://localhost:3000,http://127.0.0.1:8081,http://10.0.2.2:8081"

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
V1_HOSTNAME="$V1_HOSTNAME" \
V1_PORT="$V1_PORT" \
V1_PG_HOSTNAME="$V1_HOSTNAME" \
V1_PG_USERNAME="$V1_PG_USERNAME" \
V1_PG_DBNAME="$V1_PG_DBNAME" \
V1_PG_PASSWORD="$V1_PG_PASSWORD" \
V1_ALLOWED_ORIGINS="$V1_ALLOWED_ORIGINS" \
V1_SECRET_KEY="$V1_SECRET_KEY" \
TMP_UPLOADS="$TMP_UPLOADS" \
BLOG_UPLOADS="$BLOG_UPLOADS" \
BOOK_UPLOADS="$BOOK_UPLOADS" \
USER_UPLOADS="$USER_UPLOADS" \
./target/release/loony_axum_postgres
