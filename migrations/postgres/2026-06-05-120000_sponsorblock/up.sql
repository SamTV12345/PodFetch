ALTER TABLE podcast_episodes ADD COLUMN youtube_video_id TEXT;
ALTER TABLE settings ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE episode_sponsor_segments (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    uuid TEXT NOT NULL,
    category TEXT NOT NULL,
    action_type TEXT NOT NULL,
    start_ms BIGINT NOT NULL,
    end_ms BIGINT NOT NULL,
    votes INTEGER NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT false,
    duration_mismatch BOOLEAN NOT NULL DEFAULT false,
    fetched_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX uq_episode_sponsor_segments_episode_uuid
    ON episode_sponsor_segments (episode_id, uuid);

CREATE TABLE sponsorblock_user_settings (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    skip_sponsor BOOLEAN NOT NULL DEFAULT true,
    skip_selfpromo BOOLEAN NOT NULL DEFAULT true,
    skip_interaction BOOLEAN NOT NULL DEFAULT false,
    skip_intro BOOLEAN NOT NULL DEFAULT false,
    skip_outro BOOLEAN NOT NULL DEFAULT false,
    skip_preview BOOLEAN NOT NULL DEFAULT false,
    skip_filler BOOLEAN NOT NULL DEFAULT false,
    skip_music_offtopic BOOLEAN NOT NULL DEFAULT false
);
