CREATE TABLE blogs (
    blog_id serial PRIMARY KEY NOT NULL,
    author_id INT,
    title TEXT,
    body TEXT,
    images TEXT,
    metadata TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE blog (
    uid serial PRIMARY KEY NOT NULL,
    blog_id INT,
    parent_id INT,
    title TEXT,
    body TEXT,
    images TEXT,
    identity smallINT,
    metadata TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
