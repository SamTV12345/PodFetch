-- Your SQL goes here
ALTER TABLE podcast_episodes DROP COLUMN local_image_url;
ALTER TABLE podcast_episodes DROP COLUMN local_url;


ALTER TABLE podcasts ADD COLUMN download_location TEXT;
ALTER TABLE podcast_episodes ADD COLUMN download_location TEXT;

UPDATE podcasts SET download_location = 'Local';
UPDATE podcast_episodes SET download_location = 'Local' WHERE status = 'D';

ALTER TABLE podcast_episodes DROP COLUMN status;