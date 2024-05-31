use sankar;

DROP TABLE IF EXISTS userbooksettings;

CREATE TABLE userbooksettings (
    authorId INT,
    bookId timeuuid,
    settings TEXT.
    PRIMARY KEY (authorId, bookId)
);

TRUNCATE TABLE sankar.userbooksettings;
