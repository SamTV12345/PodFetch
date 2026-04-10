CREATE TABLE device_sync_groups (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    group_id INTEGER NOT NULL,
    device_id TEXT NOT NULL,
    UNIQUE(username, device_id)
);

CREATE TABLE gpodder_settings (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL,
    scope TEXT NOT NULL,
    scope_id TEXT,
    data TEXT NOT NULL DEFAULT '{}',
    UNIQUE(username, scope, scope_id)
);
