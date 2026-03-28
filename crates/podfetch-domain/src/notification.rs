
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}

pub trait NotificationRepository: Send + Sync {
    type Error;

    fn create(&self, notification: Notification) -> Result<Notification, Self::Error>;
    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error>;
    fn update_status_of_notification(&self, id: i32, status: &str) -> Result<(), Self::Error>;
}

