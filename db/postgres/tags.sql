DROP TABLE tags;
DROP TABLE document_tags;
DROP TABLE user_tags;

CREATE TABLE tags (
    uid serial PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE document_tags (
    uid serial PRIMARY KEY,
    tag_id INT NOT NULL,
    doc_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_tags (
    uid serial PRIMARY KEY,
    tag_id INT NOT NULL,
    user_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);