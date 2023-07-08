-- Your SQL goes here
ALTER TABLE episodes ALTER COLUMN episode TYPE TEXT;
ALTER TABLE episodes ALTER COLUMN guid TYPE TEXT;
ALTER TABLE podcasts ALTER COLUMN original_image_url TYPE TEXT;
ALTER TABLE podcasts ALTER COLUMN directory_name TYPE TEXT;