//! The user as the Audiobookshelf protocol sees it.
//!
//! Anti-corruption layer towards identity: this context never handles
//! password hashes or identity's user record, only this translation.

use chrono::NaiveDateTime;
use podfetch_identity::domain::user::User;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    /// The bearer token Audiobookshelf clients send (the user's api key).
    pub api_key: Option<String>,
    pub created_at: NaiveDateTime,
}

impl Account {
    pub fn is_admin(&self) -> bool {
        podfetch_identity::api::Principal::from(self).is_admin()
    }

    /// Admins and uploaders may add and change content.
    pub fn is_privileged_user(&self) -> bool {
        podfetch_identity::api::Principal::from(self).is_privileged_user()
    }
}

impl From<User> for Account {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            role: user.role,
            api_key: user.api_key,
            created_at: user.created_at,
        }
    }
}

impl From<&Account> for podfetch_identity::api::Principal {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.into(),
            username: account.username.clone(),
            // Unknown values can only come from a hand-edited database and
            // get the least privileged role, as in identity.
            role: podfetch_identity::role::Role::try_from(account.role.clone())
                .unwrap_or(podfetch_identity::role::Role::User),
        }
    }
}
