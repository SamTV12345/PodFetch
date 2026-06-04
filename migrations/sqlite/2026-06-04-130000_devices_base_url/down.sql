PRAGMA foreign_keys = OFF;

CREATE TABLE devices_old (
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

INSERT INTO devices_old (id, deviceid, kind, name, user_id, chromecast_uuid, agent_id, last_seen_at, ip)
    SELECT id, deviceid, kind, name, user_id, chromecast_uuid, agent_id, last_seen_at, ip FROM devices;

DROP TABLE devices;
ALTER TABLE devices_old RENAME TO devices;

CREATE INDEX idx_devices ON devices(name);
CREATE INDEX idx_devices_chromecast_uuid ON devices(chromecast_uuid);
CREATE INDEX idx_devices_agent_id ON devices(agent_id);

PRAGMA foreign_keys = ON;
