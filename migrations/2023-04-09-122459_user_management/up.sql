-- Your SQL goes here
CREATE TABLE if not exists users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username VARCHAR(255) NOT NULL,
    role TEXT CHECK(role IN ('admin', 'uploader', 'user')) NOT NULL,
    password VARCHAR(255) NULL,
    explicit_consent BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (username)
);

CREATE TABLE invites (
    id VARCHAR(255) NOT NULL PRIMARY KEY,
    role TEXT CHECK(role IN ('admin', 'uploader', 'user')) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    accepted_at DATETIME NULL,
    explicit_consent BOOLEAN NOT NULL DEFAULT 0,
    expires_at DATETIME NOT NULL
);

ALTER TABLE podcasts DROP COLUMN favored;

CREATE TABLE favorites (
    username TEXT NOT NULL,
    podcast_id INTEGER NOT NULL,
    favored BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (username, podcast_id),
    FOREIGN KEY (username) REFERENCES users (username) ON DELETE CASCADE,
    FOREIGN KEY (podcast_id) REFERENCES podcasts (id) ON DELETE CASCADE
);


ALTER TABLE podcast_history_items ADD COLUMN username TEXT NOT NULL DEFAULT 'user123';