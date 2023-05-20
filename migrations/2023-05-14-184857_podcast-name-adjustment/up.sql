-- Your SQL goes here
ALTER TABLE settings ADD COLUMN replace_invalid_characters  BOOLEAN DEFAULT TRUE NOT NULL;
ALTER TABLE settings ADD COLUMN use_existing_filename  BOOLEAN DEFAULT FALSE NOT NULL;
ALTER TABLE settings ADD COLUMN replacement_strategy  TEXT CHECK(replacement_strategy IN
        ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash')) NOT NULL DEFAULT
            'replace-with-dash-and-underscore';
ALTER TABLE settings ADD COLUMN episode_format TEXT NOT NULL DEFAULT '{}';
ALTER TABLE settings ADD COLUMN podcast_format TEXT NOT NULL DEFAULT '{}';