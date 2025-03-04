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

-- create search on title for books
ALTER TABLE books
ADD COLUMN books_search_vector tsvector;
UPDATE books
SET books_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX books_search_vector_idx ON books USING GIN (books_search_vector);

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

-- Create and index to select chapters and sections
CREATE INDEX idx_book_page_id ON book(doc_id, page_id);

-- create search on title and tags for book
ALTER TABLE book
ADD COLUMN book_search_vector tsvector;
UPDATE book
SET book_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX book_search_vector_idx ON book USING GIN (book_search_vector);