use crate::config::EnvironmentService;
use diesel::Connection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::sync::OnceLock;
#[cfg(feature = "sqlite")]
use std::time::Duration;

pub type DbPool = Pool<ConnectionManager<DBType>>;

#[derive(diesel::MultiConnection)]
pub enum DBType {
    #[cfg(feature = "postgresql")]
    Postgresql(diesel::PgConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(diesel::SqliteConnection),
}

#[derive(Clone)]
pub struct Database {
    pool: DbPool,
}

static DATABASE: OnceLock<Database> = OnceLock::new();

impl Database {
    pub fn from_url(database_url: &str, _max_connections: u32) -> Result<Self, PersistenceError> {
        let pool = match DBType::establish(database_url)? {
            #[cfg(feature = "postgresql")]
            DBType::Postgresql(_) => init_postgres_db_pool(database_url, _max_connections)?,
            #[cfg(feature = "sqlite")]
            DBType::Sqlite(_) => init_sqlite_db_pool(database_url)?,
        };

        Ok(Self { pool })
    }

    pub fn connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<DBType>>, PersistenceError> {
        self.pool.get().map_err(PersistenceError::Pool)
    }
}

pub fn shared_database(environment: &EnvironmentService) -> Result<Database, PersistenceError> {
    if let Some(database) = DATABASE.get() {
        return Ok(database.clone());
    }

    let database = Database::from_url(
        &environment.database_url,
        environment.max_database_connections(),
    )?;
    let _ = DATABASE.set(database);

    Ok(DATABASE
        .get()
        .expect("database should be initialized")
        .clone())
}

pub fn shared_connection(
    environment: &EnvironmentService,
) -> Result<r2d2::PooledConnection<ConnectionManager<DBType>>, PersistenceError> {
    shared_database(environment)?.connection()
}

#[cfg(feature = "sqlite")]
#[derive(Debug)]
struct ConnectionOptions {
    enable_wal: bool,
    enable_foreign_keys: bool,
    busy_timeout: Option<Duration>,
}

#[cfg(feature = "sqlite")]
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

#[cfg(feature = "postgresql")]
fn init_postgres_db_pool(
    database_url: &str,
    max_connections: u32,
) -> Result<DbPool, PersistenceError> {
    let manager = ConnectionManager::<DBType>::new(database_url);
    Pool::builder()
        .max_size(max_connections)
        .build(manager)
        .map_err(PersistenceError::Pool)
}

#[cfg(feature = "sqlite")]
fn init_sqlite_db_pool(database_url: &str) -> Result<DbPool, PersistenceError> {
    let manager = ConnectionManager::<DBType>::new(database_url);
    Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(120)),
        }))
        .build(manager)
        .map_err(PersistenceError::Pool)
}

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("pool error: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("connection error: {0}")]
    Connection(#[from] diesel::ConnectionError),
}
