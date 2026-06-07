ALTER TABLE settings ADD COLUMN nfo_format TEXT NOT NULL DEFAULT 'off';
ALTER TABLE settings ADD COLUMN cover_filename TEXT NOT NULL DEFAULT 'image';
ALTER TABLE podcast_settings ADD COLUMN nfo_format TEXT NOT NULL DEFAULT 'off';
ALTER TABLE podcast_settings ADD COLUMN cover_filename TEXT NOT NULL DEFAULT 'image';
