-- Your SQL goes here
CREATE TABLE podcast_episode_chapters (
    id TEXT PRIMARY KEY NOT NULL ,
    episode_id INTEGER NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    href TEXT,
    image TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL ,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX (episode_id, start_time) ON podcast_episode_chapters;