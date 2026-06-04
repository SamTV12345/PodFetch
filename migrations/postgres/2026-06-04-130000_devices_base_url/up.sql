ALTER TABLE devices ADD COLUMN base_url TEXT;
ALTER TABLE devices DROP CONSTRAINT IF EXISTS devices_kind_check;
