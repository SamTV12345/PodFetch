//! The published interface of the identity context.
//!
//! Other contexts see a user only as a [`Principal`]: who is making the
//! request and what they may do. Credentials (password hash, API key) and
//! profile data never leave this context through it.

use std::sync::Arc;

use common_infrastructure::error::CustomError;
use podfetch_kernel::UserId;

use crate::domain::user_admin::{ManagedUser, UserAdminRepository};
use crate::role::Role;

/// The authenticated caller of a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Principal {
    pub id: UserId,
    pub username: String,
    pub role: Role,
}

impl Principal {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Admin
    }

    /// Admins and uploaders may add and change podcasts.
    pub fn is_privileged_user(&self) -> bool {
        matches!(self.role, Role::Admin | Role::Uploader)
    }
}

impl From<&ManagedUser> for Principal {
    fn from(user: &ManagedUser) -> Self {
        Self {
            id: UserId::from(user.id),
            username: user.username.clone(),
            // Roles are validated on write; an unknown value can only come from
            // a hand-edited database and gets the least privileged role.
            role: Role::try_from(user.role.clone()).unwrap_or(Role::User),
        }
    }
}

/// Questions other contexts may ask the identity context.
pub trait IdentityApi: Send + Sync {
    fn principal(&self, id: UserId) -> Result<Option<Principal>, CustomError>;

    fn principal_by_username(&self, username: &str) -> Result<Option<Principal>, CustomError>;

    /// The user's API key, used to build personal feed URLs. `None` when
    /// the user has none (yet).
    fn api_key(&self, id: UserId) -> Result<Option<String>, CustomError>;

    /// The user's preferred content language, e.g. for podcast discovery.
    fn language(&self, id: UserId) -> Result<Option<String>, CustomError>;
}

/// [`IdentityApi`] on top of the user repository.
pub struct Identity {
    users: Arc<dyn UserAdminRepository<Error = CustomError>>,
}

impl Identity {
    pub fn new(users: Arc<dyn UserAdminRepository<Error = CustomError>>) -> Self {
        Self { users }
    }

    fn user(&self, id: UserId) -> Result<Option<ManagedUser>, CustomError> {
        self.users.find_by_id(id.as_uuid())
    }
}

impl IdentityApi for Identity {
    fn principal(&self, id: UserId) -> Result<Option<Principal>, CustomError> {
        Ok(self.user(id)?.as_ref().map(Principal::from))
    }

    fn principal_by_username(&self, username: &str) -> Result<Option<Principal>, CustomError> {
        Ok(self
            .users
            .find_by_username(username)?
            .as_ref()
            .map(Principal::from))
    }

    fn api_key(&self, id: UserId) -> Result<Option<String>, CustomError> {
        Ok(self.user(id)?.and_then(|user| user.api_key))
    }

    fn language(&self, id: UserId) -> Result<Option<String>, CustomError> {
        Ok(self.user(id)?.and_then(|user| user.language))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn user(role: &str) -> ManagedUser {
        ManagedUser::new(
            Uuid::now_v7(),
            "alice",
            role,
            Some("hash"),
            Utc::now().naive_utc(),
            false,
        )
    }

    #[test]
    fn principal_carries_no_credentials_and_maps_roles() {
        let admin = user("admin");
        let principal = Principal::from(&admin);
        assert_eq!(principal.id, admin.id);
        assert_eq!(principal.username, "alice");
        assert!(principal.is_admin() && principal.is_privileged_user());

        let uploader = Principal::from(&user("uploader"));
        assert!(!uploader.is_admin() && uploader.is_privileged_user());
    }

    #[test]
    fn unknown_roles_get_the_least_privilege() {
        let principal = Principal::from(&user("superuser"));
        assert_eq!(principal.role, Role::User);
        assert!(!principal.is_privileged_user());
    }
}
