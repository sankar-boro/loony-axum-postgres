#!/bin/bash

PWD=$(pwd)
dropfile=$PWD/lily_setup/postgres/drop.sql

blog=$PWD/lily_setup/postgres/blog.sql
book=$PWD/lily_setup/postgres/book.sql
user=$PWD/lily_setup/postgres/user.sql

psql -h localhost -U sankar -d sankar \
-f $blog \
-f $book \
-f $user \
-W
