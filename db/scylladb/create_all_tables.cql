CREATE KEYSPACE sankar WITH replication = {
    'class': 'SimpleStrategy',
    'replication_factor': 1
};

-- Create blog tables

CREATE TABLE blogs (
    blogId timeuuid,
    userId INT,
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
    userId INT,
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
    userId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId, blogId)
) WITH CLUSTERING ORDER BY (blogId DESC);;
CREATE TABLE categoryblogs (
    category TEXT.
    blogId timeuuid,
    userId INT,
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
    userId INT,
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
    userId INT,
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
    userId INT,
    title varchar,
    body TEXT.
    url TEXT.
    metadata TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId, bookId)
) WITH CLUSTERING ORDER BY (bookId DESC);
CREATE TABLE categorybooks (
    category TEXT.
    bookId timeuuid,
    userId INT,
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
    userId INT,
    category varchar,
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId)
);
CREATE TABLE allcategories (
    category varchar,
    division varchar,
    PRIMARY KEY (category, division)
);