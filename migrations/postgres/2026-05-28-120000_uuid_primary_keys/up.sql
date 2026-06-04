-- Migration: Convert all integer primary keys (and their integer foreign keys)
-- to TEXT UUIDs. podcasts and podcast_episodes additionally keep their old
-- integer id as legacy_id for backwards-compatible lookups.
--
-- PostgreSQL supports in-place ALTER TABLE and transactional DDL, so no table
-- recreation is needed. All foreign-key constraints that involve a converting
-- column are dropped up front and re-added at the end, so the column swaps run
-- without enforcement and the re-added constraints validate the final state.
-- gen_random_uuid() is built in on PostgreSQL 13+.

-- ============================================================
-- A1. Add uuid to every integer-PK parent + legacy_id where required.
-- ============================================================
ALTER TABLE users ADD COLUMN uuid TEXT;
UPDATE users SET uuid = gen_random_uuid()::text;

ALTER TABLE podcasts ADD COLUMN uuid TEXT;
UPDATE podcasts SET uuid = gen_random_uuid()::text;
ALTER TABLE podcasts ADD COLUMN legacy_id BIGINT;
UPDATE podcasts SET legacy_id = id;

ALTER TABLE podcast_episodes ADD COLUMN uuid TEXT;
UPDATE podcast_episodes SET uuid = gen_random_uuid()::text;
ALTER TABLE podcast_episodes ADD COLUMN legacy_id BIGINT;
UPDATE podcast_episodes SET legacy_id = id;

ALTER TABLE devices ADD COLUMN uuid TEXT;
UPDATE devices SET uuid = gen_random_uuid()::text;

ALTER TABLE subscriptions ADD COLUMN uuid TEXT;
UPDATE subscriptions SET uuid = gen_random_uuid()::text;

ALTER TABLE episodes ADD COLUMN uuid TEXT;
UPDATE episodes SET uuid = gen_random_uuid()::text;

ALTER TABLE gpodder_settings ADD COLUMN uuid TEXT;
UPDATE gpodder_settings SET uuid = gen_random_uuid()::text;

ALTER TABLE device_sync_groups ADD COLUMN uuid TEXT;
UPDATE device_sync_groups SET uuid = gen_random_uuid()::text;

ALTER TABLE notifications ADD COLUMN uuid TEXT;
UPDATE notifications SET uuid = gen_random_uuid()::text;

ALTER TABLE settings ADD COLUMN uuid TEXT;
UPDATE settings SET uuid = gen_random_uuid()::text;

ALTER TABLE listening_events ADD COLUMN uuid TEXT;
UPDATE listening_events SET uuid = gen_random_uuid()::text;

-- ============================================================
-- A2. Add *_uuid backfill columns to every child FK and populate them by
--     joining on the parent's still-present integer id.
-- ============================================================
ALTER TABLE podcasts ADD COLUMN added_by_uuid TEXT;
UPDATE podcasts SET added_by_uuid = u.uuid FROM users u WHERE podcasts.added_by = u.id;

ALTER TABLE podcast_episodes ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_episodes SET podcast_id_uuid = p.uuid FROM podcasts p WHERE podcast_episodes.podcast_id = p.id;

ALTER TABLE devices ADD COLUMN user_id_uuid TEXT;
UPDATE devices SET user_id_uuid = u.uuid FROM users u WHERE devices.user_id = u.id;

ALTER TABLE subscriptions ADD COLUMN user_id_uuid TEXT;
UPDATE subscriptions SET user_id_uuid = u.uuid FROM users u WHERE subscriptions.user_id = u.id;

ALTER TABLE episodes ADD COLUMN user_id_uuid TEXT;
UPDATE episodes SET user_id_uuid = u.uuid FROM users u WHERE episodes.user_id = u.id;

ALTER TABLE gpodder_settings ADD COLUMN user_id_uuid TEXT;
UPDATE gpodder_settings SET user_id_uuid = u.uuid FROM users u WHERE gpodder_settings.user_id = u.id;

ALTER TABLE device_sync_groups ADD COLUMN user_id_uuid TEXT;
UPDATE device_sync_groups SET user_id_uuid = u.uuid FROM users u WHERE device_sync_groups.user_id = u.id;

ALTER TABLE listening_events ADD COLUMN user_id_uuid TEXT;
UPDATE listening_events SET user_id_uuid = u.uuid FROM users u WHERE listening_events.user_id = u.id;
ALTER TABLE listening_events ADD COLUMN podcast_id_uuid TEXT;
UPDATE listening_events SET podcast_id_uuid = p.uuid FROM podcasts p WHERE listening_events.podcast_id = p.id;
ALTER TABLE listening_events ADD COLUMN podcast_episode_db_id_uuid TEXT;
UPDATE listening_events SET podcast_episode_db_id_uuid = e.uuid FROM podcast_episodes e WHERE listening_events.podcast_episode_db_id = e.id;

ALTER TABLE podcast_settings ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_settings SET podcast_id_uuid = p.uuid FROM podcasts p WHERE podcast_settings.podcast_id = p.id;

ALTER TABLE favorites ADD COLUMN user_id_uuid TEXT;
UPDATE favorites SET user_id_uuid = u.uuid FROM users u WHERE favorites.user_id = u.id;
ALTER TABLE favorites ADD COLUMN podcast_id_uuid TEXT;
UPDATE favorites SET podcast_id_uuid = p.uuid FROM podcasts p WHERE favorites.podcast_id = p.id;

ALTER TABLE favorite_podcast_episodes ADD COLUMN user_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET user_id_uuid = u.uuid FROM users u WHERE favorite_podcast_episodes.user_id = u.id;
ALTER TABLE favorite_podcast_episodes ADD COLUMN episode_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET episode_id_uuid = e.uuid FROM podcast_episodes e WHERE favorite_podcast_episodes.episode_id = e.id;

ALTER TABLE filters ADD COLUMN user_id_uuid TEXT;
UPDATE filters SET user_id_uuid = u.uuid FROM users u WHERE filters.user_id = u.id;

ALTER TABLE sessions ADD COLUMN user_id_uuid TEXT;
UPDATE sessions SET user_id_uuid = u.uuid FROM users u WHERE sessions.user_id = u.id;

ALTER TABLE tags ADD COLUMN user_id_uuid TEXT;
UPDATE tags SET user_id_uuid = u.uuid FROM users u WHERE tags.user_id = u.id;

ALTER TABLE tags_podcasts ADD COLUMN podcast_id_uuid TEXT;
UPDATE tags_podcasts SET podcast_id_uuid = p.uuid FROM podcasts p WHERE tags_podcasts.podcast_id = p.id;

ALTER TABLE playlists ADD COLUMN user_id_uuid TEXT;
UPDATE playlists SET user_id_uuid = u.uuid FROM users u WHERE playlists.user_id = u.id;

ALTER TABLE playlist_items ADD COLUMN episode_uuid TEXT;
UPDATE playlist_items SET episode_uuid = e.uuid FROM podcast_episodes e WHERE playlist_items.episode = e.id;

ALTER TABLE podcast_episode_chapters ADD COLUMN episode_id_uuid TEXT;
UPDATE podcast_episode_chapters SET episode_id_uuid = e.uuid FROM podcast_episodes e WHERE podcast_episode_chapters.episode_id = e.id;

ALTER TABLE audiobookshelf_listening_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_listening_sessions SET user_id_uuid = u.uuid FROM users u WHERE audiobookshelf_listening_sessions.user_id = u.id;
ALTER TABLE audiobookshelf_media_progress ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_media_progress SET user_id_uuid = u.uuid FROM users u WHERE audiobookshelf_media_progress.user_id = u.id;
ALTER TABLE audiobookshelf_playback_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_playback_sessions SET user_id_uuid = u.uuid FROM users u WHERE audiobookshelf_playback_sessions.user_id = u.id;

-- ============================================================
-- B. Drop every foreign-key constraint that involves a converting column.
--    (FKs onto unchanged TEXT PKs -- playlist_items.playlist_id -> playlists,
--     tags_podcasts.tag_id -> tags -- are left in place.)
-- ============================================================
ALTER TABLE devices DROP CONSTRAINT fk_devices_user;
ALTER TABLE device_sync_groups DROP CONSTRAINT fk_device_sync_groups_user;
ALTER TABLE episodes DROP CONSTRAINT fk_episodes_user;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT fk_fpe_user;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT favorite_podcast_episodes_episode_id_fkey;
ALTER TABLE favorites DROP CONSTRAINT favorites_podcast_id_fkey;
ALTER TABLE favorites DROP CONSTRAINT fk_favorites_user;
ALTER TABLE filters DROP CONSTRAINT fk_filters_user;
ALTER TABLE gpodder_settings DROP CONSTRAINT fk_gpodder_settings_user;
ALTER TABLE listening_events DROP CONSTRAINT listening_events_podcast_id_fkey;
ALTER TABLE listening_events DROP CONSTRAINT listening_events_podcast_episode_db_id_fkey;
ALTER TABLE listening_events DROP CONSTRAINT fk_listening_events_user;
ALTER TABLE playlist_items DROP CONSTRAINT playlist_items_episode_fkey;
ALTER TABLE podcast_episode_chapters DROP CONSTRAINT podcast_episode_chapters_episode_id_fkey;
ALTER TABLE podcast_episodes DROP CONSTRAINT podcast_episodes_podcast_id_fkey;
ALTER TABLE podcasts DROP CONSTRAINT podcasts_added_by_int_fkey;
ALTER TABLE sessions DROP CONSTRAINT fk_sessions_user;
ALTER TABLE subscriptions DROP CONSTRAINT fk_subscriptions_user;
ALTER TABLE tags DROP CONSTRAINT fk_tags_user;
ALTER TABLE tags_podcasts DROP CONSTRAINT tags_podcasts_podcast_id_fkey;

-- ============================================================
-- C. Drop primary-key and unique constraints that include a converting column.
-- ============================================================
ALTER TABLE users DROP CONSTRAINT users_pkey;
ALTER TABLE podcasts DROP CONSTRAINT podcasts_pkey;
ALTER TABLE podcast_episodes DROP CONSTRAINT podcast_episodes_pkey;
ALTER TABLE devices DROP CONSTRAINT devices_pkey;
ALTER TABLE subscriptions DROP CONSTRAINT subscriptions_pkey;
ALTER TABLE episodes DROP CONSTRAINT episodes_pkey;
ALTER TABLE gpodder_settings DROP CONSTRAINT gpodder_settings_pkey;
ALTER TABLE device_sync_groups DROP CONSTRAINT device_sync_groups_pkey;
ALTER TABLE notifications DROP CONSTRAINT notifications_pkey;
ALTER TABLE settings DROP CONSTRAINT settings_pkey;
ALTER TABLE listening_events DROP CONSTRAINT listening_events_pkey;
ALTER TABLE podcast_settings DROP CONSTRAINT podcast_settings_pkey;
ALTER TABLE favorites DROP CONSTRAINT favorites_pkey;
ALTER TABLE favorite_podcast_episodes DROP CONSTRAINT favorite_podcast_episodes_pkey;
ALTER TABLE filters DROP CONSTRAINT filters_pkey;
ALTER TABLE sessions DROP CONSTRAINT sessions_pkey;
ALTER TABLE playlist_items DROP CONSTRAINT playlist_items_pkey;
ALTER TABLE tags_podcasts DROP CONSTRAINT tags_podcasts_pkey;

ALTER TABLE device_sync_groups DROP CONSTRAINT device_sync_groups_user_id_device_id_key;
ALTER TABLE episodes DROP CONSTRAINT episodes_user_id_device_podcast_episode_timestamp_key;
ALTER TABLE gpodder_settings DROP CONSTRAINT gpodder_settings_user_id_scope_scope_id_key;
ALTER TABLE subscriptions DROP CONSTRAINT subscriptions_user_id_device_podcast_key;
ALTER TABLE tags DROP CONSTRAINT tags_name_user_id_key;

-- ============================================================
-- D. Swap each converting column: drop the integer column, rename the uuid
--    column into its place, and restore NOT NULL where the original required it.
-- ============================================================
-- Parents (integer id -> text uuid)
ALTER TABLE users DROP COLUMN id;
ALTER TABLE users RENAME COLUMN uuid TO id;
ALTER TABLE users ALTER COLUMN id SET NOT NULL;

ALTER TABLE podcasts DROP COLUMN id;
ALTER TABLE podcasts RENAME COLUMN uuid TO id;
ALTER TABLE podcasts ALTER COLUMN id SET NOT NULL;
ALTER TABLE podcasts DROP COLUMN added_by;
ALTER TABLE podcasts RENAME COLUMN added_by_uuid TO added_by;

ALTER TABLE podcast_episodes DROP COLUMN id;
ALTER TABLE podcast_episodes RENAME COLUMN uuid TO id;
ALTER TABLE podcast_episodes ALTER COLUMN id SET NOT NULL;
ALTER TABLE podcast_episodes DROP COLUMN podcast_id;
ALTER TABLE podcast_episodes RENAME COLUMN podcast_id_uuid TO podcast_id;
ALTER TABLE podcast_episodes ALTER COLUMN podcast_id SET NOT NULL;

ALTER TABLE devices DROP COLUMN id;
ALTER TABLE devices RENAME COLUMN uuid TO id;
ALTER TABLE devices ALTER COLUMN id SET NOT NULL;
ALTER TABLE devices DROP COLUMN user_id;
ALTER TABLE devices RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE devices ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE subscriptions DROP COLUMN id;
ALTER TABLE subscriptions RENAME COLUMN uuid TO id;
ALTER TABLE subscriptions ALTER COLUMN id SET NOT NULL;
ALTER TABLE subscriptions DROP COLUMN user_id;
ALTER TABLE subscriptions RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE subscriptions ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE episodes DROP COLUMN id;
ALTER TABLE episodes RENAME COLUMN uuid TO id;
ALTER TABLE episodes ALTER COLUMN id SET NOT NULL;
ALTER TABLE episodes DROP COLUMN user_id;
ALTER TABLE episodes RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE episodes ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE gpodder_settings DROP COLUMN id;
ALTER TABLE gpodder_settings RENAME COLUMN uuid TO id;
ALTER TABLE gpodder_settings ALTER COLUMN id SET NOT NULL;
ALTER TABLE gpodder_settings DROP COLUMN user_id;
ALTER TABLE gpodder_settings RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE gpodder_settings ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE device_sync_groups DROP COLUMN id;
ALTER TABLE device_sync_groups RENAME COLUMN uuid TO id;
ALTER TABLE device_sync_groups ALTER COLUMN id SET NOT NULL;
ALTER TABLE device_sync_groups DROP COLUMN user_id;
ALTER TABLE device_sync_groups RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE device_sync_groups ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE notifications DROP COLUMN id;
ALTER TABLE notifications RENAME COLUMN uuid TO id;
ALTER TABLE notifications ALTER COLUMN id SET NOT NULL;

ALTER TABLE settings DROP COLUMN id;
ALTER TABLE settings RENAME COLUMN uuid TO id;
ALTER TABLE settings ALTER COLUMN id SET NOT NULL;

ALTER TABLE listening_events DROP COLUMN id;
ALTER TABLE listening_events RENAME COLUMN uuid TO id;
ALTER TABLE listening_events ALTER COLUMN id SET NOT NULL;
ALTER TABLE listening_events DROP COLUMN user_id;
ALTER TABLE listening_events RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE listening_events ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE listening_events DROP COLUMN podcast_id;
ALTER TABLE listening_events RENAME COLUMN podcast_id_uuid TO podcast_id;
ALTER TABLE listening_events ALTER COLUMN podcast_id SET NOT NULL;
ALTER TABLE listening_events DROP COLUMN podcast_episode_db_id;
ALTER TABLE listening_events RENAME COLUMN podcast_episode_db_id_uuid TO podcast_episode_db_id;
ALTER TABLE listening_events ALTER COLUMN podcast_episode_db_id SET NOT NULL;

ALTER TABLE podcast_settings DROP COLUMN podcast_id;
ALTER TABLE podcast_settings RENAME COLUMN podcast_id_uuid TO podcast_id;
ALTER TABLE podcast_settings ALTER COLUMN podcast_id SET NOT NULL;

ALTER TABLE favorites DROP COLUMN user_id;
ALTER TABLE favorites RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE favorites ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE favorites DROP COLUMN podcast_id;
ALTER TABLE favorites RENAME COLUMN podcast_id_uuid TO podcast_id;
ALTER TABLE favorites ALTER COLUMN podcast_id SET NOT NULL;

ALTER TABLE favorite_podcast_episodes DROP COLUMN user_id;
ALTER TABLE favorite_podcast_episodes RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE favorite_podcast_episodes ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE favorite_podcast_episodes DROP COLUMN episode_id;
ALTER TABLE favorite_podcast_episodes RENAME COLUMN episode_id_uuid TO episode_id;
ALTER TABLE favorite_podcast_episodes ALTER COLUMN episode_id SET NOT NULL;

ALTER TABLE filters DROP COLUMN user_id;
ALTER TABLE filters RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE filters ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE sessions DROP COLUMN user_id;
ALTER TABLE sessions RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE sessions ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE tags DROP COLUMN user_id;
ALTER TABLE tags RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE tags ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE tags_podcasts DROP COLUMN podcast_id;
ALTER TABLE tags_podcasts RENAME COLUMN podcast_id_uuid TO podcast_id;
ALTER TABLE tags_podcasts ALTER COLUMN podcast_id SET NOT NULL;

ALTER TABLE playlists DROP COLUMN user_id;
ALTER TABLE playlists RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE playlists ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE playlist_items DROP COLUMN episode;
ALTER TABLE playlist_items RENAME COLUMN episode_uuid TO episode;
ALTER TABLE playlist_items ALTER COLUMN episode SET NOT NULL;

ALTER TABLE podcast_episode_chapters DROP COLUMN episode_id;
ALTER TABLE podcast_episode_chapters RENAME COLUMN episode_id_uuid TO episode_id;
ALTER TABLE podcast_episode_chapters ALTER COLUMN episode_id SET NOT NULL;

ALTER TABLE audiobookshelf_listening_sessions DROP COLUMN user_id;
ALTER TABLE audiobookshelf_listening_sessions RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE audiobookshelf_listening_sessions ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE audiobookshelf_media_progress DROP COLUMN user_id;
ALTER TABLE audiobookshelf_media_progress RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE audiobookshelf_media_progress ALTER COLUMN user_id SET NOT NULL;

ALTER TABLE audiobookshelf_playback_sessions DROP COLUMN user_id;
ALTER TABLE audiobookshelf_playback_sessions RENAME COLUMN user_id_uuid TO user_id;
ALTER TABLE audiobookshelf_playback_sessions ALTER COLUMN user_id SET NOT NULL;

-- ============================================================
-- E. Re-add primary keys, unique constraints, foreign keys, and legacy_id
--    uniqueness. Parents' PKs are added before the child FKs that reference them.
-- ============================================================
ALTER TABLE users ADD PRIMARY KEY (id);
ALTER TABLE podcasts ADD PRIMARY KEY (id);
ALTER TABLE podcast_episodes ADD PRIMARY KEY (id);
ALTER TABLE devices ADD PRIMARY KEY (id);
ALTER TABLE subscriptions ADD PRIMARY KEY (id);
ALTER TABLE episodes ADD PRIMARY KEY (id);
ALTER TABLE gpodder_settings ADD PRIMARY KEY (id);
ALTER TABLE device_sync_groups ADD PRIMARY KEY (id);
ALTER TABLE notifications ADD PRIMARY KEY (id);
ALTER TABLE settings ADD PRIMARY KEY (id);
ALTER TABLE listening_events ADD PRIMARY KEY (id);
ALTER TABLE podcast_settings ADD PRIMARY KEY (podcast_id);
ALTER TABLE favorites ADD PRIMARY KEY (user_id, podcast_id);
ALTER TABLE favorite_podcast_episodes ADD PRIMARY KEY (user_id, episode_id);
ALTER TABLE filters ADD PRIMARY KEY (user_id);
ALTER TABLE sessions ADD PRIMARY KEY (user_id, session_id);
ALTER TABLE playlist_items ADD PRIMARY KEY (playlist_id, episode);
ALTER TABLE tags_podcasts ADD PRIMARY KEY (tag_id, podcast_id);

ALTER TABLE podcasts ADD CONSTRAINT podcasts_legacy_id_key UNIQUE (legacy_id);
ALTER TABLE podcast_episodes ADD CONSTRAINT podcast_episodes_legacy_id_key UNIQUE (legacy_id);

ALTER TABLE device_sync_groups ADD CONSTRAINT device_sync_groups_user_id_device_id_key UNIQUE (user_id, device_id);
ALTER TABLE episodes ADD CONSTRAINT episodes_user_id_device_podcast_episode_timestamp_key UNIQUE (user_id, device, podcast, episode, "timestamp");
ALTER TABLE gpodder_settings ADD CONSTRAINT gpodder_settings_user_id_scope_scope_id_key UNIQUE (user_id, scope, scope_id);
ALTER TABLE subscriptions ADD CONSTRAINT subscriptions_user_id_device_podcast_key UNIQUE (user_id, device, podcast);
ALTER TABLE tags ADD CONSTRAINT tags_name_user_id_key UNIQUE (name, user_id);

ALTER TABLE podcasts ADD CONSTRAINT podcasts_added_by_fkey FOREIGN KEY (added_by) REFERENCES users(id);
ALTER TABLE podcast_episodes ADD CONSTRAINT podcast_episodes_podcast_id_fkey FOREIGN KEY (podcast_id) REFERENCES podcasts(id);
ALTER TABLE devices ADD CONSTRAINT fk_devices_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE subscriptions ADD CONSTRAINT fk_subscriptions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE episodes ADD CONSTRAINT fk_episodes_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE gpodder_settings ADD CONSTRAINT fk_gpodder_settings_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE device_sync_groups ADD CONSTRAINT fk_device_sync_groups_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE listening_events ADD CONSTRAINT fk_listening_events_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE listening_events ADD CONSTRAINT listening_events_podcast_id_fkey FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE;
ALTER TABLE listening_events ADD CONSTRAINT listening_events_podcast_episode_db_id_fkey FOREIGN KEY (podcast_episode_db_id) REFERENCES podcast_episodes(id) ON DELETE CASCADE;
ALTER TABLE podcast_settings ADD CONSTRAINT podcast_settings_podcast_id_fkey FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE;
ALTER TABLE favorites ADD CONSTRAINT fk_favorites_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE favorites ADD CONSTRAINT favorites_podcast_id_fkey FOREIGN KEY (podcast_id) REFERENCES podcasts(id) ON DELETE CASCADE;
ALTER TABLE favorite_podcast_episodes ADD CONSTRAINT fk_fpe_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE favorite_podcast_episodes ADD CONSTRAINT favorite_podcast_episodes_episode_id_fkey FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id);
ALTER TABLE filters ADD CONSTRAINT fk_filters_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE sessions ADD CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE tags ADD CONSTRAINT fk_tags_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE tags_podcasts ADD CONSTRAINT tags_podcasts_podcast_id_fkey FOREIGN KEY (podcast_id) REFERENCES podcasts(id);
ALTER TABLE playlist_items ADD CONSTRAINT playlist_items_episode_fkey FOREIGN KEY (episode) REFERENCES podcast_episodes(id);
ALTER TABLE podcast_episode_chapters ADD CONSTRAINT podcast_episode_chapters_episode_id_fkey FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id) ON DELETE CASCADE;

-- Recreate every index that sat on a converting column (dropping the integer
-- column dropped its indexes). Names/shapes match the prior migrations exactly.
CREATE INDEX idx_listening_events_user_id_time ON listening_events (user_id, listened_at);
CREATE INDEX idx_listening_events_user_id_episode ON listening_events (user_id, podcast_episode_id);
CREATE INDEX idx_listening_events_user_id_podcast ON listening_events (user_id, podcast_id);
CREATE INDEX idx_tags_user_id ON tags (user_id);
CREATE INDEX idx_podcast_episodes ON podcast_episodes (podcast_id);
CREATE INDEX podcast_episodes_podcast_id_index ON podcast_episodes (podcast_id);
CREATE UNIQUE INDEX uq_podcast_episode_chapters_episode_start ON podcast_episode_chapters (episode_id, start_time);
CREATE INDEX audiobookshelf_listening_sessions_user_idx ON audiobookshelf_listening_sessions (user_id);
CREATE INDEX audiobookshelf_media_progress_user_idx ON audiobookshelf_media_progress (user_id);
CREATE UNIQUE INDEX audiobookshelf_media_progress_user_item_episode_idx ON audiobookshelf_media_progress (user_id, library_item_id, COALESCE(episode_id, ''));
CREATE INDEX audiobookshelf_playback_sessions_user_idx ON audiobookshelf_playback_sessions (user_id);
