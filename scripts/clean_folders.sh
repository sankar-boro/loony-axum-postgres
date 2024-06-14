#!/usr/bin/env sh

# Check if the number of arguments is exactly 2
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <arg1>"
    exit 1
fi

HOME=$1
TMP_UPLOADS="$HOME/.tmp_uploads"
BLOG_UPLOADS="$HOME/.blog_uploads"
BOOK_UPLOADS="$HOME/.book_uploads"
USER_UPLOADS="$HOME/.user_uploads"

if [ -d "$TMP_UPLOADS" ]; then
    rm -rf $TMP_UPLOADS/*
fi
if [ -d "$BLOG_UPLOADS" ]; then
    rm -rf $BLOG_UPLOADS/*
fi
if [ -d "$BOOK_UPLOADS" ]; then
    rm -rf $BOOK_UPLOADS/*
fi
if [ -d "$USER_UPLOADS" ]; then
    rm -rf $USER_UPLOADS/*
fi
