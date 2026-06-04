#[cfg(test)]
pub mod test {
    use crate::role::Role;
    use chrono::Utc;
    use podfetch_domain::user::User;

    use sha256::digest;

    pub fn create_random_user() -> User {
        User::new(
            podfetch_domain::ids::new_id(),
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
