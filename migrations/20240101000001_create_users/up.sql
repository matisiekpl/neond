CREATE TABLE users (
    id            UUID        PRIMARY KEY,
    name          VARCHAR     NOT NULL,
    email         VARCHAR     NOT NULL UNIQUE,
    password_hash VARCHAR     NOT NULL
);
