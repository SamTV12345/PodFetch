use std::process::exit;
use std::sync::OnceLock;
use std::time::Duration;
use diesel::Connection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use crate::adapters::persistence::dbconfig::DBType;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::DbPool;

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
    let database_url = &ENVIRONMENT_SERVICE.get().unwrap().database_url;
    DBType::establish(database_url).unwrap_or_else(|e| {
        log::error!("Error connecting to {} with reason {}", database_url, e);
        exit(1)
    })
}

static POOL: OnceLock<DbPool> = OnceLock::new();


pub fn get_connection() -> r2d2::PooledConnection<ConnectionManager<DBType>> {
    POOL.get_or_init(init_pool).get().unwrap()
}

fn init_pool() -> DbPool {
    let conn = establish_connection();
    match conn {
        DBType::Postgresql(_) => {
            init_postgres_db_pool(&ENVIRONMENT_SERVICE.get().unwrap().database_url)
                .expect("Failed to connect to database")
        }
        DBType::Sqlite(_) => {
            init_sqlite_db_pool(&ENVIRONMENT_SERVICE.get().unwrap().database_url)
                .expect("Failed to connect to database")
        }
    }
}


fn init_postgres_db_pool(
    database_url: &str,
) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();
    let db_connections = env_service.conn_number;
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(db_connections as u32)
        .build(manager)
        .expect("Failed to create pool.");
    Ok(pool)
}

fn init_sqlite_db_pool(
    database_url: &str,
) -> Result<Pool<ConnectionManager<DBType>>, String> {
    let manager = ConnectionManager::<DBType>::new(database_url);
    let pool = Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(120)),
        }))
        .build(manager)
        .unwrap();
    Ok(pool)
}