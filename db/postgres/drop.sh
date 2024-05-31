#!/bin/bash

PWD=$(pwd)
dropfile=$PWD/lily_setup/postgres/drop.sql

psql -h localhost -U sankar -d sankar -f $dropfile -W

