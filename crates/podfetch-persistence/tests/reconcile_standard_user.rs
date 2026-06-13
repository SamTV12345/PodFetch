//! Regression test for the no-auth standard-user startup crash.
//!
//! The uuid primary-keys migration reassigned EVERY user a random uuid
//! (`gen_random_uuid()` / `randomblob`), including the seeded no-auth standard
//! user `user123`. On the next startup `ensure_standard_user_present()` looked
//! the user up by the canonical id, missed it, and tried to insert a fresh
//! `user123` -> `users_username_key` duplicate -> panic.
//!
//! The reconcile migration must move the existing `user123` row (and every row
//! it owns) onto the canonical [`STANDARD_USER_ID`] so the fixed-id invariant
//! the app relies on holds on upgraded databases too.
#![cfg(feature = "sqlite")]

use diesel::connection::SimpleConnection;
use diesel::sqlite::SqliteConnection;
use diesel::{Connection, QueryableByName, RunQueryDsl};
use diesel_migrations::MigrationHarness;
use podfetch_persistence::db::SQLITE_MIGRATIONS;

/// Canonical `STANDARD_USER_ID` (`crate::role::STANDARD_USER_ID`) in text form.
const CANON: &str = "00000000-0000-7000-8000-000000009999";
/// A random uuid standing in for what the uuid migration handed the user.
const RANDOM: &str = "11111111-2222-4333-8444-555555555555";

#[derive(QueryableByName)]
struct TextRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    val: String,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
}

fn fresh_conn(name: &str) -> (SqliteConnection, std::path::PathBuf) {
    let tmp = std::env::temp_dir().join(name);
    for ext in ["db", "db-wal", "db-shm"] {
        let _ = std::fs::remove_file(tmp.with_extension(ext));
    }
    let mut conn = SqliteConnection::establish(tmp.to_str().unwrap()).expect("establish");
    conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .unwrap();
    (conn, tmp)
}

fn cleanup(tmp: &std::path::Path) {
    for ext in ["db", "db-wal", "db-shm"] {
        let _ = std::fs::remove_file(tmp.with_extension(ext));
    }
}

/// Apply every migration EXCEPT the newest one (the reconcile migration), so a
/// test can plant a pre-reconcile database state, then return that last
/// migration ready to apply.
fn apply_through_pre_reconcile(
    conn: &mut SqliteConnection,
) -> Box<dyn diesel::migration::Migration<diesel::sqlite::Sqlite>> {
    let pending = conn.pending_migrations(SQLITE_MIGRATIONS).expect("pending");
    let (reconcile, earlier) = pending.split_last().expect("at least one migration");
    assert!(
        reconcile.name().to_string().contains("reconcile_standard_user"),
        "newest migration must be the reconcile migration, got `{}`",
        reconcile.name()
    );
    for m in earlier {
        conn.run_migration(m).expect("apply earlier migration");
    }
    // `pending` is consumed below, so hand back the reconcile migration boxed.
    let pending = conn.pending_migrations(SQLITE_MIGRATIONS).expect("pending");
    pending.into_iter().next_back().expect("reconcile migration")
}

fn fk_violations(conn: &mut SqliteConnection) -> i64 {
    diesel::sql_query("SELECT COUNT(*) AS n FROM pragma_foreign_key_check")
        .load::<CountRow>(conn)
        .unwrap()[0]
        .n
}

#[test]
fn reconciles_legacy_standard_user_to_canonical_id() {
    let (mut conn, tmp) = fresh_conn("podfetch_reconcile_std_user.db");
    let reconcile = apply_through_pre_reconcile(&mut conn);

    // Plant the bug: `user123` carrying a RANDOM uuid plus an FK-bound row it
    // owns; the canonical id is absent.
    conn.batch_execute(&format!(
        "INSERT INTO users (id, username, role, explicit_consent, created_at) \
           VALUES ('{RANDOM}', 'user123', 'admin', 1, '2026-01-01T00:00:00');\
         INSERT INTO filters (user_id, ascending, only_favored) \
           VALUES ('{RANDOM}', 0, 0);"
    ))
    .expect("plant pre-reconcile state");

    conn.run_migration(&*reconcile).expect("apply reconcile migration");

    // The standard user now carries the canonical id.
    let users: Vec<TextRow> =
        diesel::sql_query("SELECT id AS val FROM users WHERE username = 'user123'")
            .load(&mut conn)
            .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].val, CANON, "standard user must move to the canonical id");

    // The random id is gone.
    let leftover: Vec<CountRow> =
        diesel::sql_query(format!("SELECT COUNT(*) AS n FROM users WHERE id = '{RANDOM}'"))
            .load(&mut conn)
            .unwrap();
    assert_eq!(leftover[0].n, 0, "the random-id row must be gone");

    // The owned filter was repointed.
    let owner: Vec<TextRow> = diesel::sql_query("SELECT user_id AS val FROM filters")
        .load(&mut conn)
        .unwrap();
    assert_eq!(owner.len(), 1);
    assert_eq!(owner[0].val, CANON, "owned rows must be repointed to the canonical id");

    // Foreign keys are intact.
    assert_eq!(fk_violations(&mut conn), 0, "no foreign-key violations after reconcile");

    cleanup(&tmp);
}

#[test]
fn reconcile_is_noop_when_already_canonical() {
    let (mut conn, tmp) = fresh_conn("podfetch_reconcile_std_user_noop.db");
    let reconcile = apply_through_pre_reconcile(&mut conn);

    // A fresh install already seeds `user123` on the canonical id.
    conn.batch_execute(&format!(
        "INSERT INTO users (id, username, role, explicit_consent, created_at) \
           VALUES ('{CANON}', 'user123', 'admin', 1, '2026-01-01T00:00:00');\
         INSERT INTO filters (user_id, ascending, only_favored) \
           VALUES ('{CANON}', 0, 0);"
    ))
    .expect("seed canonical state");

    conn.run_migration(&*reconcile).expect("apply reconcile migration");

    let users: Vec<TextRow> =
        diesel::sql_query("SELECT id AS val FROM users WHERE username = 'user123'")
            .load(&mut conn)
            .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].val, CANON, "already-canonical user must be left untouched");
    assert_eq!(fk_violations(&mut conn), 0, "no foreign-key violations");

    cleanup(&tmp);
}
