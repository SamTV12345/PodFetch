CREATE TABLE listening_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast_episode_id TEXT NOT NULL,
    podcast_id INTEGER NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    podcast_episode_db_id INTEGER NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    delta_seconds INTEGER NOT NULL,
    start_position INTEGER NOT NULL,
    end_position INTEGER NOT NULL,
    listened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_listening_events_username_time
    ON listening_events (username, listened_at);

CREATE INDEX idx_listening_events_username_episode
    ON listening_events (username, podcast_episode_id);

CREATE INDEX idx_listening_events_username_podcast
    ON listening_events (username, podcast_id);
