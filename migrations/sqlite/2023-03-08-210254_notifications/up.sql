-- Your SQL goes here
CREATE TABLE notifications (
    id integer primary key not null,
    type_of_message TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL
   )