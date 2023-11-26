-- This file should undo anything in `up.sql`
DROP TABLE users;
DROP TABLE invites;
ALTER TABLE podcasts ADD COLUMN favored BOOLEAN;
DROP TABLE favorites;