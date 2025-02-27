#!/bin/bash

if [ "$#" -ne 2 ]; then
  echo "Usage: ./drop.sh <user_name> <db_name>"
  exit 0
fi

# Set environment (dev or prod)
user_name=${1}
db_name=${2}

PWD=$(pwd)
dropfile=./drop_all_tables.sql

psql -h localhost -U $user_name -d $db_name -f $dropfile -W

