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
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
CREATE UNIQUE INDEX uq_transcripts_episode_url
    ON podcast_episode_transcripts (episode_id, original_url);
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

CREATE VIRTUAL TABLE transcript_segments_fts USING fts5(
    text,
    content='podcast_episode_transcript_segments',
    content_rowid='rowid'
);
CREATE TRIGGER transcript_segments_ai AFTER INSERT ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;
CREATE TRIGGER transcript_segments_ad AFTER DELETE ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text)
        VALUES ('delete', old.rowid, old.text);
END;
CREATE TRIGGER transcript_segments_au AFTER UPDATE ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text)
        VALUES ('delete', old.rowid, old.text);
    INSERT INTO transcript_segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;

CREATE TABLE transcription_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL UNIQUE REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'running'|'done'|'failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

ALTER TABLE podcast_settings ADD COLUMN auto_transcribe BOOLEAN NOT NULL DEFAULT FALSE;
