-- Your SQL goes here

CREATE TABLE watch_togethers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    room_id TEXT NOT NULL,
    admin TEXT NOT NULL, --- token of the user who created the room
    room_name TEXT NOT NULL
);


CREATE TABLE watch_together_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    room_id TEXT NOT NULL,
    user TEXT NOT NULL, --- token of the user who joined the room
    username TEXT,
    status TEXT NOT NULL,
    UNIQUE(room_id, user),
    FOREIGN KEY (room_id) REFERENCES watch_togethers(room_id)
);

ALTER TABLE settings ADD COLUMN jwt_key BLOB NULL;