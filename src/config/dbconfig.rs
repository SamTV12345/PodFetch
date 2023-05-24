use diesel::prelude::*;
use std::env;
use std::time::Duration;
use crate::DbConnection;


#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

#[cfg(sqlite)]
impl r2d2::CustomizeConnection<DbConnection, diesel::r2d2::Error>
for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut DbConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;
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

#[cfg(sqlite)]
pub fn establish_connection() -> DbConnection {
    let database_url = &get_database_url();
    DbConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


#[cfg(postgresql)]
pub fn establish_connection()->PgConnection{
    let database_url = &get_database_url();
    println!("Connecting to {}", database_url);
    PgConnection::establish(database_url)
        .unwrap_or_else(|e| panic!("Error connecting to {} with error {}", database_url,e))
}

#[cfg(mysql)]
pub fn establish_connection()->PgConnection{
    let database_url = &get_database_url();
    MysqlConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


pub fn get_database_url()->String{
    //println!("{}",env::var("DATABASE_URL").unwrap_or("sqlite://./db/podcast.db".to_string()));
    env::var("DATABASE_URL").unwrap_or("sqlite://./db/podcast.db".to_string())
}
