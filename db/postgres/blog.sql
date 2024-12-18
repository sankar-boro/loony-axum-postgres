DROP TABLE IF EXISTS blogs;
DROP TABLE IF EXISTS blog;

ALTER TABLE blogs DROP COLUMN IF EXISTS blogs_search_vector;
ALTER TABLE blog DROP COLUMN IF EXISTS blog_search_vector;

CREATE TABLE blogs (
    uid serial PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
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
    blog_id INT,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    parent_id INT,
    title VARCHAR(255) NOT NULL,
    content TEXT,
    images TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);


-- Create and index to select chapters and sections
CREATE INDEX idx_blog_page_id ON blog(blog_id, page_id);

-- create search on title and tags for blog
ALTER TABLE blog
ADD COLUMN blog_search_vector tsvector;
UPDATE blog
SET blog_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX blog_search_vector_idx ON blog USING GIN (blog_search_vector);