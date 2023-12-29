use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::dbconfig::DBType;
use crate::DBType as DbConnection;
use diesel::prelude::*;
use std::process::exit;
use std::time::Duration;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl r2d2::CustomizeConnection<DbConnection, diesel::r2d2::Error> for ConnectionOptions {
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

pub fn establish_connection() -> DBType {
    let database_url = &ENVIRONMENT_SERVICE.get().unwrap().database_url;
    DBType::establish(database_url).unwrap_or_else(|e| {
        log::error!("Error connecting to {} with reason {}", database_url, e);
        exit(1)
    })
}
