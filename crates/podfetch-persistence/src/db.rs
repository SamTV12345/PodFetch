pub use common_infrastructure::db::*;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;

pub fn database() -> Database {
    common_infrastructure::db::shared_database(&ENVIRONMENT_SERVICE)
        .expect("Failed to connect to database")
}

pub fn get_connection() -> r2d2::PooledConnection<diesel::r2d2::ConnectionManager<DBType>> {
    common_infrastructure::db::shared_connection(&ENVIRONMENT_SERVICE)
        .expect("Failed to connect to database")
}
