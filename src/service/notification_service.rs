use crate::DBType as DbConnection;
use crate::models::notification::Notification;
use crate::utils::error::CustomError;

#[derive(Clone)]
pub struct NotificationService {
}

impl NotificationService {
    pub fn new() -> NotificationService {
        NotificationService {
        }
    }

    pub fn get_unread_notifications(&mut self, conn:&mut DbConnection)
        ->Result<Vec<Notification>, CustomError>{
        Notification::get_unread_notifications(conn)
    }

    pub fn update_status_of_notification(&mut self, id: i32, status: &str,conn:&mut DbConnection) -> Result<(),
        CustomError> {
        Notification::update_status_of_notification(id, status, conn)
    }
}