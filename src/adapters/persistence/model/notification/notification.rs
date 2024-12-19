use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use diesel::insert_into;
use diesel::Queryable;
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::domain::models::notification::notification::Notification;

#[derive(Serialize, Deserialize, Queryable, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEntity {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}


impl From<Notification> for NotificationEntity {
    fn from(notification: Notification) -> Self {
        Self {
            id: notification.id,
            type_of_message: notification.type_of_message,
            message: notification.message,
            created_at: notification.created_at,
            status: notification.status,
        }
    }
}

impl Into<Notification> for NotificationEntity {
    fn into(self) -> Notification {
        Notification {
            id: self.id,
            type_of_message: self.type_of_message,
            message: self.message,
            created_at: self.created_at,
            status: self.status,
        }
    }
}
