#[cfg(test)]
pub mod test {
    use chrono::Utc;
    use podfetch_domain::user::User;
    use crate::role::Role;

    use sha256::digest;

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

#[cfg(test)]
pub mod test_builder;
