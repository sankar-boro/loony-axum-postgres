#!/bin/bash

# =====================================================================
# PostgreSQL Database Setup Script
# Modular version by ChatGPT
# =====================================================================

set +e  # Disable exit on error

# ---------------------------------------------------------------------
# Load Environment Variables
# ---------------------------------------------------------------------
load_env() {
  set -o allexport
  source .env.postgres
  set +o allexport
}

# ---------------------------------------------------------------------
# Create a Role if it doesn't exist
# ---------------------------------------------------------------------
create_role() {
  echo "=== Checking role '$NEW_DB_OWNER' ==="

  PGPASSWORD="$DB_SUPERUSER_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" \
    -U "$DB_SUPERUSER" -v ON_ERROR_STOP=1 <<-SQL
  DO \$\$
  BEGIN
      IF NOT EXISTS (
          SELECT FROM pg_roles WHERE rolname = '$NEW_DB_OWNER'
      ) THEN
          CREATE ROLE $NEW_DB_OWNER LOGIN PASSWORD '$NEW_DB_PASSWORD';
          RAISE NOTICE 'Role "$NEW_DB_OWNER" created.';
      ELSE
          RAISE NOTICE 'Role "$NEW_DB_OWNER" already exists.';
      END IF;
  END
  \$\$;
SQL
}

# ---------------------------------------------------------------------
# Create the Database if it doesn't exist
# ---------------------------------------------------------------------
create_database() {
  echo "=== Checking database '$NEW_DB_NAME' ==="

  if PGPASSWORD="$DB_SUPERUSER_PASSWORD" psql -U "$DB_SUPERUSER" \
      -h "$DB_HOST" -p "$DB_PORT" -lqt | cut -d \| -f 1 | grep -qw "$NEW_DB_NAME"; then
    echo "Database '$NEW_DB_NAME' already exists ✅"
  else
    echo "Creating database '$NEW_DB_NAME'..."
    PGPASSWORD="$DB_SUPERUSER_PASSWORD" createdb -U "$DB_SUPERUSER" \
      -h "$DB_HOST" -p "$DB_PORT" "$NEW_DB_NAME"
    echo "Database '$NEW_DB_NAME' created successfully 🎉"
  fi
}

# ---------------------------------------------------------------------
# Configure Database: ownership, schema, privileges
# ---------------------------------------------------------------------
configure_database() {
  echo "=== Configuring database '$NEW_DB_NAME' ==="

  PGPASSWORD="$DB_SUPERUSER_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" \
    -U "$DB_SUPERUSER" -v ON_ERROR_STOP=1 <<-SQL

  -- Change ownership
  ALTER DATABASE $NEW_DB_NAME OWNER TO $NEW_DB_OWNER;
    \echo 'OWNER of DATABASE "$NEW_DB_NAME" has been changed TO "$NEW_DB_OWNER" from postgres';

  -- Revoke public access
  REVOKE ALL ON DATABASE $NEW_DB_NAME FROM PUBLIC;

  \connect $NEW_DB_NAME

  DO \$\$
  BEGIN
      IF NOT EXISTS (
          SELECT FROM information_schema.schemata WHERE schema_name = '$NEW_DB_SCHEMA'
      ) THEN
          CREATE SCHEMA $NEW_DB_SCHEMA AUTHORIZATION $NEW_DB_OWNER;
          RAISE NOTICE 'Schema "$NEW_DB_SCHEMA" created.';
      ELSE
          RAISE NOTICE 'Schema "$NEW_DB_SCHEMA" already exists.';
      END IF;
  END
  \$\$;

  -- Revoke access from public schema
  REVOKE ALL ON SCHEMA public FROM PUBLIC;

  -- Grant privileges
  GRANT USAGE, CREATE ON SCHEMA $NEW_DB_SCHEMA TO $NEW_DB_OWNER;
  GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA $NEW_DB_SCHEMA TO $NEW_DB_OWNER;

  -- Default privileges for future tables
  ALTER DEFAULT PRIVILEGES IN SCHEMA $NEW_DB_SCHEMA
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO $NEW_DB_OWNER;

  -- Set search path
  ALTER ROLE $NEW_DB_OWNER SET search_path = $NEW_DB_SCHEMA;
SQL
}

# ---------------------------------------------------------------------
# Create initial tables (example: users)
# ---------------------------------------------------------------------
create_tables() {
  echo "=== Creating tables in schema '$NEW_DB_SCHEMA' ==="

  PGPASSWORD="$NEW_DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" \
    -U "$NEW_DB_OWNER" -d "$NEW_DB_NAME" -v ON_ERROR_STOP=1 <<-SQL
    
    \connect $NEW_DB_NAME;

  CREATE TABLE users (
      uid serial PRIMARY KEY NOT NULL,
      fname VARCHAR(25) NOT NULL,
      lname TEXT,
      images TEXT,
      email VARCHAR(50) UNIQUE NOT NULL,
      phone VARCHAR(15),
      password VARCHAR(260),
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      deleted_at TIMESTAMP WITH TIME ZONE
  );

  CREATE TABLE blogs (
      uid serial PRIMARY KEY,
      user_id INT NOT NULL,
      title VARCHAR(255) NOT NULL,
      content TEXT,
      images TEXT,
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      deleted_at TIMESTAMP WITH TIME ZONE NULL
  );
  CREATE TABLE blog (
      uid serial PRIMARY KEY,
      doc_id INT,
      user_id INT NOT NULL,
      parent_id INT,
      title VARCHAR(255) NOT NULL,
      content TEXT,
      images TEXT,
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      deleted_at TIMESTAMP WITH TIME ZONE NULL
  );

  CREATE TABLE books (
      uid serial PRIMARY KEY,
      user_id INT NOT NULL,
      title VARCHAR(255) NOT NULL,
      content TEXT,
      images TEXT,
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      deleted_at TIMESTAMP WITH TIME ZONE NULL
  );
  CREATE TABLE book (
      uid serial PRIMARY KEY,
      doc_id INT,
      user_id INT NOT NULL,
      page_id INT, -- page_id is required to show all nodes of a page
      parent_id INT,
      title VARCHAR(255) NOT NULL,
      content TEXT,
      images TEXT,
      identity SMALLINT,
      created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
      deleted_at TIMESTAMP WITH TIME ZONE NULL
  );

  CREATE TABLE book_tags (
      uid serial PRIMARY KEY,
      doc_id INT NOT NULL,
      user_id INT NOT NULL,
      tag VARCHAR(50),
      score INT NOT NULL
  );
  CREATE TABLE blog_tags (
      uid serial PRIMARY KEY,
      doc_id INT NOT NULL,
      user_id INT NOT NULL,
      tag VARCHAR(50),
      score INT NOT NULL
  );
SQL
}

# ---------------------------------------------------------------------
# Main Setup Function
# ---------------------------------------------------------------------
main() {
  echo "=== Starting PostgreSQL setup ==="
  load_env
  create_role
  create_database
  configure_database
  create_tables
  echo "✅ Database '$NEW_DB_NAME' configured successfully!"
}

# ---------------------------------------------------------------------
# Execute Main
# ---------------------------------------------------------------------
main
