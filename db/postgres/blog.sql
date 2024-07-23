CREATE TABLE blogs (
    blog_id serial PRIMARY KEY NOT NULL,
    user_id INT,
    title TEXT,
    body TEXT,
    images TEXT,
    tags TEXT,
    theme SMALLINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);
CREATE TABLE blog (
    uid serial PRIMARY KEY NOT NULL,
    blog_id INT,
    user_id INT,
    parent_id INT,
    title TEXT,
    body TEXT,
    tags TEXT,
    images TEXT,
    theme SMALLINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);
