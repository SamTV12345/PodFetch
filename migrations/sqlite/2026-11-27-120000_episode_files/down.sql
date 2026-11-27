ALTER TABLE podcast_episodes ADD COLUMN download_time DATETIME NULL;
ALTER TABLE podcast_episodes ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE podcast_episodes ADD COLUMN file_episode_path TEXT;
ALTER TABLE podcast_episodes ADD COLUMN file_image_path TEXT;
ALTER TABLE podcast_episodes ADD COLUMN download_location TEXT;

UPDATE podcast_episodes
SET download_time     = (SELECT f.download_time FROM episode_files f WHERE f.episode_id = podcast_episodes.id),
    deleted           = COALESCE((SELECT f.deleted FROM episode_files f WHERE f.episode_id = podcast_episodes.id), FALSE),
    file_episode_path = (SELECT f.file_episode_path FROM episode_files f WHERE f.episode_id = podcast_episodes.id),
    file_image_path   = (SELECT f.file_image_path FROM episode_files f WHERE f.episode_id = podcast_episodes.id),
    download_location = (SELECT f.download_location FROM episode_files f WHERE f.episode_id = podcast_episodes.id);

DROP TABLE episode_files;
