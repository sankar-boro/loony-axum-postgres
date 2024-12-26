#!/bin/bash

# Set environment (dev or prod)
environment=${1:-dev}

# Define database names for different environments
if [ "$environment" == "prod" ]; then
  db_name="loony"
  user_name="loony"
else
  db_name="sankar"
  user_name="sankar"
fi

# File paths
dropfile=$PWD/db/postgres/drop_all_tables.sql
user=$PWD/db/postgres/user.sql
blog=$PWD/db/postgres/blog.sql
book=$PWD/db/postgres/book.sql
tags=$PWD/db/postgres/tags.sql

# Execute SQL files
psql -h localhost -U $user_name -d $db_name \
  -f $dropfile \
  -f $user \
  -f $blog \
  -f $book \
  -f $tags \
  -W
