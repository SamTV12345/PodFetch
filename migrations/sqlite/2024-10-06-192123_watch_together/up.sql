-- Your SQL goes here

CREATE TABLE watch_together (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL,
    admin TEXT NOT NULL, --- token of the user who created the room
    room_name TEXT NOT NULL
);


CREATE TABLE watch_together_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL,
    user TEXT NOT NULL, --- token of the user who joined the room
    status TEXT NOT NULL,
    UNIQUE(room_id, user),
    FOREIGN KEY (room_id) REFERENCES watch_together(room_id)
);

