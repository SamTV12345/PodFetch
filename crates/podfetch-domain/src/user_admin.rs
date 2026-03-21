use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagedUser {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime,
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserWithApiKey {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool,
    pub api_key: Option<String>,
    pub read_only: bool,
}

impl ManagedUser {
    pub fn new(
        id: i32,
        username: impl Into<String>,
        role: impl ToString,
        password: Option<impl Into<String>>,
        created_at: NaiveDateTime,
        explicit_consent: bool,
    ) -> Self {
        Self {
            id,
            username: username.into(),
            role: role.to_string(),
            password: password.map(|password| password.into()),
            explicit_consent,
            created_at,
            api_key: None,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_privileged_user(&self) -> bool {
        self.role == "admin" || self.role == "uploader"
    }

    pub fn to_summary(&self) -> UserSummary {
        UserSummary {
            id: self.id,
            username: self.username.clone(),
            role: self.role.clone(),
            created_at: self.created_at,
            explicit_consent: self.explicit_consent,
        }
    }

    pub fn to_api_dto(&self, read_only: bool) -> UserWithApiKey {
        UserWithApiKey {
            id: self.id,
            username: self.username.clone(),
            role: self.role.clone(),
            created_at: self.created_at,
            explicit_consent: self.explicit_consent,
            api_key: self.api_key.clone(),
            read_only,
        }
    }
}

pub trait UserAdminRepository: Send + Sync {
    type Error;

    fn create(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error>;
    fn find_by_api_key(&self, api_key: &str) -> Result<Option<ManagedUser>, Self::Error>;
    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error>;
    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error>;
    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
}
