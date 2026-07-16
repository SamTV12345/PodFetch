-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS transcription_jobs;
DROP INDEX IF EXISTS idx_segments_fts;
DROP TABLE IF EXISTS podcast_episode_transcript_segments;
DROP TABLE IF EXISTS podcast_episode_transcripts;

ALTER TABLE podcast_settings DROP COLUMN auto_transcribe;
