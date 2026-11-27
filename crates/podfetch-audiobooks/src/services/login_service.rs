//! Audiobookshelf-compatible login flow.
//!
//! Identity checks the password and owns the api key; this service only
//! translates the result into an [`Account`], whose api key Audiobookshelf
//! clients use as Bearer token.
//!
//! Reverse-proxy / OIDC-only users have no local password and cannot log in
//! this way; they use `/api/authorize` with a token obtained out-of-band.

use crate::account::Account;
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_identity::services::user_auth::service::UserAuthService;
use std::sync::Arc;

#[derive(Clone)]
pub struct AudiobookshelfLoginService {
    identity: Arc<UserAuthService>,
}

impl AudiobookshelfLoginService {
    pub fn new(identity: Arc<UserAuthService>) -> Self {
        Self { identity }
    }

    /// Checks the credentials and makes sure the user has a token.
    pub fn authenticate(&self, username: &str, password: &str) -> Result<Account, CustomError> {
        let user = self.identity.authenticate(username, password)?;
        self.identity.ensure_api_key(user).map(Account::from)
    }

    pub fn user_from_token(&self, token: &str) -> Result<Option<Account>, CustomError> {
        Ok(self.identity.find_by_api_key(token)?.map(Account::from))
    }

    /// Issues a new token; the old one stops working.
    pub fn rotate_api_key(&self, account: &Account) -> Result<Account, CustomError> {
        let user = self.identity.find_by_username(&account.username)?;
        self.identity.rotate_api_key(user).map(Account::from)
    }

    pub fn require_user_for_token(&self, token: &str) -> Result<Account, CustomError> {
        self.user_from_token(token)?.ok_or_else(|| {
            CustomError::from(CustomErrorInner::UnAuthorized(
                "Invalid token".to_string(),
                Debug,
            ))
        })
    }
}
