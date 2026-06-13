-- Forward-only migration: the random uuid the standard user previously carried
-- cannot be reconstructed. Restore from a database backup taken before the
-- upgrade if a rollback is required.
SELECT 1;
