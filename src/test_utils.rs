#[cfg(test)]
pub mod test {
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use sha256::digest;
    use testcontainers::{ContainerRequest, ImageExt};
    use testcontainers::core::ContainerPort;
    use testcontainers_modules::postgres::Postgres;

    pub fn init() -> ContainerRequest<Postgres> {
        let container = setup_container();
        // Set frontend url
        container
    }

    pub fn setup_container() -> ContainerRequest<Postgres> {
        let container = Postgres::default().with_mapped_port(55002, ContainerPort::Tcp(5432));
        let connection_string = "postgres://postgres:postgres@127.0.0.1:55002/postgres";
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
