-- Your SQL goes here

CREATE TABLE tags (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    username TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL,
    color TEXT NOT NULL,
    UNIQUE (name, username)
);

CREATE TABLE tags_podcasts
(
    tag_id     TEXT NOT NULL,
    podcast_id INTEGER NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES tags (id),
    FOREIGN KEY (podcast_id) REFERENCES podcasts (id),
    PRIMARY KEY (tag_id, podcast_id)
);


-- INDEXES
CREATE INDEX idx_tags_name ON tags (name);
CREATE INDEX idx_tags_username ON tags (username);
CREATE INDEX idx_devices ON devices(name);
CREATE INDEX idx_episodes_podcast ON episodes(podcast);
CREATE INDEX idx_episodes_episode ON episodes(episode);
CREATE INDEX idx_podcast_episodes ON podcast_episodes(podcast_id);
CREATE INDEX idx_podcast_episodes_url ON podcast_episodes(url);
CREATE INDEX idx_podcasts_name ON podcasts(name);
CREATE INDEX idx_podcasts_rssfeed ON podcasts(rssfeed);
CREATE INDEX idx_subscriptions ON subscriptions(username);
CREATE INDEX idx_subscriptions_device ON subscriptions(device);