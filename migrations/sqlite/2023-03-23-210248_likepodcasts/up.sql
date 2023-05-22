-- Your SQL goes here
-- 1 = favored, 0 = not favored
ALTER TABLE podcasts ADD COLUMN favored INT NOT NULL DEFAULT 0;