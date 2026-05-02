ALTER TABLE podcast_episode_chapters
    ADD COLUMN chapter_type TEXT NOT NULL DEFAULT 'content';

ALTER TABLE podcast_episodes
    ADD COLUMN sponsorblock_fetched_at TIMESTAMP NULL;

ALTER TABLE settings
    ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE settings
    ADD COLUMN sponsorblock_categories TEXT NOT NULL
        DEFAULT 'sponsor,selfpromo,interaction';

ALTER TABLE podcast_settings
    ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE podcast_settings
    ADD COLUMN sponsorblock_categories TEXT NOT NULL
        DEFAULT 'sponsor,selfpromo,interaction';
