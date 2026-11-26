use chrono::NaiveDateTime;
use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── UserAdmin ───────────────────────────────────────────────────────────────

use crate::domain::user_admin::{ManagedUser, UserAdminRepository};
use crate::persistence::user_admin::DieselUserAdminRepository;

pub struct UserAdminRepositoryImpl {
    inner: DieselUserAdminRepository,
}

impl UserAdminRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselUserAdminRepository::new(database),
        }
    }
}

impl UserAdminRepository for UserAdminRepositoryImpl {
    type Error = CustomError;

    fn create(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.inner.create(user).map_err(Into::into)
    }

    fn ensure_with_id(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.inner.ensure_with_id(user).map_err(Into::into)
    }

    fn find_by_api_key(&self, api_key: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_api_key(api_key).map_err(Into::into)
    }

    fn find_by_id(&self, id: Uuid) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_id(id).map_err(Into::into)
    }

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_username(username).map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        self.inner.find_all().map_err(Into::into)
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.inner.update(user).map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.inner.delete_by_username(username).map_err(Into::into)
    }
}

// ── Session ─────────────────────────────────────────────────────────────────

use crate::domain::session::{Session, SessionRepository};
use crate::persistence::session::DieselSessionRepository;

pub struct SessionRepositoryImpl {
    inner: DieselSessionRepository,
}

impl SessionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSessionRepository::new(database),
        }
    }
}

impl SessionRepository for SessionRepositoryImpl {
    type Error = CustomError;

    fn create(&self, session: Session) -> Result<Session, Self::Error> {
        self.inner.create(session).map_err(Into::into)
    }

    fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>, Self::Error> {
        self.inner
            .find_by_session_id(session_id)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<usize, Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn cleanup_expired(&self, now: NaiveDateTime) -> Result<usize, Self::Error> {
        self.inner.cleanup_expired(now).map_err(Into::into)
    }
}

// ── Invite ──────────────────────────────────────────────────────────────────

use crate::domain::invite::{Invite, InviteRepository};
use crate::persistence::invite::DieselInviteRepository;

pub struct InviteRepositoryImpl {
    inner: DieselInviteRepository,
}

impl InviteRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselInviteRepository::new(database),
        }
    }
}

impl InviteRepository for InviteRepositoryImpl {
    type Error = CustomError;

    fn create(&self, role: &str, explicit_consent: bool) -> Result<Invite, Self::Error> {
        self.inner
            .create(role, explicit_consent)
            .map_err(Into::into)
    }

    fn find_by_id(&self, invite_id: &str) -> Result<Option<Invite>, Self::Error> {
        self.inner.find_by_id(invite_id).map_err(Into::into)
    }

    fn find_all(&self) -> Result<Vec<Invite>, Self::Error> {
        self.inner.find_all().map_err(Into::into)
    }

    fn invalidate(&self, invite_id: &str) -> Result<(), Self::Error> {
        self.inner.invalidate(invite_id).map_err(Into::into)
    }

    fn delete(&self, invite_id: &str) -> Result<(), Self::Error> {
        self.inner.delete(invite_id).map_err(Into::into)
    }
}
