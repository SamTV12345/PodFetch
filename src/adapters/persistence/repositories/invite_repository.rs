use crate::utils::error::CustomError;
use podfetch_domain::invite::{Invite, InviteRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::invite::DieselInviteRepository;

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
