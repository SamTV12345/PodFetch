#[cfg(test)]
pub mod test {
    use crate::constants::inner_constants::Role;
    use crate::models::user::User;
    use chrono::Utc;

    use sha256::digest;
    
    #[cfg(feature = "postgresql")]
    use testcontainers::{ContainerRequest, ImageExt};
    #[cfg(feature = "postgresql")]
    use testcontainers_modules::postgres::Postgres;
    #[cfg(feature = "postgresql")]
    pub fn setup_container() -> ContainerRequest<Postgres> {
        Postgres::default().with_mapped_port(55002, 5432.into())
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
