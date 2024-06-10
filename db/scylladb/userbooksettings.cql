use sankar;

DROP TABLE IF EXISTS userbooksettings;

CREATE TABLE userbooksettings (
    userId INT,
    bookId timeuuid,
    settings TEXT.
    PRIMARY KEY (userId, bookId)
);

TRUNCATE TABLE sankar.userbooksettings;
