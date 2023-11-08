#[path = "schemas/sqlite/schema.rs"]
pub mod schema;

#[cfg(mysql)]
#[path = "schemas/mysql/schema.rs"]
pub mod schema;


#[macro_export]
#[cfg(mysql)]
macro_rules! import_database_connections {
    () => {
        use diesel::MysqlConnection;
    };
}

#[derive(diesel::MultiConnection)]
pub enum DBType {
    Postgresql(diesel::PgConnection),
    Sqlite(diesel::SqliteConnection),
}

#[macro_export]
macro_rules! import_database_config{
    ()=>{
    #[cfg(sqlite)]
    pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/sqlite");


    #[cfg(postgresql)]
    pub const POSTGRES_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/postgres");
    }
}