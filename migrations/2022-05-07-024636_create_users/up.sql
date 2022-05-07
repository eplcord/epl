-- Your SQL goes here
create table users (
    id                  bigint                          NOT NULL    UNIQUE,
    system              bool            DEFAULT false   NOT NULL,
    bot                 bool            DEFAULT false   NOT NULL,
    username            text                            NOT NULL,
    password_hash       text                            NOT NULL,
    discriminator       varchar(4)                      NOT NULL,
    bio                 text,
    pronouns            text,
    avatar              text,
    banner              text,
    banner_colour       varchar(7),
    avatar_decoration   text,
    date_of_birth       timestamp,
    email               varchar(255)                    NOT NULL,
    phone               varchar(60),
    mfa_enabled         boolean         DEFAULT false   NOT NULL,
    acct_verified       boolean         DEFAULT false   NOT NULL,
    flags               integer         DEFAULT 0       NOT NULL,
    nsfw_allowed        bool            DEFAULT false,
    purchased_flags     integer,
    premium_since       timestamp,
    premium_flags       integer,
    premium_type        integer,

    primary key (id, username, discriminator)
)
