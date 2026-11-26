pub use common_infrastructure::db::*;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::ops::DerefMut;

#[cfg(feature = "sqlite")]
pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/sqlite");

#[cfg(feature = "postgresql")]
pub const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/postgres");

pub fn database() -> Database {
    shared_database(&ENVIRONMENT_SERVICE)
        .unwrap_or_else(|e| panic!("Failed to connect to database: {e}"))
}

pub fn get_connection() -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DBType>> {
    shared_connection(&ENVIRONMENT_SERVICE)
        .unwrap_or_else(|e| panic!("Failed to connect to database: {e}"))
}

pub fn run_migrations() {
    migrate(&database());
}

/// Runs all pending migrations on `database`.
///
/// # Panics
/// When a migration fails: PodFetch cannot run on a half-migrated schema.
pub fn migrate(database: &Database) {
    let mut conn = database
        .connection()
        .unwrap_or_else(|e| panic!("Failed to connect to database: {e}"));
    match conn.deref_mut() {
        #[cfg(feature = "postgresql")]
        DBType::Postgresql(conn) => {
            conn.run_pending_migrations(POSTGRES_MIGRATIONS)
                .expect("Could not run postgres migrations");
        }
        #[cfg(feature = "sqlite")]
        DBType::Sqlite(conn) => {
            conn.run_pending_migrations(SQLITE_MIGRATIONS)
                .expect("Could not run sqlite migrations");
        }
    }
}

/// Test-only helpers shared by every DB-touching test module in this crate.
#[cfg(all(test, feature = "sqlite"))]
pub(crate) mod test_db {
    use std::sync::{Mutex, MutexGuard};

    /// Process-wide lock serializing ALL DB-touching tests in this crate. They
    /// share one sqlite test DB, so `run_migrations()` from parallel threads in
    /// different test modules races on `__diesel_schema_migrations` (UNIQUE
    /// violation), and concurrent writes corrupt each other's assertions. Every
    /// DB test module MUST serialize via this single lock (not a module-local
    /// one) so they exclude each other ACROSS modules.
    static TEST_DB_LOCK: Mutex<()> = Mutex::new(());

    /// Acquire the shared DB lock and ensure migrations have run. Hold the
    /// returned guard for the whole test body.
    pub(crate) fn setup() -> MutexGuard<'static, ()> {
        let guard = TEST_DB_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::run_migrations();
        guard
    }
}
