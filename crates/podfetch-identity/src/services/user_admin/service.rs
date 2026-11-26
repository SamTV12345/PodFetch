use crate::domain::user::{User, UserWithoutPassword};
use crate::domain::user_admin::UserAdminRepository;
use crate::events::UserDeleted;
use crate::persistence::user_admin::delete_by_username_on;
use crate::role::STANDARD_USER_ID;
use crate::user_admin::{ManagedUser, UserAdminApplicationService};
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_events::{EventBus, EventBusError};
use podfetch_kernel::UserId;
use std::sync::Arc;

fn map_event_error(error: EventBusError) -> CustomError {
    tracing::error!("Could not publish identity event: {error}");
    CustomErrorInner::Unknown(ErrorSeverity::Error).into()
}

#[derive(Clone)]
pub struct UserAdminService {
    repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
    environment: Arc<EnvironmentService>,
    events: Arc<EventBus>,
}

impl UserAdminService {
    pub fn new(
        repository: Arc<dyn UserAdminRepository<Error = CustomError>>,
        environment: Arc<EnvironmentService>,
        events: Arc<EventBus>,
    ) -> Self {
        Self {
            repository,
            environment,
            events,
        }
    }

    /// Deletes the user and publishes [`UserDeleted`] in the same
    /// transaction, so other contexts learn about every deletion and about
    /// no deletion that was rolled back.
    fn delete_and_publish(&self, username: &str) -> Result<(), CustomError> {
        self.events
            .transaction(|tx| -> Result<(), EventBusError> {
                if let Some(user_id) = delete_by_username_on(tx.conn(), username)? {
                    tx.publish(&UserDeleted {
                        user_id: UserId::from(user_id),
                        username: username.to_owned(),
                    })?;
                }
                Ok(())
            })
            .map_err(map_event_error)
    }

    pub fn read_only_admin_id(&self) -> uuid::Uuid {
        if let Some(admin_username) = &self.environment.username
            && let Ok(Some(user)) = self.repository.find_by_username(admin_username)
        {
            return user.id;
        }

        STANDARD_USER_ID
    }

    pub fn oidc_configured(&self) -> bool {
        self.environment.oidc_configured
    }

    pub fn create_user(&self, user: User) -> Result<User, CustomError> {
        self.repository.create(user.into()).map(Into::into)
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<User>, CustomError> {
        self.repository
            .find_by_username(username)
            .map(|user| user.map(Into::into))
    }

    pub fn list_users(&self) -> Result<Vec<UserWithoutPassword>, CustomError> {
        self.repository
            .find_all()
            .map(|users| users.into_iter().map(|user| user.to_summary()).collect())
    }

    pub fn update_user(&self, user: User) -> Result<User, CustomError> {
        self.repository.update(user.into()).map(Into::into)
    }

    pub fn delete_user_by_username(&self, username: &str) -> Result<(), CustomError> {
        self.delete_and_publish(username)
    }
}

pub fn map_requester(user: &User) -> ManagedUser {
    user.clone().into()
}

impl UserAdminApplicationService for UserAdminService {
    type Error = CustomError;

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error> {
        self.repository
            .find_by_username(username)
            .map(|user| user.map(Into::into))
    }

    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error> {
        self.repository
            .find_all()
            .map(|users| users.into_iter().map(Into::into).collect())
    }

    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error> {
        self.repository.update(user.into()).map(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error> {
        self.delete_and_publish(username)
    }
}
