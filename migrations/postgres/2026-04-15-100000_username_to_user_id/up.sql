-- Migration: Convert all tables from username (TEXT) to user_id (INTEGER) foreign key
-- PostgreSQL supports ALTER TABLE ADD/DROP COLUMN natively

-- 1. favorites
ALTER TABLE favorites ADD COLUMN user_id INTEGER;
UPDATE favorites SET user_id = u.id FROM users u WHERE favorites.username = u.username;
DELETE FROM favorites WHERE user_id IS NULL;
ALTER TABLE favorites ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE favorites DROP CONSTRAINT favorites_pkey;
ALTER TABLE favorites DROP COLUMN username;
ALTER TABLE favorites ADD PRIMARY KEY (user_id, podcast_id);
ALTER TABLE favorites ADD CONSTRAINT fk_favorites_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 2. filters
ALTER TABLE filters ADD COLUMN user_id INTEGER;
UPDATE filters SET user_id = u.id FROM users u WHERE filters.username = u.username;
DELETE FROM filters WHERE user_id IS NULL;
ALTER TABLE filters ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE filters DROP CONSTRAINT filters_pkey;
ALTER TABLE filters DROP COLUMN username;
ALTER TABLE filters ADD PRIMARY KEY (user_id);
ALTER TABLE filters ADD CONSTRAINT fk_filters_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 3. sessions (keep username for GPodder URL validation)
ALTER TABLE sessions ADD COLUMN user_id INTEGER;
UPDATE sessions SET user_id = u.id FROM users u WHERE sessions.username = u.username;
DELETE FROM sessions WHERE user_id IS NULL;
ALTER TABLE sessions ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE sessions DROP CONSTRAINT sessions_pkey;
ALTER TABLE sessions ADD PRIMARY KEY (user_id, session_id);
ALTER TABLE sessions ADD CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 4. favorite_podcast_episodes
ALTER TABLE favorite_podcast_episodes ADD COLUMN user_id INTEGER;
UPDATE favorite_podcast_episodes SET user_id = u.id FROM users u WHERE favorite_podcast_episodes.username = u.username;
DELETE FROM favorite_podcast_episodes WHERE user_id IS NULL;
ALTER TABLE favorite_podcast_episodes ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT favorite_podcast_episodes_pkey;
ALTER TABLE favorite_podcast_episodes DROP COLUMN username;
ALTER TABLE favorite_podcast_episodes ADD PRIMARY KEY (user_id, episode_id);
ALTER TABLE favorite_podcast_episodes ADD CONSTRAINT fk_fpe_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 5. subscriptions
ALTER TABLE subscriptions ADD COLUMN user_id INTEGER;
UPDATE subscriptions SET user_id = u.id FROM users u WHERE subscriptions.username = u.username;
DELETE FROM subscriptions WHERE user_id IS NULL;
ALTER TABLE subscriptions ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE subscriptions DROP CONSTRAINT IF EXISTS subscriptions_username_device_podcast_key;
ALTER TABLE subscriptions DROP COLUMN username;
ALTER TABLE subscriptions ADD CONSTRAINT subscriptions_user_id_device_podcast_key UNIQUE (user_id, device, podcast);
ALTER TABLE subscriptions ADD CONSTRAINT fk_subscriptions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 6. devices
ALTER TABLE devices ADD COLUMN user_id INTEGER;
UPDATE devices SET user_id = u.id FROM users u WHERE devices.username = u.username;
DELETE FROM devices WHERE user_id IS NULL;
ALTER TABLE devices ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE devices DROP COLUMN username;
ALTER TABLE devices ADD CONSTRAINT fk_devices_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 7. episodes
ALTER TABLE episodes ADD COLUMN user_id INTEGER;
UPDATE episodes SET user_id = u.id FROM users u WHERE episodes.username = u.username;
DELETE FROM episodes WHERE user_id IS NULL;
ALTER TABLE episodes ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE episodes DROP CONSTRAINT IF EXISTS episodes_username_device_podcast_episode_timestamp_key;
ALTER TABLE episodes DROP COLUMN username;
ALTER TABLE episodes ADD CONSTRAINT episodes_user_id_device_podcast_episode_timestamp_key UNIQUE (user_id, device, podcast, episode, timestamp);
ALTER TABLE episodes ADD CONSTRAINT fk_episodes_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 8. tags
DROP INDEX IF EXISTS idx_tags_username;
ALTER TABLE tags ADD COLUMN user_id INTEGER;
UPDATE tags SET user_id = u.id FROM users u WHERE tags.username = u.username;
DELETE FROM tags WHERE user_id IS NULL;
ALTER TABLE tags ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE tags DROP CONSTRAINT IF EXISTS tags_name_username_key;
ALTER TABLE tags DROP COLUMN username;
ALTER TABLE tags ADD CONSTRAINT tags_name_user_id_key UNIQUE (name, user_id);
ALTER TABLE tags ADD CONSTRAINT fk_tags_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
CREATE INDEX idx_tags_user_id ON tags (user_id);

-- 9. listening_events
DROP INDEX IF EXISTS idx_listening_events_username_time;
DROP INDEX IF EXISTS idx_listening_events_username_episode;
DROP INDEX IF EXISTS idx_listening_events_username_podcast;
ALTER TABLE listening_events ADD COLUMN user_id INTEGER;
UPDATE listening_events SET user_id = u.id FROM users u WHERE listening_events.username = u.username;
DELETE FROM listening_events WHERE user_id IS NULL;
ALTER TABLE listening_events ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE listening_events DROP COLUMN username;
ALTER TABLE listening_events ADD CONSTRAINT fk_listening_events_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
CREATE INDEX idx_listening_events_user_id_time ON listening_events (user_id, listened_at);
CREATE INDEX idx_listening_events_user_id_episode ON listening_events (user_id, podcast_episode_id);
CREATE INDEX idx_listening_events_user_id_podcast ON listening_events (user_id, podcast_id);

-- 10. device_sync_groups
ALTER TABLE device_sync_groups ADD COLUMN user_id INTEGER;
UPDATE device_sync_groups SET user_id = u.id FROM users u WHERE device_sync_groups.username = u.username;
DELETE FROM device_sync_groups WHERE user_id IS NULL;
ALTER TABLE device_sync_groups ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE device_sync_groups DROP CONSTRAINT IF EXISTS device_sync_groups_username_device_id_key;
ALTER TABLE device_sync_groups DROP COLUMN username;
ALTER TABLE device_sync_groups ADD CONSTRAINT device_sync_groups_user_id_device_id_key UNIQUE (user_id, device_id);
ALTER TABLE device_sync_groups ADD CONSTRAINT fk_device_sync_groups_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 11. gpodder_settings
ALTER TABLE gpodder_settings ADD COLUMN user_id INTEGER;
UPDATE gpodder_settings SET user_id = u.id FROM users u WHERE gpodder_settings.username = u.username;
DELETE FROM gpodder_settings WHERE user_id IS NULL;
ALTER TABLE gpodder_settings ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE gpodder_settings DROP CONSTRAINT IF EXISTS gpodder_settings_username_scope_scope_id_key;
ALTER TABLE gpodder_settings DROP COLUMN username;
ALTER TABLE gpodder_settings ADD CONSTRAINT gpodder_settings_user_id_scope_scope_id_key UNIQUE (user_id, scope, scope_id);
ALTER TABLE gpodder_settings ADD CONSTRAINT fk_gpodder_settings_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 12. podcasts.added_by (TEXT -> INTEGER)
ALTER TABLE podcasts ADD COLUMN added_by_int INTEGER REFERENCES users(id);
UPDATE podcasts SET added_by_int = u.id FROM users u WHERE podcasts.added_by = u.username;
ALTER TABLE podcasts DROP COLUMN added_by;
ALTER TABLE podcasts RENAME COLUMN added_by_int TO added_by;
