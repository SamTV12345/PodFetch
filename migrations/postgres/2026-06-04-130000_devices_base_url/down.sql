ALTER TABLE devices ADD CONSTRAINT devices_kind_check CHECK (kind IN ('desktop', 'laptop', 'server', 'mobile', 'Other'));
ALTER TABLE devices DROP COLUMN base_url;
