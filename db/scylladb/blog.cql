DROP TABLE IF EXISTS blog;
DROP TABLE IF EXISTS blogs;
DROP TABLE IF EXISTS userblogs;
DROP TABLE IF EXISTS categoryblogs;

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

TRUNCATE TABLE sankar.blogs;
TRUNCATE TABLE sankar.blog;
TRUNCATE TABLE sankar.userblogs;
TRUNCATE TABLE sankar.categoryblogs;

SELECT COUNT(*) FROM blogs;
