-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS transcript_segments_au;
DROP TRIGGER IF EXISTS transcript_segments_ad;
DROP TRIGGER IF EXISTS transcript_segments_ai;
DROP TABLE IF EXISTS transcript_segments_fts;

DROP TABLE IF EXISTS transcription_jobs;
DROP TABLE IF EXISTS podcast_episode_transcript_segments;
DROP TABLE IF EXISTS podcast_episode_transcripts;

ALTER TABLE podcast_settings DROP COLUMN auto_transcribe;
