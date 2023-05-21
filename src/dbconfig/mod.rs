use std::{sync::Arc, time::Duration};


use diesel::{
    connection::SimpleConnection,
    r2d2::{ConnectionManager, CustomizeConnection, Pool, PooledConnection},
};

use tokio::{
    sync::{Mutex, OwnedSemaphorePermit, Semaphore},
    time::timeout,
};
use crate::get_database_url;
use crate::constants::constants::DATABASE_TIMEOUT_SECONDS;


#[cfg(sqlite)]
#[path = "schemas/sqlite/schema.rs"]
pub mod __sqlite_schema;

#[cfg(mysql)]
#[path = "schemas/mysql/schema.rs"]
pub mod __mysql_schema;

#[cfg(postgresql)]
#[path = "schemas/postgresql/schema.rs"]
pub mod __postgresql_schema;

macro_rules! generate_connections {
    ( $( $name:ident: $ty:ty ),+ ) => {
        #[allow(non_camel_case_types, dead_code)]
        #[derive(Eq, PartialEq)]
        pub enum DbConnType { $( $name, )+ }

        pub struct DbConn {
            pub conn: Arc<Mutex<Option<DbConnInner>>>,
            permit: Option<OwnedSemaphorePermit>,
        }

        #[allow(non_camel_case_types)]
        pub enum DbConnInner { $( #[cfg($name)] $name(PooledConnection<ConnectionManager< $ty >>), )+ }

        #[derive(Debug)]
        pub struct DbConnOptions {
            pub init_stmts: String,
        }

        $( // Based on <https://stackoverflow.com/a/57717533>.
        #[cfg($name)]
        impl CustomizeConnection<$ty, diesel::r2d2::Error> for DbConnOptions {
            fn on_acquire(&self, conn: &mut $ty) -> Result<(), diesel::r2d2::Error> {
                if !self.init_stmts.is_empty() {
                    conn.batch_execute(&self.init_stmts).map_err(diesel::r2d2::Error::QueryError)?;
                }
                Ok(())
            }
        })+

        #[derive(Clone)]
        pub struct DbPool {
            // This is an 'Option' so that we can drop the pool in a 'spawn_blocking'.
            pool: Option<DbPoolInner>,
            semaphore: Arc<Semaphore>
        }

        #[allow(non_camel_case_types)]
        #[derive(Clone)]
        pub enum DbPoolInner { $( #[cfg($name)] $name(Pool<ConnectionManager< $ty >>), )+ }

        impl Drop for DbConn {
            fn drop(&mut self) {
                let conn = Arc::clone(&self.conn);
                let permit = self.permit.take();

                // Since connection can't be on the stack in an async fn during an
                // await, we have to spawn a new blocking-safe thread...
                tokio::task::spawn_blocking(move || {
                    // And then re-enter the runtime to wait on the async mutex, but in a blocking fashion.
                    let mut conn = tokio::runtime::Handle::current().block_on(conn.lock_owned());

                    if let Some(conn) = conn.take() {
                        drop(conn);
                    }

                    // Drop permit after the connection is dropped
                    drop(permit);
                });
            }
        }

        impl Drop for DbPool {
            fn drop(&mut self) {
                let pool = self.pool.take();
                tokio::task::spawn_blocking(move || drop(pool));
            }
        }

        impl DbPool {
            // For the given database URL, guess its type, run migrations, create pool, and return it
            pub fn from_config() -> Self {
                let url = get_database_url();
                let conn_type = DbConnType::from_url(&url);

                match conn_type { $(
                    DbConnType::$name => {
                        #[cfg($name)]
                        {
                            paste::paste!{ [< $name _migrations >]::run_migrations()?; }
                            let manager = ConnectionManager::new(&url);
                            let pool = Pool::builder()
                                .max_size(CONFIG.database_max_conns())
                                .connection_timeout(Duration::from_secs(CONFIG.database_timeout()))
                                .connection_customizer(Box::new(DbConnOptions{
                                    init_stmts: conn_type.get_init_stmts()
                                }))
                                .build(manager)
                                .unwrap();
                            return DbPool {
                                pool: Some(DbPoolInner::$name(pool)),
                                semaphore: Arc::new(Semaphore::new(CONFIG.database_max_conns() as usize)),
                            }
                        }
                        #[cfg(not($name))]
                        println!("Warning: {} is not supported", stringify!($name));
                        unreachable!("Trying to use a DB backend when it's feature is disabled")
                    },
                )+ }
            }
            // Get a connection from the pool
            pub async fn get(&self) -> Result<DbConn, String> {
                let duration = Duration::from_secs(DATABASE_TIMEOUT_SECONDS);
                let permit = match timeout(duration, Arc::clone(&self.semaphore).acquire_owned()).await {
                    Ok(p) => p.expect("Semaphore should be open"),
                    Err(_) => {
                        panic!("Error")
                    }
                };

                match self.pool.as_ref().expect("DbPool.pool should always be Some()") { _=>{
                    Err("Error configuring datbaase".to_string())
                },  $(
                    #[cfg($name)]
                    DbPoolInner::$name(p) => {
                        let pool = p.clone();
                        let c = run_blocking(move || pool.get_timeout(duration)).await.unwrap();

                        return Ok(DbConn {
                            conn: Arc::new(Mutex::new(Some(DbConnInner::$name(c)))),
                            permit: Some(permit)
                        })
                    },
                )+
    }


            }
        }
    };
}

impl DbConnType{
    pub fn from_url(url: &str)->DbConnType{
        match url {
            "mysql"=>{
            DbConnType::mysql
            },
            "postgres"=>{
                DbConnType::postgresql
            }
            _ => {
                #[cfg(sqlite)]
                return DbConnType::sqlite;

                #[cfg(not(sqlite))]
                panic!("Feature not activated")
            }
        }
    }
}


#[macro_export]
macro_rules! db_run {
    // Same for all dbs
    ( $conn:ident: $body:block ) => {
        db_run! { $conn: sqlite, mysql, postgresql $body }
    };

    ( @raw $conn:ident: $body:block ) => {
        db_run! { @raw $conn: sqlite, mysql, postgresql $body }
    };

    // Different code for each db
    ( $conn:ident: $( $($db:ident),+ $body:block )+ ) => {{
        #[allow(unused)] use diesel::prelude::*;
        #[allow(unused)] use crate::dbconfig::FromDb;

        let conn = $conn.conn.clone();
        let mut conn = conn.lock_owned().await;
        match conn.as_mut().expect("internal invariant broken: self.connection is Some") {
            _=> {
                 unreachable!("Trying to use a DB backend when it's feature is disabled")
            },
                $($(
                #[cfg($db)]
                $crate::db::DbConnInner::$db($conn) => {
                    paste::paste! {
                        #[allow(unused)] use $crate::db::[<__ $db _schema>]::{self as schema, *};
                        #[allow(unused)] use [<__ $db _model>]::*;
                    }

                    tokio::task::block_in_place(move || { $body }) // Run blocking can't be used due to the 'static limitation, use block_in_place instead
                },
            )+)+

        }
    }};

    ( @raw $conn:ident: $( $($db:ident),+ $body:block )+ ) => {{
        #[allow(unused)] use diesel::prelude::*;
        #[allow(unused)] use $crate::db::FromDb;

        let conn = $conn.conn.clone();
        let mut conn = conn.lock_owned().await;
        match conn.as_mut().expect("internal invariant broken: self.connection is Some") {
                $($(
                #[cfg($db)]
                $crate::db::DbConnInner::$db($conn) => {
                    paste::paste! {
                        #[allow(unused)] use $crate::db::[<__ $db _schema>]::{self as schema, *};
                        // @ RAW: #[allow(unused)] use [<__ $db _model>]::*;
                    }

                    tokio::task::block_in_place(move || { $body }) // Run blocking can't be used due to the 'static limitation, use block_in_place instead
                },
            )+)+
        }
    }};
}

pub trait FromDb {
    type Output;
    #[allow(clippy::wrong_self_convention)]
    fn from_db(self) -> Self::Output;
}

impl<T: FromDb> FromDb for Vec<T> {
    type Output = Vec<T::Output>;
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    fn from_db(self) -> Self::Output {
        self.into_iter().map(FromDb::from_db).collect()
    }
}

impl<T: FromDb> FromDb for Option<T> {
    type Output = Option<T::Output>;
    #[allow(clippy::wrong_self_convention)]
    #[inline(always)]
    fn from_db(self) -> Self::Output {
        self.map(FromDb::from_db)
    }
}

generate_connections! {
    sqlite: diesel::sqlite::SqliteConnection,
    mysql: diesel::mysql::MysqlConnection,
    postgresql: diesel::pg::PgConnection
}