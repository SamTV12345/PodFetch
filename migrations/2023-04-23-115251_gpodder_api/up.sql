-- Your SQL goes here
CREATE TABLE devices(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    deviceid VARCHAR(255) NOT NULL,
    kind TEXT CHECK(kind IN ('desktop', 'laptop', 'server', 'other')) NOT NULL,
    name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username)
);