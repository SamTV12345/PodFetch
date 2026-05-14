//! Audiobookshelf-compatible login flow.
//!
//! Validates a `(username, password)` against the existing `user_auth_service`,
//! then returns the user's `api_key` as Bearer token. If the user has no
//! `api_key` yet, one is generated and persisted via `UserAdminService::update`.
//!
//! Reverse-proxy / OIDC-only setups are not supported here yet; the flow
//! requires a password hash to validate against. (TODO: proxy/OIDC parity in a
//! follow-up; for now those users must hit /api/authorize with a token they
//! obtained out-of-band.)

use crate::services::user_admin::service::UserAdminService;
use crate::services::user_auth::service::UserAuthService;
use common_infrastructure::error::ErrorSeverity::{Debug, Warning};
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::user::User;
use sha256::digest;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AudiobookshelfLoginService {
    user_auth_service: Arc<UserAuthService>,
    user_admin_service: Arc<UserAdminService>,
}

impl AudiobookshelfLoginService {
    pub fn new(
        user_auth_service: Arc<UserAuthService>,
        user_admin_service: Arc<UserAdminService>,
    ) -> Self {
        Self {
            user_auth_service,
            user_admin_service,
        }
    }

    /// Validates username + password, returns the user with an api_key set.
    /// Generates an api_key if the user doesn't have one yet.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<User, CustomError> {
        let user = self
            .user_auth_service
            .find_by_username(username)
            .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;

        let Some(stored_hash) = user.password.as_deref() else {
            tracing::warn!(
                "audiobookshelf login: user '{username}' has no local password (likely OIDC/proxy)"
            );
            return Err(CustomErrorInner::Forbidden(Warning).into());
        };

        if stored_hash != digest(password) {
            tracing::warn!("audiobookshelf login: password mismatch for user '{username}'");
            return Err(CustomErrorInner::Forbidden(Warning).into());
        }

        self.ensure_api_key(user)
    }

    /// Looks up a user by bearer token. Returns `None` if no user matches.
    pub fn user_from_token(&self, token: &str) -> Result<Option<User>, CustomError> {
        self.user_auth_service.find_by_api_key(token)
    }

    /// Rotates the user's api_key to a fresh UUID. Used by /logout when the
    /// `audiobookshelf_rotate_api_key_on_logout` flag is on.
    pub fn rotate_api_key(&self, mut user: User) -> Result<User, CustomError> {
        user.api_key = Some(format!("abs_{}", Uuid::new_v4().simple()));
        self.user_admin_service.update_user(user)
    }

    fn ensure_api_key(&self, mut user: User) -> Result<User, CustomError> {
        if user.api_key.as_deref().is_some_and(|k| !k.is_empty()) {
            return Ok(user);
        }
        user.api_key = Some(format!("abs_{}", Uuid::new_v4().simple()));
        self.user_admin_service.update_user(user)
    }

    pub fn require_user_for_token(&self, token: &str) -> Result<User, CustomError> {
        self.user_from_token(token)?.ok_or_else(|| {
            CustomError::from(CustomErrorInner::UnAuthorized(
                "Invalid token".to_string(),
                Debug,
            ))
        })
    }
}
