#[cfg(test)]
pub mod tests {
    use crate::models::user::User;
    use fake::Fake;
    use fake::faker::internet::raw::Username;
    use fake::locales::EN;

    pub struct UserTestDataBuilder {
        id: i32,
        username: String,
        role: String,
        password: Option<String>,
        explicit_consent: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        api_key: Option<String>,
    }

    impl Default for UserTestDataBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl UserTestDataBuilder {
        pub fn new() -> UserTestDataBuilder {
            UserTestDataBuilder {
                id: 1,
                username: Username(EN).fake(),
                role: "user".to_string(),
                password: Some(sha256::digest("password".to_string())),
                explicit_consent: true,
                created_at: chrono::Utc::now(),
                api_key: Some("api_key".to_string()),
            }
        }

        pub fn build(self) -> User {
            User {
                id: self.id,
                explicit_consent: self.explicit_consent,
                username: self.username,
                password: self.password,
                created_at: self.created_at.naive_utc(),
                api_key: self.api_key,
                role: self.role,
            }
        }
    }
}
