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
    podcast_id INTEGER NOT NULL,
    created Datetime NOT NULL,
    deleted Datetime,
    UNIQUE (username, device, podcast_id)
);


CREATE TABLE subscription_devices(
    subscription_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    foreign key(subscription_id) references subscriptions(id),
    foreign key(device_id) references devices(id),
    PRIMARY KEY (subscription_id, device_id)
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
    total INTEGER
);