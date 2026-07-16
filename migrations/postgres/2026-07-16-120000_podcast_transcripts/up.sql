-- Your SQL goes here
CREATE TABLE podcast_episode_transcripts (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    source TEXT NOT NULL, -- 'feed' | 'generated'
    original_url TEXT,
    file_path TEXT,
    mime_type TEXT NOT NULL,
    language TEXT,
    is_preferred BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'downloaded'|'parsed'|'failed'
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);
CREATE UNIQUE INDEX uq_transcripts_episode_url
    ON podcast_episode_transcripts (episode_id, original_url);
CREATE UNIQUE INDEX uq_transcripts_episode_generated
    ON podcast_episode_transcripts (episode_id) WHERE source = 'generated';
CREATE INDEX idx_transcripts_episode ON podcast_episode_transcripts (episode_id);

CREATE TABLE podcast_episode_transcript_segments (
    id TEXT PRIMARY KEY NOT NULL,
    transcript_id TEXT NOT NULL REFERENCES podcast_episode_transcripts(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    start_ms INTEGER,
    end_ms INTEGER,
    speaker TEXT,
    text TEXT NOT NULL
);
CREATE INDEX idx_segments_transcript ON podcast_episode_transcript_segments (transcript_id);

ALTER TABLE podcast_episode_transcript_segments
    ADD COLUMN text_search tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', text)) STORED;
CREATE INDEX idx_segments_fts ON podcast_episode_transcript_segments USING GIN (text_search);

CREATE TABLE transcription_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL UNIQUE REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'running'|'done'|'failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE podcast_settings ADD COLUMN auto_transcribe BOOLEAN NOT NULL DEFAULT FALSE;
