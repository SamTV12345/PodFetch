ALTER TABLE podcast_episodes
    ADD COLUMN download_time TIMESTAMP NULL,
    ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN file_episode_path TEXT,
    ADD COLUMN file_image_path TEXT,
    ADD COLUMN download_location TEXT;

UPDATE podcast_episodes e
SET download_time     = f.download_time,
    deleted           = f.deleted,
    file_episode_path = f.file_episode_path,
    file_image_path   = f.file_image_path,
    download_location = f.download_location
FROM episode_files f
WHERE f.episode_id = e.id;

DROP TABLE episode_files;
