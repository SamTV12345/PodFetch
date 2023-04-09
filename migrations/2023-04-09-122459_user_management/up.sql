-- Your SQL goes here
CREATE TABLE if not exists users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) NOT NULL,
    role TEXT CHECK(role IN ('admin', 'uploader', 'user')) NOT NULL,
    password VARCHAR(255) NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE invites (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    role TEXT CHECK(role IN ('admin', 'uploader', 'user')) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accepted_at DATETIME NULL,
    expires_at DATETIME NOT NULL
)