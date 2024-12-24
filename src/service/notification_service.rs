use crate::models::notification::Notification;
use crate::utils::error::CustomError;

pub struct NotificationService {}

impl NotificationService {
    pub fn get_unread_notifications() -> Result<Vec<Notification>, CustomError> {
        Notification::get_unread_notifications()
    }

    pub fn update_status_of_notification(id: i32, status: &str) -> Result<(), CustomError> {
        Notification::update_status_of_notification(id, status)
    }
}
