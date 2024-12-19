use chrono::NaiveDateTime;
use crate::constants::inner_constants::Role;

pub struct User {
    pub id: i32,
    pub username: String,
    pub role: Role,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime,
    pub api_key: Option<String>,
}

impl User {
    pub fn new(
        id: i32,
        username: String,
        role: Role,
        password: Option<String>,
        created_at: NaiveDateTime,
        explicit_consent: bool,
    ) -> Self {
        User {
            id,
            username,
            role,
            password,
            created_at,
            explicit_consent,
            api_key: None,
        }
    }

    pub fn is_privileged_user(&self) -> bool {
        self.role == Role::Admin || self.role == Role::Uploader
    }
}
