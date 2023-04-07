use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::time::Duration;
use diesel::connection::SimpleConnection;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
            .map_err(diesel::r2d2::Error::QueryError)
    }
}
pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = &get_database_url();
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


pub fn get_database_url()->String{
    env::var("DATABASE_URL").unwrap_or("sqlite://./db/podcast.db".to_string())
}
