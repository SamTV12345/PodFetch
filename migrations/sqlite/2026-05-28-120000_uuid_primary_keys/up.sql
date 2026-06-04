-- Migration: Convert all integer primary keys (and their integer foreign keys)
-- to TEXT UUIDs. podcasts and podcast_episodes additionally keep their old
-- integer id as legacy_id for backwards-compatible lookups.
-- SQLite cannot alter a primary key in place, so each table is recreated.
-- Foreign key enforcement is off during the migration.
PRAGMA foreign_keys = OFF;

-- ============================================================
-- 1. Add `uuid` to every integer-PK parent + `*_uuid` to children.
--    Process parents before children so the backfill subqueries see
--    populated uuid values.
-- ============================================================

-- users
ALTER TABLE users ADD COLUMN uuid TEXT;
UPDATE users SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);

-- podcasts (+ added_by remap)
ALTER TABLE podcasts ADD COLUMN uuid TEXT;
UPDATE podcasts SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE podcasts ADD COLUMN added_by_uuid TEXT;
UPDATE podcasts SET added_by_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = podcasts.added_by);

-- podcast_episodes (+ podcast_id remap)
ALTER TABLE podcast_episodes ADD COLUMN uuid TEXT;
UPDATE podcast_episodes SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE podcast_episodes ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_episodes SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = podcast_episodes.podcast_id);

-- devices
ALTER TABLE devices ADD COLUMN uuid TEXT;
UPDATE devices SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE devices ADD COLUMN user_id_uuid TEXT;
UPDATE devices SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = devices.user_id);

-- subscriptions
ALTER TABLE subscriptions ADD COLUMN uuid TEXT;
UPDATE subscriptions SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE subscriptions ADD COLUMN user_id_uuid TEXT;
UPDATE subscriptions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = subscriptions.user_id);

-- episodes (gpodder episode actions)
ALTER TABLE episodes ADD COLUMN uuid TEXT;
UPDATE episodes SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE episodes ADD COLUMN user_id_uuid TEXT;
UPDATE episodes SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = episodes.user_id);

-- gpodder_settings
ALTER TABLE gpodder_settings ADD COLUMN uuid TEXT;
UPDATE gpodder_settings SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE gpodder_settings ADD COLUMN user_id_uuid TEXT;
UPDATE gpodder_settings SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = gpodder_settings.user_id);

-- device_sync_groups
ALTER TABLE device_sync_groups ADD COLUMN uuid TEXT;
UPDATE device_sync_groups SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE device_sync_groups ADD COLUMN user_id_uuid TEXT;
UPDATE device_sync_groups SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = device_sync_groups.user_id);

-- notifications (no FK)
ALTER TABLE notifications ADD COLUMN uuid TEXT;
UPDATE notifications SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);

-- settings (singleton, no FK)
ALTER TABLE settings ADD COLUMN uuid TEXT;
UPDATE settings SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);

-- listening_events (FKs: user_id, podcast_id, podcast_episode_db_id)
ALTER TABLE listening_events ADD COLUMN uuid TEXT;
UPDATE listening_events SET uuid = lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
);
ALTER TABLE listening_events ADD COLUMN user_id_uuid TEXT;
UPDATE listening_events SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = listening_events.user_id);
ALTER TABLE listening_events ADD COLUMN podcast_id_uuid TEXT;
UPDATE listening_events SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = listening_events.podcast_id);
ALTER TABLE listening_events ADD COLUMN podcast_episode_db_id_uuid TEXT;
UPDATE listening_events SET podcast_episode_db_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = listening_events.podcast_episode_db_id);

-- podcast_settings (PK = podcast_id, also FK to podcasts)
ALTER TABLE podcast_settings ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_settings SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = podcast_settings.podcast_id);

-- Composite / Text-PK children whose FK columns convert:
ALTER TABLE favorites ADD COLUMN user_id_uuid TEXT;
UPDATE favorites SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = favorites.user_id);
ALTER TABLE favorites ADD COLUMN podcast_id_uuid TEXT;
UPDATE favorites SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = favorites.podcast_id);

ALTER TABLE favorite_podcast_episodes ADD COLUMN user_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = favorite_podcast_episodes.user_id);
ALTER TABLE favorite_podcast_episodes ADD COLUMN episode_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET episode_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = favorite_podcast_episodes.episode_id);

ALTER TABLE filters ADD COLUMN user_id_uuid TEXT;
UPDATE filters SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = filters.user_id);

ALTER TABLE sessions ADD COLUMN user_id_uuid TEXT;
UPDATE sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = sessions.user_id);

ALTER TABLE tags ADD COLUMN user_id_uuid TEXT;
UPDATE tags SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = tags.user_id);

ALTER TABLE tags_podcasts ADD COLUMN podcast_id_uuid TEXT;
UPDATE tags_podcasts SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = tags_podcasts.podcast_id);

ALTER TABLE playlists ADD COLUMN user_id_uuid TEXT;
UPDATE playlists SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = playlists.user_id);

ALTER TABLE playlist_items ADD COLUMN episode_uuid TEXT;
UPDATE playlist_items SET episode_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = playlist_items.episode);

ALTER TABLE podcast_episode_chapters ADD COLUMN episode_id_uuid TEXT;
UPDATE podcast_episode_chapters SET episode_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = podcast_episode_chapters.episode_id);

-- ABS user_id columns (no FK constraint existed before; keep unconstrained TEXT)
ALTER TABLE audiobookshelf_listening_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_listening_sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_listening_sessions.user_id);
ALTER TABLE audiobookshelf_media_progress ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_media_progress SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_media_progress.user_id);
ALTER TABLE audiobookshelf_playback_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_playback_sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_playback_sessions.user_id);

-- ============================================================
-- 2. Recreate each integer-PK / integer-FK table with the final
--    UUID column set. Parents are recreated before children.
-- ============================================================

-- users
CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    username VARCHAR(255) NOT NULL,
    role TEXT CHECK(role IN ('admin', 'uploader', 'user')) NOT NULL,
    password VARCHAR(255) NULL,
    explicit_consent BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    api_key VARCHAR(255),
    country TEXT,
    language TEXT,
    UNIQUE (username)
);
INSERT INTO users_new (id, username, role, password, explicit_consent, created_at, api_key, country, language)
    SELECT uuid, username, role, password, explicit_consent, created_at, api_key, country, language FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
CREATE INDEX users_api_key_idx ON users (api_key);

-- podcasts (keeps legacy_id; id INTEGER was not AUTOINCREMENT)
CREATE TABLE podcasts_new (
    id TEXT PRIMARY KEY NOT NULL,
    legacy_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    directory_id TEXT NOT NULL,
    rssfeed TEXT NOT NULL,
    image_url TEXT NOT NULL,
    summary TEXT,
    language TEXT,
    explicit TEXT,
    keywords TEXT,
    last_build_date TEXT,
    author TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    original_image_url TEXT NOT NULL DEFAULT '',
    directory_name TEXT NOT NULL DEFAULT '',
    download_location TEXT,
    guid TEXT,
    added_by TEXT REFERENCES users(id)
);
INSERT INTO podcasts_new (id, legacy_id, name, directory_id, rssfeed, image_url, summary, language, explicit, keywords, last_build_date, author, active, original_image_url, directory_name, download_location, guid, added_by)
    SELECT uuid, id, name, directory_id, rssfeed, image_url, summary, language, explicit, keywords, last_build_date, author, active, original_image_url, directory_name, download_location, guid, added_by_uuid FROM podcasts;
DROP TABLE podcasts;
ALTER TABLE podcasts_new RENAME TO podcasts;
CREATE INDEX idx_podcasts_name ON podcasts(name);
CREATE INDEX idx_podcasts_rssfeed ON podcasts(rssfeed);

-- podcast_episodes (keeps legacy_id)
CREATE TABLE podcast_episodes_new (
    id TEXT PRIMARY KEY NOT NULL,
    legacy_id INTEGER UNIQUE,
    podcast_id TEXT NOT NULL,
    episode_id TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    date_of_recording TEXT NOT NULL,
    image_url TEXT NOT NULL,
    total_time INTEGER DEFAULT 0 NOT NULL,
    description TEXT DEFAULT '' NOT NULL,
    download_time DATETIME NULL,
    guid TEXT NOT NULL DEFAULT '',
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    file_episode_path TEXT,
    file_image_path TEXT,
    episode_numbering_processed BOOLEAN NOT NULL DEFAULT FALSE,
    download_location TEXT,
    FOREIGN KEY (podcast_id) REFERENCES podcasts(id)
);
INSERT INTO podcast_episodes_new (id, legacy_id, podcast_id, episode_id, name, url, date_of_recording, image_url, total_time, description, download_time, guid, deleted, file_episode_path, file_image_path, episode_numbering_processed, download_location)
    SELECT uuid, id, podcast_id_uuid, episode_id, name, url, date_of_recording, image_url, total_time, description, download_time, guid, deleted, file_episode_path, file_image_path, episode_numbering_processed, download_location FROM podcast_episodes;
DROP TABLE podcast_episodes;
ALTER TABLE podcast_episodes_new RENAME TO podcast_episodes;
CREATE INDEX podcast_episodes_podcast_id_index ON podcast_episodes (podcast_id);
CREATE INDEX podcast_episode_url_index ON podcast_episodes (url);
CREATE INDEX idx_podcast_episodes ON podcast_episodes(podcast_id);
CREATE INDEX idx_podcast_episodes_url ON podcast_episodes(url);

-- devices
CREATE TABLE devices_new (
    id TEXT PRIMARY KEY NOT NULL,
    deviceid TEXT NOT NULL,
    kind TEXT CHECK(kind IN ('desktop', 'laptop', 'server', 'mobile', 'Other')) NOT NULL,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    chromecast_uuid TEXT,
    agent_id TEXT,
    last_seen_at TIMESTAMP,
    ip TEXT,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO devices_new (id, deviceid, kind, name, user_id, chromecast_uuid, agent_id, last_seen_at, ip)
    SELECT uuid, deviceid, kind, name, user_id_uuid, chromecast_uuid, agent_id, last_seen_at, ip FROM devices;
DROP TABLE devices;
ALTER TABLE devices_new RENAME TO devices;
CREATE INDEX idx_devices ON devices(name);
CREATE INDEX idx_devices_chromecast_uuid ON devices(chromecast_uuid);
CREATE INDEX idx_devices_agent_id ON devices(agent_id);

-- subscriptions
CREATE TABLE subscriptions_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast TEXT NOT NULL,
    created DATETIME NOT NULL,
    deleted DATETIME,
    UNIQUE (user_id, device, podcast),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO subscriptions_new (id, user_id, device, podcast, created, deleted)
    SELECT uuid, user_id_uuid, device, podcast, created, deleted FROM subscriptions;
DROP TABLE subscriptions;
ALTER TABLE subscriptions_new RENAME TO subscriptions;
CREATE INDEX idx_subscriptions ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_device ON subscriptions(device);

-- episodes
CREATE TABLE episodes_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast TEXT NOT NULL,
    episode TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    guid TEXT,
    action TEXT NOT NULL,
    started INTEGER,
    position INTEGER,
    total INTEGER,
    UNIQUE (user_id, device, podcast, episode, timestamp),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO episodes_new (id, user_id, device, podcast, episode, timestamp, guid, action, started, position, total)
    SELECT uuid, user_id_uuid, device, podcast, episode, timestamp, guid, action, started, position, total FROM episodes;
DROP TABLE episodes;
ALTER TABLE episodes_new RENAME TO episodes;
CREATE INDEX idx_episodes_podcast ON episodes(podcast);
CREATE INDEX idx_episodes_episode ON episodes(episode);

-- gpodder_settings
CREATE TABLE gpodder_settings_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    scope_id TEXT,
    data TEXT NOT NULL DEFAULT '{}',
    UNIQUE(user_id, scope, scope_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO gpodder_settings_new (id, user_id, scope, scope_id, data)
    SELECT uuid, user_id_uuid, scope, scope_id, data FROM gpodder_settings;
DROP TABLE gpodder_settings;
ALTER TABLE gpodder_settings_new RENAME TO gpodder_settings;

-- device_sync_groups
CREATE TABLE device_sync_groups_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    group_id INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    UNIQUE(user_id, device_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO device_sync_groups_new (id, user_id, group_id, device_id)
    SELECT uuid, user_id_uuid, group_id, device_id FROM device_sync_groups;
DROP TABLE device_sync_groups;
ALTER TABLE device_sync_groups_new RENAME TO device_sync_groups;

-- notifications
CREATE TABLE notifications_new (
    id TEXT PRIMARY KEY NOT NULL,
    type_of_message TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TEXT NOT NULL,
    status TEXT NOT NULL
);
INSERT INTO notifications_new (id, type_of_message, message, created_at, status)
    SELECT uuid, type_of_message, message, created_at, status FROM notifications;
DROP TABLE notifications;
ALTER TABLE notifications_new RENAME TO notifications;

-- settings (singleton)
CREATE TABLE settings_new (
    id TEXT PRIMARY KEY NOT NULL,
    auto_download BOOLEAN NOT NULL DEFAULT TRUE,
    auto_update BOOLEAN NOT NULL DEFAULT TRUE,
    auto_cleanup BOOLEAN NOT NULL DEFAULT FALSE,
    auto_cleanup_days INTEGER NOT NULL DEFAULT -1,
    podcast_prefill INTEGER DEFAULT 5 NOT NULL,
    replace_invalid_characters BOOLEAN DEFAULT TRUE NOT NULL,
    use_existing_filename BOOLEAN DEFAULT FALSE NOT NULL,
    replacement_strategy TEXT CHECK(replacement_strategy IN
        ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash')) NOT NULL DEFAULT
            'replace-with-dash-and-underscore',
    episode_format TEXT NOT NULL DEFAULT '{}',
    podcast_format TEXT NOT NULL DEFAULT '{}',
    direct_paths BOOLEAN NOT NULL DEFAULT FALSE,
    auto_transcode_opus BOOLEAN NOT NULL DEFAULT FALSE,
    use_one_cover_for_all_episodes BOOLEAN NOT NULL DEFAULT FALSE
);
INSERT INTO settings_new (id, auto_download, auto_update, auto_cleanup, auto_cleanup_days, podcast_prefill, replace_invalid_characters, use_existing_filename, replacement_strategy, episode_format, podcast_format, direct_paths, auto_transcode_opus, use_one_cover_for_all_episodes)
    SELECT uuid, auto_download, auto_update, auto_cleanup, auto_cleanup_days, podcast_prefill, replace_invalid_characters, use_existing_filename, replacement_strategy, episode_format, podcast_format, direct_paths, auto_transcode_opus, use_one_cover_for_all_episodes FROM settings;
DROP TABLE settings;
ALTER TABLE settings_new RENAME TO settings;

-- listening_events
CREATE TABLE listening_events_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast_episode_id TEXT NOT NULL,
    podcast_id TEXT NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    podcast_episode_db_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    delta_seconds INTEGER NOT NULL,
    start_position INTEGER NOT NULL,
    end_position INTEGER NOT NULL,
    listened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO listening_events_new (id, user_id, device, podcast_episode_id, podcast_id, podcast_episode_db_id, delta_seconds, start_position, end_position, listened_at)
    SELECT uuid, user_id_uuid, device, podcast_episode_id, podcast_id_uuid, podcast_episode_db_id_uuid, delta_seconds, start_position, end_position, listened_at FROM listening_events;
DROP TABLE listening_events;
ALTER TABLE listening_events_new RENAME TO listening_events;
CREATE INDEX idx_listening_events_user_id_time ON listening_events (user_id, listened_at);
CREATE INDEX idx_listening_events_user_id_episode ON listening_events (user_id, podcast_episode_id);
CREATE INDEX idx_listening_events_user_id_podcast ON listening_events (user_id, podcast_id);

-- podcast_settings (PK = podcast_id, also FK to podcasts)
CREATE TABLE podcast_settings_new (
    podcast_id TEXT PRIMARY KEY NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    episode_numbering BOOLEAN NOT NULL DEFAULT FALSE,
    auto_download BOOLEAN NOT NULL DEFAULT FALSE,
    auto_update BOOLEAN NOT NULL DEFAULT TRUE,
    auto_cleanup BOOLEAN NOT NULL DEFAULT FALSE,
    auto_cleanup_days INTEGER NOT NULL DEFAULT -1,
    replace_invalid_characters BOOLEAN DEFAULT TRUE NOT NULL,
    use_existing_filename BOOLEAN DEFAULT FALSE NOT NULL,
    replacement_strategy TEXT CHECK(replacement_strategy IN
            ('replace-with-dash-and-underscore', 'remove', 'replace-with-dash')) NOT NULL DEFAULT
                'replace-with-dash-and-underscore',
    episode_format TEXT NOT NULL DEFAULT '{}',
    podcast_format TEXT NOT NULL DEFAULT '{}',
    direct_paths BOOLEAN NOT NULL DEFAULT FALSE,
    activated BOOLEAN NOT NULL DEFAULT FALSE,
    podcast_prefill INTEGER NOT NULL DEFAULT 0,
    use_one_cover_for_all_episodes BOOLEAN NOT NULL DEFAULT FALSE
);
INSERT INTO podcast_settings_new (podcast_id, episode_numbering, auto_download, auto_update, auto_cleanup, auto_cleanup_days, replace_invalid_characters, use_existing_filename, replacement_strategy, episode_format, podcast_format, direct_paths, activated, podcast_prefill, use_one_cover_for_all_episodes)
    SELECT podcast_id_uuid, episode_numbering, auto_download, auto_update, auto_cleanup, auto_cleanup_days, replace_invalid_characters, use_existing_filename, replacement_strategy, episode_format, podcast_format, direct_paths, activated, podcast_prefill, use_one_cover_for_all_episodes FROM podcast_settings;
DROP TABLE podcast_settings;
ALTER TABLE podcast_settings_new RENAME TO podcast_settings;

-- favorites (composite PK)
CREATE TABLE favorites_new (
    user_id TEXT NOT NULL,
    podcast_id TEXT NOT NULL,
    favored BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (user_id, podcast_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (podcast_id) REFERENCES podcasts (id) ON DELETE CASCADE
);
INSERT INTO favorites_new (user_id, podcast_id, favored)
    SELECT user_id_uuid, podcast_id_uuid, favored FROM favorites;
DROP TABLE favorites;
ALTER TABLE favorites_new RENAME TO favorites;

-- favorite_podcast_episodes (composite PK)
CREATE TABLE favorite_podcast_episodes_new (
    user_id TEXT NOT NULL,
    episode_id TEXT NOT NULL,
    favorite BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (user_id, episode_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id)
);
INSERT INTO favorite_podcast_episodes_new (user_id, episode_id, favorite)
    SELECT user_id_uuid, episode_id_uuid, favorite FROM favorite_podcast_episodes;
DROP TABLE favorite_podcast_episodes;
ALTER TABLE favorite_podcast_episodes_new RENAME TO favorite_podcast_episodes;

-- filters (PK = user_id)
CREATE TABLE filters_new (
    user_id TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    ascending BOOLEAN DEFAULT FALSE NOT NULL,
    filter TEXT CHECK(filter IN ('PublishedDate', 'Title')),
    only_favored BOOLEAN DEFAULT FALSE NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO filters_new (user_id, title, ascending, filter, only_favored)
    SELECT user_id_uuid, title, ascending, filter, only_favored FROM filters;
DROP TABLE filters;
ALTER TABLE filters_new RENAME TO filters;

-- sessions (composite PK)
CREATE TABLE sessions_new (
    user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    session_id TEXT NOT NULL,
    expires DATETIME NOT NULL,
    PRIMARY KEY (user_id, session_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO sessions_new (user_id, username, session_id, expires)
    SELECT user_id_uuid, username, session_id, expires FROM sessions;
DROP TABLE sessions;
ALTER TABLE sessions_new RENAME TO sessions;

-- tags (TEXT PK unchanged; user_id converts)
CREATE TABLE tags_new (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL,
    color TEXT NOT NULL,
    UNIQUE (name, user_id),
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);
INSERT INTO tags_new (id, name, user_id, description, created_at, color)
    SELECT id, name, user_id_uuid, description, created_at, color FROM tags;
DROP TABLE tags;
ALTER TABLE tags_new RENAME TO tags;
CREATE INDEX idx_tags_name ON tags (name);
CREATE INDEX idx_tags_user_id ON tags (user_id);

-- tags_podcasts (composite PK; tag_id unchanged, podcast_id converts)
CREATE TABLE tags_podcasts_new (
    tag_id TEXT NOT NULL,
    podcast_id TEXT NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES tags (id),
    FOREIGN KEY (podcast_id) REFERENCES podcasts (id),
    PRIMARY KEY (tag_id, podcast_id)
);
INSERT INTO tags_podcasts_new (tag_id, podcast_id)
    SELECT tag_id, podcast_id_uuid FROM tags_podcasts;
DROP TABLE tags_podcasts;
ALTER TABLE tags_podcasts_new RENAME TO tags_podcasts;

-- playlists (TEXT PK unchanged; user_id converts)
CREATE TABLE playlists_new (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL
);
INSERT INTO playlists_new (id, name, user_id)
    SELECT id, name, user_id_uuid FROM playlists;
DROP TABLE playlists;
ALTER TABLE playlists_new RENAME TO playlists;

-- playlist_items (composite PK; playlist_id unchanged, episode converts)
CREATE TABLE playlist_items_new (
    playlist_id TEXT NOT NULL,
    episode TEXT NOT NULL,
    position INTEGER NOT NULL,
    FOREIGN KEY (playlist_id) REFERENCES playlists(id),
    FOREIGN KEY (episode) REFERENCES podcast_episodes(id),
    PRIMARY KEY (playlist_id, episode)
);
INSERT INTO playlist_items_new (playlist_id, episode, position)
    SELECT playlist_id, episode_uuid, position FROM playlist_items;
DROP TABLE playlist_items;
ALTER TABLE playlist_items_new RENAME TO playlist_items;

-- podcast_episode_chapters (TEXT PK unchanged; episode_id converts)
CREATE TABLE podcast_episode_chapters_new (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    href TEXT,
    image TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
INSERT INTO podcast_episode_chapters_new (id, episode_id, title, start_time, end_time, href, image, created_at, updated_at)
    SELECT id, episode_id_uuid, title, start_time, end_time, href, image, created_at, updated_at FROM podcast_episode_chapters;
DROP TABLE podcast_episode_chapters;
ALTER TABLE podcast_episode_chapters_new RENAME TO podcast_episode_chapters;
CREATE UNIQUE INDEX uq_podcast_episode_chapters_episode_start
    ON podcast_episode_chapters (episode_id, start_time);

-- audiobookshelf_playback_sessions (TEXT PK unchanged; user_id retyped, no FK)
CREATE TABLE audiobookshelf_playback_sessions_new (
    id TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    library_id TEXT,
    library_item_id TEXT NOT NULL,
    episode_id TEXT,
    media_type TEXT NOT NULL,
    play_method INTEGER NOT NULL DEFAULT 0,
    position_seconds REAL NOT NULL DEFAULT 0,
    duration REAL NOT NULL DEFAULT 0,
    started_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    finished_at TIMESTAMP,
    time_listening_total REAL NOT NULL DEFAULT 0,
    display_title TEXT,
    display_author TEXT,
    cover_path TEXT,
    media_metadata_json TEXT,
    device_info_json TEXT
);
INSERT INTO audiobookshelf_playback_sessions_new (id, user_id, library_id, library_item_id, episode_id, media_type, play_method, position_seconds, duration, started_at, updated_at, finished_at, time_listening_total, display_title, display_author, cover_path, media_metadata_json, device_info_json)
    SELECT id, user_id_uuid, library_id, library_item_id, episode_id, media_type, play_method, position_seconds, duration, started_at, updated_at, finished_at, time_listening_total, display_title, display_author, cover_path, media_metadata_json, device_info_json FROM audiobookshelf_playback_sessions;
DROP TABLE audiobookshelf_playback_sessions;
ALTER TABLE audiobookshelf_playback_sessions_new RENAME TO audiobookshelf_playback_sessions;
CREATE INDEX audiobookshelf_playback_sessions_user_idx
    ON audiobookshelf_playback_sessions (user_id);

-- audiobookshelf_media_progress (TEXT PK unchanged; user_id retyped, no FK)
CREATE TABLE audiobookshelf_media_progress_new (
    id TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    library_item_id TEXT NOT NULL,
    episode_id TEXT,
    media_type TEXT NOT NULL,
    duration REAL NOT NULL DEFAULT 0,
    position_seconds REAL NOT NULL DEFAULT 0,
    progress REAL NOT NULL DEFAULT 0,
    is_finished BOOLEAN NOT NULL DEFAULT 0,
    hide_from_continue_listening BOOLEAN NOT NULL DEFAULT 0,
    last_update TIMESTAMP NOT NULL,
    started_at TIMESTAMP NOT NULL,
    finished_at TIMESTAMP
);
INSERT INTO audiobookshelf_media_progress_new (id, user_id, library_item_id, episode_id, media_type, duration, position_seconds, progress, is_finished, hide_from_continue_listening, last_update, started_at, finished_at)
    SELECT id, user_id_uuid, library_item_id, episode_id, media_type, duration, position_seconds, progress, is_finished, hide_from_continue_listening, last_update, started_at, finished_at FROM audiobookshelf_media_progress;
DROP TABLE audiobookshelf_media_progress;
ALTER TABLE audiobookshelf_media_progress_new RENAME TO audiobookshelf_media_progress;
CREATE UNIQUE INDEX audiobookshelf_media_progress_user_item_episode_idx
    ON audiobookshelf_media_progress (user_id, library_item_id, COALESCE(episode_id, ''));
CREATE INDEX audiobookshelf_media_progress_user_idx
    ON audiobookshelf_media_progress (user_id);

-- audiobookshelf_listening_sessions (TEXT PK unchanged; user_id retyped, no FK)
CREATE TABLE audiobookshelf_listening_sessions_new (
    id TEXT NOT NULL PRIMARY KEY,
    user_id TEXT NOT NULL,
    library_id TEXT,
    library_item_id TEXT NOT NULL,
    episode_id TEXT,
    media_type TEXT NOT NULL,
    duration REAL NOT NULL DEFAULT 0,
    position_seconds REAL NOT NULL DEFAULT 0,
    time_listening REAL NOT NULL DEFAULT 0,
    play_method INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    display_title TEXT,
    display_author TEXT,
    cover_path TEXT
);
INSERT INTO audiobookshelf_listening_sessions_new (id, user_id, library_id, library_item_id, episode_id, media_type, duration, position_seconds, time_listening, play_method, started_at, updated_at, display_title, display_author, cover_path)
    SELECT id, user_id_uuid, library_id, library_item_id, episode_id, media_type, duration, position_seconds, time_listening, play_method, started_at, updated_at, display_title, display_author, cover_path FROM audiobookshelf_listening_sessions;
DROP TABLE audiobookshelf_listening_sessions;
ALTER TABLE audiobookshelf_listening_sessions_new RENAME TO audiobookshelf_listening_sessions;
CREATE INDEX audiobookshelf_listening_sessions_user_idx
    ON audiobookshelf_listening_sessions (user_id);

PRAGMA foreign_keys = ON;
