-- This file should undo anything in `up.sql`
ALTER TABLE podcasts DROP COLUMN episode_numbering;
DROP TABLE podcast_settings;