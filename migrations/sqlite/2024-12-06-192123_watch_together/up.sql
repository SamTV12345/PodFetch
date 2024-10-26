-- Your SQL goes here

CREATE TABLE watch_togethers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    room_id TEXT NOT NULL,
    room_name TEXT NOT NULL
);


CREATE TABLE watch_together_users (
    subject TEXT PRIMARY KEY NOT NULL,
    name TEXT NULL
);


CREATE TABLE watch_together_users_to_room_mapping (
    room_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    status TEXT NOT NULL,
    FOREIGN KEY (room_id) REFERENCES watch_togethers(id),
    FOREIGN KEY (subject) REFERENCES watch_together_users(name),
    PRIMARY KEY (room_id, subject)
);

ALTER TABLE settings ADD COLUMN jwt_key BLOB NULL;