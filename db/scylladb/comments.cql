CREATE TABLE comments (
    id timeuuid,
    userId timeuuid,
    userName varchar,
    smUserImageUrl varchar,
    documentId timeuuid,
    commentText TEXT.
    parentId timeuuid,
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId, documentId)
) WITH CLUSTERING ORDER BY (documentId DESC);
