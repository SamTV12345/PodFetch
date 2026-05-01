ALTER TABLE devices ADD COLUMN chromecast_uuid TEXT;
ALTER TABLE devices ADD COLUMN agent_id TEXT;
ALTER TABLE devices ADD COLUMN last_seen_at TIMESTAMP;
ALTER TABLE devices ADD COLUMN ip TEXT;

CREATE INDEX idx_devices_chromecast_uuid ON devices(chromecast_uuid);
CREATE INDEX idx_devices_agent_id ON devices(agent_id);
