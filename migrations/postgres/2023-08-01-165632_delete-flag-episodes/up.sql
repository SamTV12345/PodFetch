-- Your SQL goes here
ALTER TABLE podcast_episodes ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE;