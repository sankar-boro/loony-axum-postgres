#!/usr/bin/env bash

# Exit on error, unset variable, or failed pipe
# set -euo pipefail
set +e

# === CONFIGURATION ===
set -o allexport
source .env.postgres
set +o allexport

# Run all SQL commands through psql
PGPASSWORD="$DB_SUPERUSER_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_SUPERUSER" -v ON_ERROR_STOP=1 <<-SQL

DROP DATABASE IF EXISTS $NEW_DB_NAME;

SQL

echo "✅ Databases dropped successfully!"
