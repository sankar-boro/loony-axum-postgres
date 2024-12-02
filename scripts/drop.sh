#!/bin/bash

PWD=$(pwd)
dropfile=$PWD/db/postgres/drop.sql

psql -h localhost -U sankar -d sankar -f $dropfile -W

