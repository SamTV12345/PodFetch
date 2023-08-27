-- Your SQL goes here
ALTER TABLE podcast_episodes ADD COLUMN file_episode_path TEXT;
ALTER TABLE podcast_episodes ADD COLUMN file_image_path TEXT;

UPDATE podcast_episodes SET file_episode_path = local_url;
UPDATE podcast_episodes SET file_image_path = local_image_url;