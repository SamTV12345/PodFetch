use crate::adapters::persistence::dbconfig::DBType;
use crate::commands::startup::DbPool;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use diesel::r2d2::ConnectionManager;
use diesel::Connection;
use r2d2::Pool;
use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl r2d2::CustomizeConnection<DBType, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut DBType) -> Result<(), diesel::r2d2::Error> {
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
    let database_url = &ENVIRONMENT_SERVICE.database_url;
    DBType::establish(database_url).unwrap_or_else(|e| {
        log::error!("Error connecting to {} with reason {}", database_url, e);
        panic!("Error connecting to database")
    })
}

static POOL: OnceLock<DbPool> = OnceLock::new();

pub fn get_connection() -> r2d2::PooledConnection<ConnectionManager<DBType>> {
    POOL.get_or_init(init_pool).get().unwrap()
}

fn init_pool() -> DbPool {
    let conn = establish_connection();
    match conn {
        #[cfg(feature = "postgresql")]
        DBType::Postgresql(_) => init_postgres_db_pool(&ENVIRONMENT_SERVICE.database_url)
            .expect("Failed to connect to database"),
        #[cfg(feature = "sqlite")]
        DBType::Sqlite(_) => init_sqlite_db_pool(&ENVIRONMENT_SERVICE.database_url)
            .expect("Failed to connect to database"),
    }
}

#[cfg(feature = "postgresql")]
fn init_postgres_db_pool(database_url: &str) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let db_connections = ENVIRONMENT_SERVICE.conn_number;
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(db_connections as u32)
        .build(manager);

    match pool {
        Err(e) => {
            log::error!("Failed to create postgres pool: {}", e);
            exit(1);
        }
        Ok(pool) => Ok(pool),
    }
}

#[cfg(feature = "sqlite")]
fn init_sqlite_db_pool(database_url: &str) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(120)),
        }))
        .build(manager);

    match pool {
        Err(e) => {
            log::error!("Failed to create pool: {}", e);
            exit(1);
        }
        Ok(pool) => Ok(pool),
    }
}
