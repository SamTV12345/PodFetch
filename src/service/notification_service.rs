use crate::db::DB;
use crate::models::models::Notification;

#[derive(Clone)]
pub struct NotificationService {
    db: DB
}

impl NotificationService {
    pub fn new() -> NotificationService {
        NotificationService {
            db: DB::new().expect("Error creating db")
        }
    }

    pub fn get_unread_notifications(&mut self)->Result<Vec<Notification>, String>{
        self.db.get_unread_notifications()
    }

    pub fn update_status_of_notification(&mut self, id: i32, status: &str) -> Result<(), String> {
        self.db.update_status_of_notification(id, status)
    }
}