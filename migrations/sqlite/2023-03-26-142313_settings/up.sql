-- Your SQL goes here
CREATE TABLE settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    auto_download BOOLEAN NOT NULL DEFAULT TRUE,
    auto_update BOOLEAN NOT NULL DEFAULT TRUE,
    auto_cleanup BOOLEAN NOT NULL DEFAULT FALSE,
    auto_cleanup_days INTEGER NOT NULL DEFAULT -1
);

ALTER TABLE podcast_episodes ADD COLUMN download_time DATETIME NULL;
ALTER TABLE podcasts ADD COLUMN active BOOLEAN NOT NULL DEFAULT TRUE;

UPDATE podcast_episodes SET download_time = datetime('now') WHERE status='D';