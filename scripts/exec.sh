#!/bin/bash

ARG1=$1
PWD=$(pwd)
execfile=$PWD/db/postgres/$ARG1

psql -h localhost -U sankar -d sankar -f $execfile -W

