CREATE TABLE users (
    uid serial PRIMARY KEY NOT NULL,
    fname VARCHAR(25) NOT NULL,
    lname TEXT,
    images TEXT,
    email VARCHAR(50) UNIQUE NOT NULL,
    phone VARCHAR(15),
    password VARCHAR(260),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

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