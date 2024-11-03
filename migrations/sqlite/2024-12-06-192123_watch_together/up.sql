-- Your SQL goes here

CREATE TABLE watch_togethers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    room_id TEXT NOT NULL,
    room_name TEXT NOT NULL
);


CREATE TABLE watch_together_users (
    --- uuid
    subject TEXT PRIMARY KEY NOT NULL,
    name TEXT NULL,
    user_id INTEGER NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);



CREATE TABLE watch_together_users_to_room_mappings (
    room_id INTEGER NOT NULL,
    subject TEXT NOT NULL,
    status TEXT NOT NULL,
    role TEXT NOT NULL,
    FOREIGN KEY (room_id) REFERENCES watch_togethers(id),
    FOREIGN KEY (subject) REFERENCES watch_together_users(subject),
    PRIMARY KEY (room_id, subject)
);

ALTER TABLE settings ADD COLUMN jwt_key BLOB NULL;