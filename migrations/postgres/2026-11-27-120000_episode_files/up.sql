-- File state of downloaded episodes moves out of the catalog's
-- podcast_episodes into its own table (owned by the media library).
-- Only episodes with any file state get a row; no row means "not
-- downloaded and not deleted by the user".
CREATE TABLE episode_files (
    episode_id        TEXT PRIMARY KEY NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    download_time     TIMESTAMP NULL,
    file_episode_path TEXT NULL,
    file_image_path   TEXT NULL,
    download_location TEXT NULL,
    deleted           BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO episode_files
    (episode_id, download_time, file_episode_path, file_image_path, download_location, deleted)
SELECT id, download_time, file_episode_path, file_image_path, download_location, deleted
FROM podcast_episodes
WHERE download_time IS NOT NULL
   OR file_episode_path IS NOT NULL
   OR file_image_path IS NOT NULL
   OR download_location IS NOT NULL
   OR deleted;

CREATE INDEX idx_episode_files_downloaded ON episode_files (download_location);

ALTER TABLE podcast_episodes
    DROP COLUMN download_time,
    DROP COLUMN file_episode_path,
    DROP COLUMN file_image_path,
    DROP COLUMN download_location,
    DROP COLUMN deleted;
