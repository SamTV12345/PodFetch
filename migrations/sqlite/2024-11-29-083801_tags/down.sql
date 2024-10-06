-- This file should undo anything in `up.sql`
DROP TABLE tags;
DROP TABLE tags_podcasts;

DROP INDEX idx_tags_name;
DROP INDEX idx_tags_username;
DROP INDEX idx_devices;
DROP INDEX idx_episodes_podcast;
DROP INDEX idx_episodes_episode;
DROP INDEX idx_podcast_episodes;
DROP INDEX idx_podcast_episodes_url;
DROP INDEX idx_podcasts_name;
DROP INDEX idx_podcasts_rssfeed;
DROP INDEX idx_subscriptions;
DROP INDEX idx_subscriptions_device;