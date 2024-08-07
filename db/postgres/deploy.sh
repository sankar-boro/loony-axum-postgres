#!/bin/bash

PWD=$(pwd)

dropfile=$PWD/db/postgres/drop.sql
user=$PWD/db/postgres/user.sql
blog=$PWD/db/postgres/blog.sql
book=$PWD/db/postgres/book.sql
tags=$PWD/db/postgres/tags.sql

psql -h localhost -U sankar -d sankar \
-f $dropfile \
-f $user \
-f $blog \
-f $book \
-f $tags \
-W
