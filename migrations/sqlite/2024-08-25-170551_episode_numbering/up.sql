-- Your SQL goes here

ALTER TABLE podcast_episodes ADD COLUMN episode_numbering_processed BOOLEAN NOT NULL DEFAULT FALSE;
CREATE TABLE podcast_settings (
    podcast_id INTEGER PRIMARY KEY NOT NULL,
    episode_numbering BOOLEAN NOT NULL DEFAULT FALSE,
    auto_download BOOLEAN NOT NULL DEFAULT FALSE,
    auto_update BOOLEAN NOT NULL DEFAULT TRUE,
    auto_cleanup BOOLEAN NOT NULL DEFAULT FALSE,
    auto_cleanup_days INTEGER NOT NULL DEFAULT -1,
    replace_invalid_characters  BOOLEAN DEFAULT TRUE NOT NULL,
    use_existing_filename  BOOLEAN DEFAULT FALSE NOT NULL,
    replacement_strategy  TEXT CHECK(replacement_strategy IN
            ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash')) NOT NULL DEFAULT
                'replace-with-dash-and-underscore',
    episode_format TEXT NOT NULL DEFAULT '{}',
    podcast_format TEXT NOT NULL DEFAULT '{}',
    direct_paths BOOLEAN NOT NULL DEFAULT FALSE
);