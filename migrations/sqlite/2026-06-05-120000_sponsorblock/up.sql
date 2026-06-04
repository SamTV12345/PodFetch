ALTER TABLE podcast_episodes ADD COLUMN youtube_video_id TEXT;
ALTER TABLE settings ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT 1;

CREATE TABLE episode_sponsor_segments (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    uuid TEXT NOT NULL,
    category TEXT NOT NULL,
    action_type TEXT NOT NULL,
    start_ms BIGINT NOT NULL,
    end_ms BIGINT NOT NULL,
    votes INTEGER NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT 0,
    duration_mismatch BOOLEAN NOT NULL DEFAULT 0,
    fetched_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX uq_episode_sponsor_segments_episode_uuid
    ON episode_sponsor_segments (episode_id, uuid);

CREATE TABLE sponsorblock_user_settings (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    skip_sponsor BOOLEAN NOT NULL DEFAULT 1,
    skip_selfpromo BOOLEAN NOT NULL DEFAULT 1,
    skip_interaction BOOLEAN NOT NULL DEFAULT 0,
    skip_intro BOOLEAN NOT NULL DEFAULT 0,
    skip_outro BOOLEAN NOT NULL DEFAULT 0,
    skip_preview BOOLEAN NOT NULL DEFAULT 0,
    skip_filler BOOLEAN NOT NULL DEFAULT 0,
    skip_music_offtopic BOOLEAN NOT NULL DEFAULT 0
);
