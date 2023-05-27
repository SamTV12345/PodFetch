-- Your SQL goes here
CREATE INDEX podcast_episode_url_index ON podcast_episodes (url);
--- N->Not download D->Downloaded P->Pending
ALTER TABLE podcast_episodes ADD COLUMN status CHAR(1) NOT NULL DEFAULT 'N';