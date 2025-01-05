#[cfg(test)]
pub mod test {
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;
    use crate::run_migrations;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use sha256::digest;
    use std::sync::{LazyLock, RwLock};
    use testcontainers::runners::SyncRunner;
    use testcontainers::Container;
    use testcontainers_modules::postgres::Postgres;

    static CONTAINER: LazyLock<RwLock<Option<Container<Postgres>>>> =
        LazyLock::new(|| RwLock::new(None));

    #[cfg(test)]
    #[ctor::ctor]
    fn init() {
        let container = setup_container();
        let mut conatiner = CONTAINER.write().unwrap();
        *conatiner = Some(container);
    }

    #[cfg(test)]
    #[ctor::dtor]
    fn stop() {
        if std::env::var("GH_ACTION").is_err() {
            if let Ok(mut container) = CONTAINER.write() {
                *container = None
            }
        }
    }

    pub fn setup_container() -> Container<Postgres> {
        let container = Postgres::default().start().unwrap();
        let host_port = container.get_host_port_ipv4(5432).unwrap();
        let connection_string =
            &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
        std::env::set_var("DATABASE_URL", connection_string);
        run_migrations();
        container
    }

    pub fn clear_users() {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        diesel::delete(users)
            .execute(&mut crate::get_connection())
            .unwrap();
    }

    pub fn create_random_user() -> User {
        User::new(
            0,
            "testuser",
            Role::User,
            Some(digest("testuser")),
            Utc::now().naive_utc(),
            false,
        )
    }
}
