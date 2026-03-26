pub use common_infrastructure::db::*;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::ops::DerefMut;

#[cfg(feature = "sqlite")]
pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/sqlite");

#[cfg(feature = "postgresql")]
pub const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/postgres");

pub fn database() -> Database {
    common_infrastructure::db::shared_database(&ENVIRONMENT_SERVICE)
        .expect("Failed to connect to database")
}

pub fn get_connection() -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DBType>> {
    common_infrastructure::db::shared_connection(&ENVIRONMENT_SERVICE)
        .expect("Failed to connect to database")
}

pub fn run_migrations() {
    let mut conn = get_connection();
    let conn = conn.deref_mut();

    match conn {
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
