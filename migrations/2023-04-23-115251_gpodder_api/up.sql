-- Your SQL goes here
CREATE TABLE devices(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    deviceid VARCHAR(255) NOT NULL,
    kind TEXT CHECK(kind IN ('desktop', 'laptop', 'server','mobile', 'Other')) NOT NULL,
    name VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username)
);

CREATE TABLE sessions(
    username VARCHAR(255) NOT NULL,
    session_id VARCHAR(255) NOT NULL,
    expires DATETIME NOT NULL,
    PRIMARY KEY (username, session_id)
);

CREATE TABLE subscriptions(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast TEXT NOT NULL,
    created Datetime NOT NULL,
    deleted Datetime,
    UNIQUE (username, device, podcast)
);

CREATE TABLE episodes(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username VARCHAR(255) NOT NULL,
    device VARCHAR(255) NOT NULL,
    podcast VARCHAR(255) NOT NULL,
    episode VARCHAR(255) NOT NULL,
    timestamp DATETIME NOT NULL,
    guid VARCHAR(255),
    action VARCHAR(255) NOT NULL,
    started INTEGER,
    position INTEGER,
    total INTEGER,
    UNIQUE (username, device, podcast, episode, timestamp)
);


CREATE table if not exists podcast_history_items2 (
                                                     id integer primary key not null,
                                                     podcast_id integer not null,
                                                     episode_id TEXT not null,
                                                     watched_time integer not null,
                                                     date DATETIME not null,
                                                     username text not null,
                                                     FOREIGN KEY (podcast_id) REFERENCES podcasts(id));

INSERT INTO podcast_history_items2 SELECT * FROM podcast_history_items;

DROP TABLE podcast_history_items;
ALTER TABLE podcast_history_items2 RENAME TO podcast_history_items;