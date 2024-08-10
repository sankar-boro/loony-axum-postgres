DROP TABLE tags;
DROP TABLE document_tags;
DROP TABLE user_tags;

CREATE TABLE tags (
    uid serial PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL
);

CREATE TABLE book_tags (
    uid serial PRIMARY KEY,
    tag_id INT NOT NULL,
    book_id INT NOT NULL
);

CREATE TABLE blog_tags (
    uid serial PRIMARY KEY,
    tag_id INT NOT NULL,
    blog_id INT NOT NULL
);

CREATE TABLE user_tags (
    uid serial PRIMARY KEY,
    tag_id INT NOT NULL,
    user_id INT NOT NULL
);