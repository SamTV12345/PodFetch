-- Your SQL goes here
ALTER TABLE podcasts ADD COLUMN guid TEXT;
CREATE UNIQUE INDEX unique_guid ON podcasts (guid);