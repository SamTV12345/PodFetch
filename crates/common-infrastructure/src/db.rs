use crate::config::EnvironmentService;
use diesel::Connection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use std::sync::OnceLock;
#[cfg(any(feature = "sqlite", not(feature = "postgresql")))]
use std::time::Duration;

pub type DbPool = Pool<ConnectionManager<DBType>>;

#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection;

#[derive(diesel::MultiConnection)]
pub enum DBType {
    #[cfg(feature = "postgresql")]
    Postgresql(diesel::PgConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
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
            DBType::Sqlite(conn) => init_sqlite_db_pool(database_url, conn, SQLITE_BUSY_TIMEOUT)?,
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

#[cfg(any(feature = "sqlite", not(feature = "postgresql")))]
#[derive(Debug)]
struct ConnectionOptions {
    enable_wal: bool,
    enable_foreign_keys: bool,
    busy_timeout: Option<Duration>,
}

#[cfg(any(feature = "sqlite", not(feature = "postgresql")))]
impl r2d2::CustomizeConnection<DBType, diesel::r2d2::Error> for ConnectionOptions {
    fn on_acquire(&self, conn: &mut DBType) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;
        (|| {
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL;")?;
            }
            // Per-connection setting, safe in WAL mode.
            conn.batch_execute("PRAGMA synchronous = NORMAL;")?;
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

/// How long a sqlite connection waits for a lock held by someone else before
/// giving up with `database is locked`.
#[cfg(any(feature = "sqlite", not(feature = "postgresql")))]
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(120);

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

/// Prepares the sqlite database on a single connection and then builds the pool.
///
/// Opening a WAL database after an unclean shutdown makes the first connection
/// recover the WAL while holding a lock. The pool used to open 16 connections
/// at once and switch every one of them to WAL in `on_acquire`; while the
/// recovery (or a lock held by another process) blocked them inside the
/// 120 s busy timeout, r2d2's 30 s build timeout fired with the useless
/// `Pool(Error(None))` and the server panicked – which killed the recovery and
/// started the same loop on the next container restart (#2338).
///
/// Now the persistent `journal_mode = WAL` switch happens once, here, on a
/// plain connection that is allowed to wait for the busy timeout, and the pool
/// only opens a single connection eagerly.
#[cfg(any(feature = "sqlite", not(feature = "postgresql")))]
fn init_sqlite_db_pool(
    database_url: &str,
    mut setup_conn: SqliteConnection,
    busy_timeout: Duration,
) -> Result<DbPool, PersistenceError> {
    use diesel::connection::SimpleConnection;

    tracing::info!("Opening sqlite database {database_url}");
    setup_conn
        .batch_execute(&format!(
            "PRAGMA busy_timeout = {}; PRAGMA journal_mode = WAL;",
            busy_timeout.as_millis()
        ))
        .map_err(|e| PersistenceError::Locked {
            url: database_url.to_string(),
            source: e,
        })?;
    drop(setup_conn);

    let manager = ConnectionManager::<DBType>::new(database_url);
    Pool::builder()
        .max_size(16)
        .min_idle(Some(1))
        // Must exceed the busy timeout, otherwise a waiting connection is
        // reported as the context-free `Error(None)` instead of its real error.
        .connection_timeout(busy_timeout + Duration::from_secs(10))
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: false,
            enable_foreign_keys: true,
            busy_timeout: Some(busy_timeout),
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
    #[error(
        "could not open the database {url}: {source}. The database file is locked – \
         is another PodFetch instance (or another tool) using the same database file? \
         Network shares (NFS/SMB) often do not support sqlite locking."
    )]
    Locked {
        url: String,
        source: diesel::result::Error,
    },
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use diesel::connection::SimpleConnection;
    use std::time::Instant;

    fn temp_db(name: &str) -> (std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join(format!("pf-db-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("podcast.db");
        let url = path.to_str().unwrap().to_string();
        (path, url)
    }

    fn open(url: &str) -> SqliteConnection {
        SqliteConnection::establish(url).unwrap()
    }

    #[test]
    fn locked_database_reports_a_meaningful_error() {
        let (_path, url) = temp_db("locked");
        let mut locker = open(&url);
        locker
            .batch_execute("CREATE TABLE t(x); BEGIN EXCLUSIVE; INSERT INTO t VALUES(1);")
            .unwrap();

        let start = Instant::now();
        let Err(err) = init_sqlite_db_pool(&url, open(&url), Duration::from_secs(1)) else {
            panic!("a locked database must not yield a pool");
        };
        assert!(start.elapsed() < Duration::from_secs(10));
        assert!(matches!(err, PersistenceError::Locked { .. }), "{err:?}");
        assert!(err.to_string().contains("locked"), "{err}");
    }

    #[test]
    fn waits_for_a_short_lock_instead_of_failing() {
        let (_path, url) = temp_db("short-lock");
        let mut locker = open(&url);
        locker
            .batch_execute("CREATE TABLE t(x); BEGIN EXCLUSIVE; INSERT INTO t VALUES(1);")
            .unwrap();
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            locker.batch_execute("COMMIT;").unwrap();
        });

        let pool = init_sqlite_db_pool(&url, open(&url), Duration::from_secs(20))
            .expect("pool after the lock is released");
        release.join().unwrap();
        assert_eq!(
            pool.state().connections,
            1,
            "only one connection opened eagerly"
        );

        let mut conn = pool.get().unwrap();
        #[allow(irrefutable_let_patterns)] // refutable with the postgresql feature
        let DBType::Sqlite(conn) = &mut *conn else {
            unreachable!()
        };
        #[derive(diesel::QueryableByName)]
        struct Mode {
            #[diesel(sql_type = diesel::sql_types::Text)]
            journal_mode: String,
        }
        use diesel::RunQueryDsl;
        let mode: Mode = diesel::sql_query("PRAGMA journal_mode")
            .get_result(conn)
            .unwrap();
        assert_eq!(mode.journal_mode.to_lowercase(), "wal");
    }
}
