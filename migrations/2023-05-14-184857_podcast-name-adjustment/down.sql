-- This file should undo anything in `up.sql`
ALTER TABLE settings DROP COLUMN replace_invalid_characters;
ALTER TABLE settings DROP COLUMN use_existing_filename;
ALTER TABLE settings DROP COLUMN replacement_strategy;
ALTER TABLE settings DROP COLUMN episode_format;