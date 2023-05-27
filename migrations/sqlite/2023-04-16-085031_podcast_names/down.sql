-- This file should undo anything in `up.sql`
ALTER TABLE podcasts RENAME COLUMN directory_id TO directory;
ALTER TABLE podcasts DROP COLUMN directory_name;