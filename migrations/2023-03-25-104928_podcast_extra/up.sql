-- Your SQL goes here
ALTER TABLE podcasts ADD COLUMN summary TEXT NULL;
ALTER TABLE podcasts ADD COLUMN language TEXT NULL;
ALTER TABLE podcasts ADD COLUMN explicit TEXT NULL;
ALTER TABLE podcasts ADD COLUMN keywords TEXT NULL;
ALTER TABLE podcasts ADD COLUMN last_build_date TEXT NULL;
ALTER TABLE podcasts ADD COLUMN author TEXT NULL;
