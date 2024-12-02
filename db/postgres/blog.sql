DROP TABLE IF EXISTS blogs;
DROP TABLE IF EXISTS blog;

CREATE TABLE blogs (
    uid serial PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
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
    uid serial PRIMARY KEY,
    blog_id INT,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
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
