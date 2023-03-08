-- Your SQL goes here
CREATE TABLE notifications (
    id INT NOT NULL,
    type_of_message TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL,
    PRIMARY KEY (id)
)