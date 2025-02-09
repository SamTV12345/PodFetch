#[cfg(test)]
pub mod test {
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;

    use chrono::Utc;

    use sha256::digest;

    use testcontainers::core::ContainerPort;
    use testcontainers::{ContainerRequest, ImageExt};
    use testcontainers_modules::postgres::Postgres;

    pub fn setup_container() -> ContainerRequest<Postgres> {
        Postgres::default().with_mapped_port(55002, ContainerPort::Tcp(5432))
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
