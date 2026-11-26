use crate::domain::{Notification as NotificationRecord, NotificationRepository};
use crate::dto::{Notification, NotificationApplicationService};
use chrono::Utc;
use common_infrastructure::error::CustomError;
use std::sync::Arc;
use uuid::Uuid;

/// The kinds of in-app notifications. The strings are stored and read by
/// the UI, so they must not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationKind {
    Download,
    DownloadFailed,
}

impl NotificationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Download => "Download",
            Self::DownloadFailed => "DownloadFailed",
        }
    }
}

pub const UNREAD: &str = "unread";
pub const DISMISSED: &str = "dismissed";

#[derive(Clone)]
pub struct NotificationService {
    repository: Arc<dyn NotificationRepository<Error = CustomError>>,
}

impl NotificationService {
    pub fn new(repository: Arc<dyn NotificationRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    /// Stores a new unread notification.
    pub fn notify(
        &self,
        kind: NotificationKind,
        message: impl Into<String>,
    ) -> Result<Notification, CustomError> {
        self.repository
            .create(NotificationRecord {
                id: Uuid::nil(),
                type_of_message: kind.as_str().to_owned(),
                message: message.into(),
                created_at: Utc::now().naive_utc().to_string(),
                status: UNREAD.to_owned(),
            })
            .map(Into::into)
    }

    pub fn get_unread_notifications(&self) -> Result<Vec<Notification>, CustomError> {
        self.repository
            .get_unread_notifications()
            .map(|notifications| notifications.into_iter().map(Into::into).collect())
    }

    pub fn update_status_of_notification(&self, id: Uuid, status: &str) -> Result<(), CustomError> {
        self.repository.update_status_of_notification(id, status)
    }
}

impl NotificationApplicationService for NotificationService {
    type Error = CustomError;

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        self.get_unread_notifications()
    }

    fn dismiss_notification(&self, id: Uuid) -> Result<(), Self::Error> {
        self.update_status_of_notification(id, DISMISSED)
    }

    fn dismiss_all_notifications(&self) -> Result<usize, Self::Error> {
        self.repository.update_status_of_all_unread(DISMISSED)
    }
}
