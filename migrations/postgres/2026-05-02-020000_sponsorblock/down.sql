ALTER TABLE podcast_settings DROP COLUMN sponsorblock_categories;
ALTER TABLE podcast_settings DROP COLUMN sponsorblock_enabled;
ALTER TABLE settings DROP COLUMN sponsorblock_categories;
ALTER TABLE settings DROP COLUMN sponsorblock_enabled;
ALTER TABLE podcast_episodes DROP COLUMN sponsorblock_fetched_at;
ALTER TABLE podcast_episode_chapters DROP COLUMN chapter_type;
