use crate::utils::error::CustomError;
use podfetch_domain::user_admin::{ManagedUser, UserAdminRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::user_admin::DieselUserAdminRepository;

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

    fn find_by_api_key(&self, api_key: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.inner.find_by_api_key(api_key).map_err(Into::into)
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
