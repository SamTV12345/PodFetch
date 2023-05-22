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

