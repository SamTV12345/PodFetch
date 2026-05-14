CREATE TABLE audiobookshelf_libraries (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    media_type TEXT NOT NULL,
    icon TEXT NOT NULL DEFAULT 'database',
    display_order INTEGER NOT NULL DEFAULT 1,
    folder_paths TEXT NOT NULL DEFAULT '[]',
    metadata_precedence TEXT NOT NULL DEFAULT '[]',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE audiobookshelf_playback_sessions (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
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

CREATE INDEX audiobookshelf_playback_sessions_user_idx
    ON audiobookshelf_playback_sessions (user_id);

CREATE TABLE audiobookshelf_media_progress (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
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

CREATE UNIQUE INDEX audiobookshelf_media_progress_user_item_episode_idx
    ON audiobookshelf_media_progress (user_id, library_item_id, COALESCE(episode_id, ''));

CREATE INDEX audiobookshelf_media_progress_user_idx
    ON audiobookshelf_media_progress (user_id);

CREATE TABLE audiobookshelf_listening_sessions (
    id TEXT NOT NULL PRIMARY KEY,
    user_id INTEGER NOT NULL,
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

CREATE INDEX audiobookshelf_listening_sessions_user_idx
    ON audiobookshelf_listening_sessions (user_id);
