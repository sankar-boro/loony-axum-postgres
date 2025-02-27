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