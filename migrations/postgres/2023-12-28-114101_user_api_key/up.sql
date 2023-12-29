-- Your SQL goes here
ALTER TABLE users ADD COLUMN api_key VARCHAR(255);
CREATE INDEX users_api_key_idx ON users (api_key);