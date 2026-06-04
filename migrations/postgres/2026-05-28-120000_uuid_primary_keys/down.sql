-- Forward-only migration: original integer ids for non-legacy tables cannot be
-- reconstructed. Restore from a database backup taken before the upgrade.
-- See the sqlite down.sql for the same note.
SELECT 1;
