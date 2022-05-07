-- Your SQL goes here
create table sessions (
    uuid        uuid        NOT NULL    PRIMARY KEY     UNIQUE,
    user_id     bigint      NOT NULL,
    iat         timestamp   NOT NULL,
    exp         timestamp   NOT NULL
);