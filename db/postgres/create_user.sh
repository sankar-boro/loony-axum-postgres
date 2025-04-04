#!/usr/bin/env bash

# Variables (edit these as needed)
USERNAME="test"
PASSWORD="password"
DATABASE_NAME="test"

# Run PostgreSQL commands
sudo -u postgres psql <<EOF
CREATE USER $USERNAME WITH PASSWORD '$PASSWORD';
CREATE DATABASE $DATABASE_NAME;
GRANT ALL PRIVILEGES ON DATABASE $DATABASE_NAME TO $USERNAME;
ALTER USER $USERNAME WITH SUPERUSER;
EOF
