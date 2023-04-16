-- Your SQL goes here
ALTER TABLE podcasts RENAME COLUMN directory TO directory_id;
ALTER TABLE podcasts ADD COLUMN directory_name VARCHAR(255) NOT NULL DEFAULT '';

PRAGMA foreign_keys=off;


ALTER TABLE favorites RENAME TO _favorites_old;

CREATE TABLE favorites (
                           username TEXT NOT NULL,
                           podcast_id INTEGER NOT NULL,
                           favored BOOLEAN NOT NULL DEFAULT 0,
                           PRIMARY KEY (username, podcast_id),
                           FOREIGN KEY (podcast_id) REFERENCES podcasts (id) ON DELETE CASCADE
);

INSERT INTO favorites SELECT * FROM _favorites_old;

DROP TABLE _favorites_old;
PRAGMA foreign_keys=on;