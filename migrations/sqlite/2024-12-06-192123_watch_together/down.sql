-- This file should undo anything in `up.sql`

DROP TABLE watch_togethers;
DROP TABLE watch_together_users;
ALTER TABLE settings DROP COLUMN jwt_key;
DROP TABLE watch_together_users_to_room_mappings;