#!/bin/bash

if [ "$#" -ne 2 ]; then
  echo "Usage: ./scripts/create_tables.sh <user_name> <db_name>"
  exit 0
fi

# Set environment (dev or prod)
user_name=${1}
db_name=${2}

# File paths
dropfile=./drop_all_tables.sql
user=./user.sql
blog=./blog.sql
book=./book.sql
tags=./tags.sql

# Execute SQL files
psql -h localhost -U $user_name -d $db_name \
  -f $dropfile \
  -f $user \
  -f $blog \
  -f $book \
  -f $tags \
  -W
