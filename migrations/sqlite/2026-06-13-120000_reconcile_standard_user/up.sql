-- Reconcile the no-auth standard user (`user123`) onto its canonical id.
--
-- The uuid primary-keys migration reassigned EVERY user a random uuid, so on
-- upgraded databases the seeded standard user no longer carries the canonical
-- STANDARD_USER_ID (`00000000-0000-7000-8000-000000009999`). Startup then fails
-- to find it by id, tries to re-insert `user123`, and hits the username unique
-- constraint. Move the existing row (and everything it owns) onto the canonical
-- id so the fixed-id invariant the app relies on holds after an upgrade too.
--
-- Foreign keys are switched off for the rebuild: children are repointed to the
-- canonical id before the parent row exists under it, so enforcement must be
-- relaxed (and the PRAGMA only takes effect outside a transaction -- hence
-- run_in_transaction = false in metadata.toml). Every child is repointed before
-- the parent row is moved, so the `username = 'user123'` lookup still resolves
-- to the old random id throughout.
PRAGMA foreign_keys = OFF;

UPDATE device_sync_groups SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE devices SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE episodes SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE favorite_podcast_episodes SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE favorites SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE filters SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE gpodder_settings SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE listening_events SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE playlists SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE podcasts SET added_by = '00000000-0000-7000-8000-000000009999'
  WHERE added_by = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE sessions SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE subscriptions SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE tags SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE sponsorblock_user_settings SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE audiobookshelf_listening_sessions SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE audiobookshelf_media_progress SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');
UPDATE audiobookshelf_playback_sessions SET user_id = '00000000-0000-7000-8000-000000009999'
  WHERE user_id = (SELECT id FROM users WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999');

UPDATE users SET id = '00000000-0000-7000-8000-000000009999'
  WHERE username = 'user123' AND id <> '00000000-0000-7000-8000-000000009999';

PRAGMA foreign_keys = ON;
