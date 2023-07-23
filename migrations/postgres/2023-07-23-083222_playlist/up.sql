-- Your SQL goes here
CREATE TABLE playlists (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);


CREATE TABLE playlist_items (
    playlist_id TEXT NOT NULL,
    episode INTEGER NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id),
    FOREIGN KEY (episode) REFERENCES episodes(id),
    PRIMARY KEY (playlist_id, episode)
);