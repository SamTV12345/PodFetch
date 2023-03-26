-- This file should undo anything in `up.sql`
ALTER TABLE podcasts DROP COLUMN summary;
ALTER TABLE podcasts DROP COLUMN language;
ALTER TABLE podcasts DROP COLUMN explicit;
ALTER TABLE podcasts DROP COLUMN keywords;
ALTER TABLE podcasts DROP COLUMN last_build_date;
ALTER TABLE podcasts DROP COLUMN author;