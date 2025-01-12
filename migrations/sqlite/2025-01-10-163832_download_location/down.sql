-- This file should undo anything in `up.sql`
ALTER TABLE podcast_episodes ADD COLUMN local_image_url TEXT;
ALTER TABLE podcast_episodes ADD COLUMN local_url TEXT;

ALTER TABLE podcasts DROP COLUMN download_location;
ALTER TABLE podcast_episodes DROP COLUMN download_location;