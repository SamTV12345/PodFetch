use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Uploader,
    User,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::Uploader => write!(f, "uploader"),
            Role::User => write!(f, "user"),
        }
    }
}

impl TryFrom<String> for Role {
    type Error = CustomError;

    fn try_from(value: String) -> Result<Self, CustomError> {
        match value.as_str() {
            "admin" | "Admin" => Ok(Role::Admin),
            "uploader" | "Uploader" => Ok(Role::Uploader),
            "user" | "User" => Ok(Role::User),
            _ => Err(CustomErrorInner::BadRequest("Invalid role".to_string(), Warning).into()),
        }
    }
}

impl Role {
    pub const VALUES: [Self; 3] = [Self::User, Self::Admin, Self::Uploader];
}

pub const STANDARD_USER: &str = "user123";
/// Fixed identity for the no-auth standard admin user. Stable across installs.
pub const STANDARD_USER_ID: Uuid = Uuid::from_u128(0x00000000_0000_7000_8000_000000009999);
