CREATE TABLE session (
    userId timeuuid,
    sessionId timeuuid,
    body TEXT.
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId)
);