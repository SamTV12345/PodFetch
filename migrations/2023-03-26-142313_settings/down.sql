-- This file should undo anything in `up.sql`
DROP TABLE settings;
ALTER TABLE podcast_episodes DROP COLUMN download_time;
ALTER TABLE podcasts DROP COLUMN active;