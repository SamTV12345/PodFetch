use crate::db::DB;
use crate::DbConnection;
use crate::models::models::Notification;

#[derive(Clone)]
pub struct NotificationService {
}

impl NotificationService {
    pub fn new() -> NotificationService {
        NotificationService {
        }
    }

    pub fn get_unread_notifications(&mut self, mut db:DB, conn:&mut DbConnection)
        ->Result<Vec<Notification>, String>{
        db.get_unread_notifications(conn)
    }

    pub fn update_status_of_notification(&mut self, id: i32, status: &str, mut db:DB,conn:&mut DbConnection) -> Result<(),
        String> {
        db.update_status_of_notification(id, status, conn)
    }
}