//! Regression test: the full sqlite migration chain (including the uuid
//! primary-keys migration) must apply cleanly on a FRESH EMPTY database via the
//! real diesel runner with foreign keys ON (matching the production connection
//! customizer / test harness), and re-running must be a no-op. Guards against
//! the table-rebuild + `run_in_transaction = false` migration regressing on a
//! clean install.
#![cfg(feature = "sqlite")]

use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel::Connection;
use diesel_migrations::MigrationHarness;
use podfetch_persistence::db::SQLITE_MIGRATIONS;

#[test]
fn fresh_chain_applies_cleanly() {
    let tmp = std::env::temp_dir().join("podfetch_fresh_chain.db");
    let _ = std::fs::remove_file(&tmp);
    let _ = std::fs::remove_file(tmp.with_extension("db-wal"));
    let _ = std::fs::remove_file(tmp.with_extension("db-shm"));

    let url = tmp.to_str().unwrap();
    let mut conn = SqliteConnection::establish(url).expect("establish");
    conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .unwrap();

    conn.run_pending_migrations(SQLITE_MIGRATIONS)
        .expect("fresh migrations apply cleanly");

    // Running again must be a no-op (all recorded), proving idempotency.
    conn.run_pending_migrations(SQLITE_MIGRATIONS)
        .expect("second run is a no-op");

    let _ = std::fs::remove_file(&tmp);
    let _ = std::fs::remove_file(tmp.with_extension("db-wal"));
    let _ = std::fs::remove_file(tmp.with_extension("db-shm"));
}
