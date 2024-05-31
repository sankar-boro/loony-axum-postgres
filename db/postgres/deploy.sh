#!/bin/bash

PWD=$(pwd)
dropfile=$PWD/db/postgres/drop.sql

blog=$PWD/db/postgres/blog.sql
book=$PWD/db/postgres/book.sql
user=$PWD/db/postgres/user.sql

psql -h localhost -U sankar -d sankar \
-f $blog \
-f $book \
-f $user \
-W
