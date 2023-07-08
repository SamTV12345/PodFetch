-- This file should undo anything in `up.sql`
ALTER TABLE episodes ALTER COLUMN episode TYPE varchar(255);
ALTER TABLE episodes ALTER COLUMN guid TYPE varchar(255);
ALTER TABLE podcasts ALTER COLUMN original_image_url TYPE varchar(255);
ALTER TABLE podcasts ALTER COLUMN directory_name TYPE varchar(255);