#[cfg(sqlite)]
#[path = "schemas/sqlite/schema.rs"]
pub mod schema;

#[cfg(mysql)]
#[path = "schemas/mysql/schema.rs"]
pub mod schema;

#[cfg(postgresql)]
#[path = "schemas/postgresql/schema.rs"]
pub mod schema;


#[cfg(sqlite)]
#[path = "schemas/sqlite/schema.rs"]
pub mod __sqlite_schema;

#[cfg(mysql)]
#[path = "schemas/mysql/schema.rs"]
pub mod __mysql_schema;

#[cfg(postgresql)]
#[path = "schemas/postgresql/schema.rs"]
pub mod __postgresql_schema;


#[macro_export]
#[cfg(sqlite)]
macro_rules! import_database_connections {
    () => {
        use diesel::SqliteConnection;
    };
}

#[macro_export]
#[cfg(postgresql)]
macro_rules! import_database_connections {
    () => {
        use diesel::PgConnection;
    };
}

#[macro_export]
#[cfg(mysql)]
macro_rules! import_database_connections {
    () => {
        use diesel::MysqlConnection;
    };
}