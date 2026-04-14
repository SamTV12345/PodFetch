-- Reverse migration: Convert user_id back to username
PRAGMA foreign_keys = OFF;

-- 1. favorites
CREATE TABLE favorites_old (
    username TEXT NOT NULL,
    podcast_id INTEGER NOT NULL,
    favored BOOLEAN NOT NULL DEFAULT 0,
    PRIMARY KEY (username, podcast_id),
    FOREIGN KEY (username) REFERENCES users (username) ON DELETE CASCADE,
    FOREIGN KEY (podcast_id) REFERENCES podcasts (id) ON DELETE CASCADE
);
INSERT INTO favorites_old (username, podcast_id, favored)
    SELECT u.username, f.podcast_id, f.favored
    FROM favorites f INNER JOIN users u ON f.user_id = u.id;
DROP TABLE favorites;
ALTER TABLE favorites_old RENAME TO favorites;

-- 2. filters
CREATE TABLE filters_old (
    username TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    ascending BOOLEAN DEFAULT FALSE NOT NULL,
    filter TEXT CHECK(filter IN ('PublishedDate', 'Title')),
    only_favored BOOLEAN DEFAULT FALSE NOT NULL
);
INSERT INTO filters_old (username, title, ascending, filter, only_favored)
    SELECT u.username, f.title, f.ascending, f.filter, f.only_favored
    FROM filters f INNER JOIN users u ON f.user_id = u.id;
DROP TABLE filters;
ALTER TABLE filters_old RENAME TO filters;

-- 3. sessions
CREATE TABLE sessions_old (
    username TEXT NOT NULL,
    session_id TEXT NOT NULL,
    expires DATETIME NOT NULL,
    PRIMARY KEY (username, session_id)
);
INSERT INTO sessions_old (username, session_id, expires)
    SELECT s.username, s.session_id, s.expires
    FROM sessions s;
DROP TABLE sessions;
ALTER TABLE sessions_old RENAME TO sessions;

-- 4. favorite_podcast_episodes
CREATE TABLE favorite_podcast_episodes_old (
    username TEXT NOT NULL,
    episode_id INTEGER NOT NULL,
    favorite BOOLEAN NOT NULL DEFAULT FALSE,
    PRIMARY KEY (username, episode_id),
    FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id)
);
INSERT INTO favorite_podcast_episodes_old (username, episode_id, favorite)
    SELECT u.username, fpe.episode_id, fpe.favorite
    FROM favorite_podcast_episodes fpe INNER JOIN users u ON fpe.user_id = u.id;
DROP TABLE favorite_podcast_episodes;
ALTER TABLE favorite_podcast_episodes_old RENAME TO favorite_podcast_episodes;

-- 5. subscriptions
DROP INDEX IF EXISTS idx_subscriptions;
DROP INDEX IF EXISTS idx_subscriptions_device;
CREATE TABLE subscriptions_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast TEXT NOT NULL,
    created DATETIME NOT NULL,
    deleted DATETIME,
    UNIQUE (username, device, podcast)
);
INSERT INTO subscriptions_old (id, username, device, podcast, created, deleted)
    SELECT s.id, u.username, s.device, s.podcast, s.created, s.deleted
    FROM subscriptions s INNER JOIN users u ON s.user_id = u.id;
DROP TABLE subscriptions;
ALTER TABLE subscriptions_old RENAME TO subscriptions;
CREATE INDEX idx_subscriptions ON subscriptions(username);
CREATE INDEX idx_subscriptions_device ON subscriptions(device);

-- 6. devices
DROP INDEX IF EXISTS idx_devices;
CREATE TABLE devices_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    deviceid TEXT NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    username TEXT NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username)
);
INSERT INTO devices_old (id, deviceid, kind, name, username)
    SELECT d.id, d.deviceid, d.kind, d.name, u.username
    FROM devices d INNER JOIN users u ON d.user_id = u.id;
DROP TABLE devices;
ALTER TABLE devices_old RENAME TO devices;
CREATE INDEX idx_devices ON devices(name);

-- 7. episodes
DROP INDEX IF EXISTS idx_episodes_podcast;
DROP INDEX IF EXISTS idx_episodes_episode;
CREATE TABLE episodes_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast TEXT NOT NULL,
    episode TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    guid TEXT,
    action TEXT NOT NULL,
    started INTEGER,
    position INTEGER,
    total INTEGER,
    UNIQUE (username, device, podcast, episode, timestamp)
);
INSERT INTO episodes_old (id, username, device, podcast, episode, timestamp, guid, action, started, position, total)
    SELECT e.id, u.username, e.device, e.podcast, e.episode, e.timestamp, e.guid, e.action, e.started, e.position, e.total
    FROM episodes e INNER JOIN users u ON e.user_id = u.id;
DROP TABLE episodes;
ALTER TABLE episodes_old RENAME TO episodes;
CREATE INDEX idx_episodes_podcast ON episodes(podcast);
CREATE INDEX idx_episodes_episode ON episodes(episode);

-- 8. tags
DROP INDEX IF EXISTS idx_tags_name;
DROP INDEX IF EXISTS idx_tags_user_id;
CREATE TABLE tags_old (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    username TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL,
    color TEXT NOT NULL,
    UNIQUE (name, username)
);
INSERT INTO tags_old (id, name, username, description, created_at, color)
    SELECT t.id, t.name, u.username, t.description, t.created_at, t.color
    FROM tags t INNER JOIN users u ON t.user_id = u.id;
DROP TABLE tags;
ALTER TABLE tags_old RENAME TO tags;
CREATE INDEX idx_tags_name ON tags (name);
CREATE INDEX idx_tags_username ON tags (username);

-- 9. listening_events
DROP INDEX IF EXISTS idx_listening_events_user_id_time;
DROP INDEX IF EXISTS idx_listening_events_user_id_episode;
DROP INDEX IF EXISTS idx_listening_events_user_id_podcast;
CREATE TABLE listening_events_old (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    username TEXT NOT NULL,
    device TEXT NOT NULL,
    podcast_episode_id TEXT NOT NULL,
    podcast_id INTEGER NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    podcast_episode_db_id INTEGER NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    delta_seconds INTEGER NOT NULL,
    start_position INTEGER NOT NULL,
    end_position INTEGER NOT NULL,
    listened_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO listening_events_old (id, username, device, podcast_episode_id, podcast_id, podcast_episode_db_id, delta_seconds, start_position, end_position, listened_at)
    SELECT le.id, u.username, le.device, le.podcast_episode_id, le.podcast_id, le.podcast_episode_db_id, le.delta_seconds, le.start_position, le.end_position, le.listened_at
    FROM listening_events le INNER JOIN users u ON le.user_id = u.id;
DROP TABLE listening_events;
ALTER TABLE listening_events_old RENAME TO listening_events;
CREATE INDEX idx_listening_events_username_time ON listening_events (username, listened_at);
CREATE INDEX idx_listening_events_username_episode ON listening_events (username, podcast_episode_id);
CREATE INDEX idx_listening_events_username_podcast ON listening_events (username, podcast_id);

-- 10. device_sync_groups
CREATE TABLE device_sync_groups_old (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    group_id INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    UNIQUE(username, device_id)
);
INSERT INTO device_sync_groups_old (id, username, group_id, device_id)
    SELECT dsg.id, u.username, dsg.group_id, dsg.device_id
    FROM device_sync_groups dsg INNER JOIN users u ON dsg.user_id = u.id;
DROP TABLE device_sync_groups;
ALTER TABLE device_sync_groups_old RENAME TO device_sync_groups;

-- 11. gpodder_settings
CREATE TABLE gpodder_settings_old (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    scope TEXT NOT NULL,
    scope_id TEXT,
    data TEXT NOT NULL DEFAULT '{}',
    UNIQUE(username, scope, scope_id)
);
INSERT INTO gpodder_settings_old (id, username, scope, scope_id, data)
    SELECT gs.id, u.username, gs.scope, gs.scope_id, gs.data
    FROM gpodder_settings gs INNER JOIN users u ON gs.user_id = u.id;
DROP TABLE gpodder_settings;
ALTER TABLE gpodder_settings_old RENAME TO gpodder_settings;

-- 12. podcasts.added_by (INTEGER -> TEXT)
CREATE TABLE podcasts_old (
    id INTEGER PRIMARY KEY NOT NULL,
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
    added_by TEXT
);
INSERT INTO podcasts_old (id, name, directory_id, rssfeed, image_url, summary, language, explicit, keywords, last_build_date, author, active, original_image_url, directory_name, download_location, guid, added_by)
    SELECT p.id, p.name, p.directory_id, p.rssfeed, p.image_url, p.summary, p.language, p.explicit, p.keywords, p.last_build_date, p.author, p.active, p.original_image_url, p.directory_name, p.download_location, p.guid,
           (SELECT u.username FROM users u WHERE u.id = p.added_by)
    FROM podcasts p;
DROP TABLE podcasts;
ALTER TABLE podcasts_old RENAME TO podcasts;
CREATE INDEX IF NOT EXISTS idx_podcasts_name ON podcasts(name);
CREATE INDEX IF NOT EXISTS idx_podcasts_rssfeed ON podcasts(rssfeed);
PRAGMA foreign_keys = ON;
