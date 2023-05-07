-- Your SQL goes here
CREATE TABLE filters(
    username TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    ascending BOOLEAN DEFAULT FALSE NOT NULL,
    filter TEXT CHECK(filter IN ('PublishedDate', 'Title'))
)