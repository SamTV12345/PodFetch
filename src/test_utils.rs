#[cfg(test)]
pub mod test {
    use testcontainers::Container;
    use testcontainers::runners::{SyncRunner};
    use testcontainers_modules::postgres::Postgres;
    use crate::run_migrations;
    use diesel::RunQueryDsl;

    pub fn setup_container() -> Container<Postgres> {
        let container = Postgres::default().start().unwrap();
        let host_port = container.get_host_port_ipv4(5432).unwrap();
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",
        );
        std::env::set_var("DATABASE_URL", connection_string);
        run_migrations();
        container
    }


    pub fn clear_users() {
        use crate::adapters::persistence::dbconfig::schema::users::dsl::*;
        diesel::delete(users).execute(&mut crate::get_connection()).unwrap();
    }
}