pub mod db;
#[path = "schemas/sqlite/schema.rs"]
pub mod schema;

#[derive(diesel::MultiConnection)]
pub enum DBType {
    #[cfg(feature = "postgresql")]
    Postgresql(diesel::PgConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(diesel::SqliteConnection),
}

#[macro_export]
macro_rules! import_database_config {
    () => {
        #[cfg(feature = "sqlite")]
        pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/sqlite");

        #[cfg(feature = "postgresql")]
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
            #[cfg(feature = "sqlite")]
            $crate::adapters::persistence::dbconfig::DBType::Sqlite(conn) => {
                return $diesel_func(conn);
            }
            #[cfg(feature = "postgresql")]
            $crate::adapters::persistence::dbconfig::DBType::Postgresql(conn) => {
                return $diesel_func(conn);
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
            #[cfg(feature = "sqlite")]
            $crate::adapters::persistence::dbconfig::DBType::Sqlite(conn) => $diesel_func(conn),
            #[cfg(feature = "postgresql")]
            $crate::adapters::persistence::dbconfig::DBType::Postgresql(conn) => $diesel_func(conn),
        };
    }};
}
