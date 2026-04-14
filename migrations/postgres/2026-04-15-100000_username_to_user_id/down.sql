-- Reverse migration: Convert user_id back to username

-- 1. favorites
ALTER TABLE favorites ADD COLUMN username TEXT;
UPDATE favorites SET username = u.username FROM users u WHERE favorites.user_id = u.id;
ALTER TABLE favorites DROP CONSTRAINT fk_favorites_user;
ALTER TABLE favorites DROP CONSTRAINT favorites_pkey;
ALTER TABLE favorites DROP COLUMN user_id;
ALTER TABLE favorites ALTER COLUMN username SET NOT NULL;
ALTER TABLE favorites ADD PRIMARY KEY (username, podcast_id);

-- 2. filters
ALTER TABLE filters ADD COLUMN username TEXT;
UPDATE filters SET username = u.username FROM users u WHERE filters.user_id = u.id;
ALTER TABLE filters DROP CONSTRAINT fk_filters_user;
ALTER TABLE filters DROP CONSTRAINT filters_pkey;
ALTER TABLE filters DROP COLUMN user_id;
ALTER TABLE filters ALTER COLUMN username SET NOT NULL;
ALTER TABLE filters ADD PRIMARY KEY (username);

-- 3. sessions
ALTER TABLE sessions DROP CONSTRAINT fk_sessions_user;
ALTER TABLE sessions DROP CONSTRAINT sessions_pkey;
ALTER TABLE sessions DROP COLUMN user_id;
ALTER TABLE sessions ADD PRIMARY KEY (username, session_id);

-- 4. favorite_podcast_episodes
ALTER TABLE favorite_podcast_episodes ADD COLUMN username TEXT;
UPDATE favorite_podcast_episodes SET username = u.username FROM users u WHERE favorite_podcast_episodes.user_id = u.id;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT fk_fpe_user;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT favorite_podcast_episodes_pkey;
ALTER TABLE favorite_podcast_episodes DROP COLUMN user_id;
ALTER TABLE favorite_podcast_episodes ALTER COLUMN username SET NOT NULL;
ALTER TABLE favorite_podcast_episodes ADD PRIMARY KEY (username, episode_id);

-- 5. subscriptions
ALTER TABLE subscriptions ADD COLUMN username TEXT;
UPDATE subscriptions SET username = u.username FROM users u WHERE subscriptions.user_id = u.id;
ALTER TABLE subscriptions DROP CONSTRAINT fk_subscriptions_user;
ALTER TABLE subscriptions DROP CONSTRAINT subscriptions_user_id_device_podcast_key;
ALTER TABLE subscriptions DROP COLUMN user_id;
ALTER TABLE subscriptions ALTER COLUMN username SET NOT NULL;
ALTER TABLE subscriptions ADD CONSTRAINT subscriptions_username_device_podcast_key UNIQUE (username, device, podcast);

-- 6. devices
ALTER TABLE devices ADD COLUMN username TEXT;
UPDATE devices SET username = u.username FROM users u WHERE devices.user_id = u.id;
ALTER TABLE devices DROP CONSTRAINT fk_devices_user;
ALTER TABLE devices DROP COLUMN user_id;
ALTER TABLE devices ALTER COLUMN username SET NOT NULL;

-- 7. episodes
ALTER TABLE episodes ADD COLUMN username TEXT;
UPDATE episodes SET username = u.username FROM users u WHERE episodes.user_id = u.id;
ALTER TABLE episodes DROP CONSTRAINT fk_episodes_user;
ALTER TABLE episodes DROP CONSTRAINT episodes_user_id_device_podcast_episode_timestamp_key;
ALTER TABLE episodes DROP COLUMN user_id;
ALTER TABLE episodes ALTER COLUMN username SET NOT NULL;
ALTER TABLE episodes ADD CONSTRAINT episodes_username_device_podcast_episode_timestamp_key UNIQUE (username, device, podcast, episode, timestamp);

-- 8. tags
DROP INDEX IF EXISTS idx_tags_user_id;
ALTER TABLE tags ADD COLUMN username TEXT;
UPDATE tags SET username = u.username FROM users u WHERE tags.user_id = u.id;
ALTER TABLE tags DROP CONSTRAINT fk_tags_user;
ALTER TABLE tags DROP CONSTRAINT tags_name_user_id_key;
ALTER TABLE tags DROP COLUMN user_id;
ALTER TABLE tags ALTER COLUMN username SET NOT NULL;
ALTER TABLE tags ADD CONSTRAINT tags_name_username_key UNIQUE (name, username);
CREATE INDEX idx_tags_username ON tags (username);

-- 9. listening_events
DROP INDEX IF EXISTS idx_listening_events_user_id_time;
DROP INDEX IF EXISTS idx_listening_events_user_id_episode;
DROP INDEX IF EXISTS idx_listening_events_user_id_podcast;
ALTER TABLE listening_events ADD COLUMN username TEXT;
UPDATE listening_events SET username = u.username FROM users u WHERE listening_events.user_id = u.id;
ALTER TABLE listening_events DROP CONSTRAINT fk_listening_events_user;
ALTER TABLE listening_events DROP COLUMN user_id;
ALTER TABLE listening_events ALTER COLUMN username SET NOT NULL;
CREATE INDEX idx_listening_events_username_time ON listening_events (username, listened_at);
CREATE INDEX idx_listening_events_username_episode ON listening_events (username, podcast_episode_id);
CREATE INDEX idx_listening_events_username_podcast ON listening_events (username, podcast_id);

-- 10. device_sync_groups
ALTER TABLE device_sync_groups ADD COLUMN username TEXT;
UPDATE device_sync_groups SET username = u.username FROM users u WHERE device_sync_groups.user_id = u.id;
ALTER TABLE device_sync_groups DROP CONSTRAINT fk_device_sync_groups_user;
ALTER TABLE device_sync_groups DROP CONSTRAINT device_sync_groups_user_id_device_id_key;
ALTER TABLE device_sync_groups DROP COLUMN user_id;
ALTER TABLE device_sync_groups ALTER COLUMN username SET NOT NULL;
ALTER TABLE device_sync_groups ADD CONSTRAINT device_sync_groups_username_device_id_key UNIQUE (username, device_id);

-- 11. gpodder_settings
ALTER TABLE gpodder_settings ADD COLUMN username TEXT;
UPDATE gpodder_settings SET username = u.username FROM users u WHERE gpodder_settings.user_id = u.id;
ALTER TABLE gpodder_settings DROP CONSTRAINT fk_gpodder_settings_user;
ALTER TABLE gpodder_settings DROP CONSTRAINT gpodder_settings_user_id_scope_scope_id_key;
ALTER TABLE gpodder_settings DROP COLUMN user_id;
ALTER TABLE gpodder_settings ALTER COLUMN username SET NOT NULL;
ALTER TABLE gpodder_settings ADD CONSTRAINT gpodder_settings_username_scope_scope_id_key UNIQUE (username, scope, scope_id);

-- 12. podcasts.added_by (INTEGER -> TEXT)
ALTER TABLE podcasts ADD COLUMN added_by_text TEXT;
UPDATE podcasts SET added_by_text = u.username FROM users u WHERE podcasts.added_by = u.id;
ALTER TABLE podcasts DROP COLUMN added_by;
ALTER TABLE podcasts RENAME COLUMN added_by_text TO added_by;
