use common_infrastructure::error::CustomError;
use podfetch_domain::notification::{Notification, NotificationRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::notification::DieselNotificationRepository;

pub struct NotificationRepositoryImpl {
    inner: DieselNotificationRepository,
}

impl NotificationRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselNotificationRepository::new(database),
        }
    }
}

impl NotificationRepository for NotificationRepositoryImpl {
    type Error = CustomError;

    fn create(&self, notification: Notification) -> Result<Notification, Self::Error> {
        self.inner.create(notification).map_err(Into::into)
    }

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        self.inner.get_unread_notifications().map_err(Into::into)
    }

    fn update_status_of_notification(&self, id: i32, status: &str) -> Result<(), Self::Error> {
        self.inner
            .update_status_of_notification(id, status)
            .map_err(Into::into)
    }
}

