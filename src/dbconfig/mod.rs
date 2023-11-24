#[path = "schemas/sqlite/schema.rs"]
pub mod schema;

#[derive(diesel::MultiConnection)]
pub enum DBType {
    Postgresql(diesel::PgConnection),
    Sqlite(diesel::SqliteConnection),
}

#[macro_export]
macro_rules! import_database_config {
    () => {
        pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/sqlite");

        pub const POSTGRES_MIGRATIONS: EmbeddedMigrations =
            embed_migrations!("./migrations/postgres");
    };
}

#[macro_export]
macro_rules! execute_with_conn {
    ($conn:expr, $diesel_func:expr) => {
        match $conn {
            DbConnection::Sqlite(conn) => return $diesel_func(conn),
            DbConnection::Postgresql(conn) => return $diesel_func(conn),
        }
    };
}

#[macro_export]
macro_rules! insert_with_conn {
    ($conn:expr, $diesel_func:expr) => {
        match $conn {
            DbConnection::Sqlite(conn) =>  $diesel_func(conn),
            DbConnection::Postgresql(conn) => $diesel_func(conn),
        }
    };
}