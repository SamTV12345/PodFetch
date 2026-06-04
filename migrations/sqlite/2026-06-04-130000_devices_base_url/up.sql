-- Rebuild `devices`: add base_url and drop the restrictive kind CHECK so
-- application-defined kinds (chromecast_*, mopidy_*) are accepted. Kinds are
-- validated in podfetch_domain::device::kind, not in the DB. SQLite cannot
-- alter a CHECK in place, so the table is recreated. Runs outside a
-- transaction (see metadata.toml) so the foreign_keys pragma toggle applies.
PRAGMA foreign_keys = OFF;

CREATE TABLE devices_new (
    id TEXT PRIMARY KEY NOT NULL,
    deviceid TEXT NOT NULL,
    kind TEXT NOT NULL,
    name TEXT NOT NULL,
    user_id TEXT NOT NULL,
    chromecast_uuid TEXT,
    agent_id TEXT,
    last_seen_at TIMESTAMP,
    ip TEXT,
    base_url TEXT,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

INSERT INTO devices_new (id, deviceid, kind, name, user_id, chromecast_uuid, agent_id, last_seen_at, ip)
    SELECT id, deviceid, kind, name, user_id, chromecast_uuid, agent_id, last_seen_at, ip FROM devices;

DROP TABLE devices;
ALTER TABLE devices_new RENAME TO devices;

CREATE INDEX idx_devices ON devices(name);
CREATE INDEX idx_devices_chromecast_uuid ON devices(chromecast_uuid);
CREATE INDEX idx_devices_agent_id ON devices(agent_id);

PRAGMA foreign_keys = ON;
