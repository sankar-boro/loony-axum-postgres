CREATE TABLE book_tags (
    uid serial PRIMARY KEY,
    doc_id INT NOT NULL REFERENCES books(uid) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    tag VARCHAR(50),
    score INT NOT NULL
);

CREATE TABLE blog_tags (
    uid serial PRIMARY KEY,
    blog_id INT NOT NULL REFERENCES blogs(uid) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    tag VARCHAR(50),
    score INT NOT NULL
);