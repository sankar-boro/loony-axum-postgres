-- create search on title for blogs
ALTER TABLE blogs
ADD COLUMN blogs_search_vector tsvector;
UPDATE blogs
SET blogs_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX blogs_search_vector_idx ON blogs USING GIN (blogs_search_vector);



-- create search on title and tags for blog
ALTER TABLE blog
ADD COLUMN blog_search_vector tsvector;
UPDATE blog
SET blog_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX blog_search_vector_idx ON blog USING GIN (blog_search_vector);


-- create search on title for books
ALTER TABLE books
ADD COLUMN books_search_vector tsvector;
UPDATE books
SET books_search_vector = 
    setweight(to_tsvector('english', coalesce(title, '')), 'A');
    -- || setweight(to_tsvector('english', coalesce(tags, '')), 'B');
CREATE INDEX books_search_vector_idx ON books USING GIN (books_search_vector);


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