-- Fix sqlite_sequence drift introduced by 2026-04-15-100000_username_to_user_id.
-- That migration recreated AUTOINCREMENT tables by copying rows with explicit ids,
-- then renamed *_new -> real names. sqlite_sequence rows could be left under the
-- *_new name or be missing entirely, so the next AUTOINCREMENT collides with
-- copied ids (e.g. UNIQUE constraint failed: subscriptions.id, issue #2015).
--
-- For each affected AUTOINCREMENT table: drop the orphan *_new row, then upsert
-- the seq under the real name to MAX(id) (or 0 if empty). Idempotent.

-- subscriptions
DELETE FROM sqlite_sequence WHERE name = 'subscriptions_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'subscriptions', COALESCE((SELECT MAX(id) FROM subscriptions), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'subscriptions');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM subscriptions), 0)
WHERE name = 'subscriptions';

-- episodes
DELETE FROM sqlite_sequence WHERE name = 'episodes_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'episodes', COALESCE((SELECT MAX(id) FROM episodes), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'episodes');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM episodes), 0)
WHERE name = 'episodes';

-- devices
DELETE FROM sqlite_sequence WHERE name = 'devices_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'devices', COALESCE((SELECT MAX(id) FROM devices), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'devices');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM devices), 0)
WHERE name = 'devices';

-- listening_events
DELETE FROM sqlite_sequence WHERE name = 'listening_events_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'listening_events', COALESCE((SELECT MAX(id) FROM listening_events), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'listening_events');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM listening_events), 0)
WHERE name = 'listening_events';

-- device_sync_groups
DELETE FROM sqlite_sequence WHERE name = 'device_sync_groups_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'device_sync_groups', COALESCE((SELECT MAX(id) FROM device_sync_groups), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'device_sync_groups');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM device_sync_groups), 0)
WHERE name = 'device_sync_groups';

-- gpodder_settings
DELETE FROM sqlite_sequence WHERE name = 'gpodder_settings_new';
INSERT INTO sqlite_sequence (name, seq)
SELECT 'gpodder_settings', COALESCE((SELECT MAX(id) FROM gpodder_settings), 0)
WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name = 'gpodder_settings');
UPDATE sqlite_sequence
SET seq = COALESCE((SELECT MAX(id) FROM gpodder_settings), 0)
WHERE name = 'gpodder_settings';
