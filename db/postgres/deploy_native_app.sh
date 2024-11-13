#!/bin/bash

PWD=$(pwd)

credentials=$PWD/db/postgres/credentials.sql

psql -h localhost -U sankar -d sankar \
-f $credentials \
-W
