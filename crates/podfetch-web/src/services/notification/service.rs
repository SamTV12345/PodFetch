use crate::notification::{Notification, NotificationApplicationService};
use common_infrastructure::error::CustomError;
use podfetch_domain::notification::NotificationRepository;
use podfetch_persistence::adapters::NotificationRepositoryImpl;
use podfetch_persistence::db::database;
use std::sync::Arc;

#[derive(Clone)]
pub struct NotificationService {
    repository: Arc<dyn NotificationRepository<Error = CustomError>>,
}

impl NotificationService {
    pub fn new(repository: Arc<dyn NotificationRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(NotificationRepositoryImpl::new(database())))
    }

    pub fn create_notification(notification: Notification) -> Result<Notification, CustomError> {
        Self::default_service().create(notification)
    }

    pub fn create(&self, notification: Notification) -> Result<Notification, CustomError> {
        self.repository.create(notification.into()).map(Into::into)
    }

    pub fn get_unread_notifications(&self) -> Result<Vec<Notification>, CustomError> {
        self.repository
            .get_unread_notifications()
            .map(|notifications| notifications.into_iter().map(Into::into).collect())
    }

    pub fn update_status_of_notification(&self, id: i32, status: &str) -> Result<(), CustomError> {
        self.repository.update_status_of_notification(id, status)
    }
}

impl NotificationApplicationService for NotificationService {
    type Error = CustomError;

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        self.get_unread_notifications()
    }

    fn dismiss_notification(&self, id: i32) -> Result<(), Self::Error> {
        self.update_status_of_notification(id, "dismissed")
    }
}
