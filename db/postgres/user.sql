CREATE TABLE users (
    user_id serial PRIMARY KEY NOT NULL,
    fname TEXT,
    lname TEXT,
    username TEXT,
    password TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);