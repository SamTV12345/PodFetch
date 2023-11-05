-- Your SQL goes here
ALTER TABLE episodes ADD COLUMN cleaned_url TEXT NOT NULL DEFAULT '';
UPDATE episodes SET cleaned_url = episode;