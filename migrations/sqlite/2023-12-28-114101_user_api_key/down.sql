-- This file should undo anything in `up.sql`
ALTER TABLE users DROP COLUMN api_key;
DROP INDEX users_api_key_idx;