-- This file should undo anything in `up.sql`
ALTER TABLE podcast_episodes DROP COLUMN episode_numbering_processed;
DROP TABLE IF EXISTS podcast_settings;