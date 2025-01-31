#[cfg(test)]
pub mod test {
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use sha256::digest;
    
    use axum::Router;
    use testcontainers::runners::SyncRunner;
    use testcontainers::Container;
    use testcontainers_modules::postgres::Postgres;
    use crate::commands::startup::handle_config_for_server_startup;

    pub fn init() -> (Container<Postgres>, Router) {
        let container = setup_container();
        let router = handle_config_for_server_startup();
        (container, router)
    }

    pub fn setup_container() -> Container<Postgres> {
        let container = Postgres::default().start().unwrap();
        let host_port = container.get_host_port_ipv4(5432).unwrap();
        let connection_string =
            &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
        std::env::set_var("DATABASE_URL", connection_string);
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
