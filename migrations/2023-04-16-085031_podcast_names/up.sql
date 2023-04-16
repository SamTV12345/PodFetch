-- Your SQL goes here
ALTER TABLE podcasts RENAME COLUMN directory TO directory_id;
ALTER TABLE podcasts ADD COLUMN directory_name VARCHAR(255) NOT NULL DEFAULT '';