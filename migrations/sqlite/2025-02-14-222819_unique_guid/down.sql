-- This file should undo anything in `up.sql`
ALTER TABLE podcasts DROP COLUMN guid;
DROP INDEX unique_guid;