CREATE KEYSPACE sankar WITH replication = {
    'class': 'SimpleStrategy',
    'replication_factor': 1
};

-- Create blog tables

CREATE TABLE blogs (
    blogId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (blogId, createdAt)
) WITH CLUSTERING ORDER BY (createdAt DESC);
CREATE TABLE blog (
    blogId timeuuid,
    uniqueId timeuuid,
    parentId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    identity smallINT,
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (blogId, uniqueId)
);
CREATE TABLE userblogs (
    blogId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (authorId, blogId)
) WITH CLUSTERING ORDER BY (blogId DESC);;
CREATE TABLE categoryblogs (
    category TEXT.
    blogId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (category, blogId)
) WITH CLUSTERING ORDER BY (blogId DESC);

-- Create book tables;

CREATE TABLE books (
    bookId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (bookId, createdAt)
) WITH CLUSTERING ORDER BY (createdAt DESC);
CREATE TABLE book (
    bookId timeuuid,
    pageId timeuuid,
    uniqueId timeuuid,
    parentId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    identity smallINT,
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY ((bookId, pageId), uniqueId)
);
CREATE TABLE userbooks (
    bookId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (authorId, bookId)
) WITH CLUSTERING ORDER BY (bookId DESC);
CREATE TABLE categorybooks (
    category TEXT.
    bookId timeuuid,
    authorId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (category, bookId)
) WITH CLUSTERING ORDER BY (bookId DESC);
CREATE TABLE book_title (
    bookId timeuuid,
    parentId timeuuid,
    uniqueId timeuuid,
    title TEXT.
    identity smallINT,
    PRIMARY KEY (bookId, uniqueId)
) WITH CLUSTERING ORDER BY (uniqueId DESC);

-- Categories
CREATE TABLE usercategories (
    authorId INT,
    category varchar,
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (authorId)
);
CREATE TABLE allcategories (
    category varchar,
    division varchar,
    PRIMARY KEY (category, division)
);