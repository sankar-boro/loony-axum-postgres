use sankar;

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS userCredentials;

CREATE TABLE users (
    userId timeuuid,
    fname varchar,
    lname varchar,
    email varchar,
    password blob,
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (userId)
);

CREATE TABLE userCredentials (
    userId timeuuid,
    fname varchar,
    lname varchar,
    email varchar,
    password blob,
    createdAt timeuuid,
    updatedAt timeuuid,
    PRIMARY KEY (email)
);

TRUNCATE TABLE sankar.users;
TRUNCATE TABLE sankar.userCredentials;

