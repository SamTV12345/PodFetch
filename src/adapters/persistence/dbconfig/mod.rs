pub mod db;
#[path = "schemas/sqlite/schema.rs"]
pub mod schema;

use diesel::QueryResult;

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
    ($diesel_func:expr) => {{
        use std::ops::DerefMut;
        use $crate::get_connection;

        let mut conn = get_connection();
        let _ = match conn.deref_mut() {
            $crate::adapters::persistence::dbconfig::DBType::Sqlite(conn) => {
                return $diesel_func(conn)
            }
            $crate::adapters::persistence::dbconfig::DBType::Postgresql(conn) => {
                return $diesel_func(conn)
            }
        };
    }};
}

#[macro_export]
macro_rules! insert_with_conn {
    ($diesel_func:expr) => {{
        use std::ops::DerefMut;
        use $crate::get_connection;
        let mut conn = get_connection();
        let _ = match conn.deref_mut() {
            $crate::adapters::persistence::dbconfig::DBType::Sqlite(conn) => $diesel_func(conn),
            $crate::adapters::persistence::dbconfig::DBType::Postgresql(conn) => $diesel_func(conn),
        };
    }};
}
