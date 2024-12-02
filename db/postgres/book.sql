DROP TABLE IF EXISTS books;
DROP TABLE IF EXISTS book;

CREATE TABLE books (
    uid serial PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    title TEXT NOT NULL,
    body TEXT,
    images TEXT,
    theme SMALLINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);
CREATE TABLE book (
    uid serial PRIMARY KEY,
    book_id INT,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    page_id INT, -- page_id is required to show all nodes of a page
    parent_id INT,
    title TEXT,
    body TEXT,
    images TEXT,
    identity SMALLINT,
    theme SMALLINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE NULL
);

CREATE INDEX idx_book_page_id ON book(book_id, page_id);
