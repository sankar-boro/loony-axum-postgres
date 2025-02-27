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

-- create search on title for blogs
ALTER TABLE blogs
ADD COLUMN blogs_search_vector tsvector;
UPDATE blogs
SET blogs_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX blogs_search_vector_idx ON blogs USING GIN (blogs_search_vector);


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

-- create search on title and tags for blog
ALTER TABLE blog
ADD COLUMN blog_search_vector tsvector;
UPDATE blog
SET blog_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX blog_search_vector_idx ON blog USING GIN (blog_search_vector);
