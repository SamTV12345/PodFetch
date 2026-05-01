DROP INDEX IF EXISTS idx_devices_agent_id;
DROP INDEX IF EXISTS idx_devices_chromecast_uuid;

ALTER TABLE devices DROP COLUMN ip;
ALTER TABLE devices DROP COLUMN last_seen_at;
ALTER TABLE devices DROP COLUMN agent_id;
ALTER TABLE devices DROP COLUMN chromecast_uuid;
