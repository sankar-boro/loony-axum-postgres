#!/bin/bash

if [ "$#" -ne 2 ]; then
  echo "Usage: ./scripts/create_tables.sh <user_name> <db_name>"
  exit 0
fi

# Set environment (dev or prod)
user_name=${1}
db_name=${2}

PWD=$(pwd)
dropfile=$PWD/db/postgres/drop_all_tables.sql

psql -h localhost -U $user_name -d $db_name -f $dropfile -W

