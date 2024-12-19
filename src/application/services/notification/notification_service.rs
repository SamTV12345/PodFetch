use crate::adapters::persistence::repositories::notification::notification::NotificationRepository;
use crate::domain::models::notification::notification::Notification;
use crate::utils::error::CustomError;

pub struct NotificationService;

impl NotificationService {
    pub fn get_unread_notifications() -> Result<Vec<Notification>, CustomError> {
        NotificationRepository::get_unread_notifications()
    }

    pub fn insert_notification(
        notification: Notification,
    ) -> Result<(), CustomError> {
        NotificationRepository::insert_notification(notification)
    }

    pub fn update_status_of_notification(
        id: i32,
        status: &str,
    ) -> Result<(), CustomError> {
        NotificationRepository::update_status_of_notification(id, status)
    }
}