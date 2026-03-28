use serde::Deserialize;
use utoipa::ToSchema;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}

impl From<podfetch_domain::notification::Notification> for Notification {
    fn from(value: podfetch_domain::notification::Notification) -> Self {
        Self {
            id: value.id,
            type_of_message: value.type_of_message,
            message: value.message,
            created_at: value.created_at,
            status: value.status,
        }
    }
}

impl From<Notification> for podfetch_domain::notification::Notification {
    fn from(value: Notification) -> Self {
        Self {
            id: value.id,
            type_of_message: value.type_of_message,
            message: value.message,
            created_at: value.created_at,
            status: value.status,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct NotificationId {
    pub id: i32,
}

pub trait NotificationApplicationService {
    type Error;

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error>;
    fn dismiss_notification(&self, id: i32) -> Result<(), Self::Error>;
}

pub fn get_unread_notifications<S>(service: &S) -> Result<Vec<Notification>, S::Error>
where
    S: NotificationApplicationService,
{
    service.get_unread_notifications()
}

pub fn dismiss_notification<S>(service: &S, id: i32) -> Result<(), S::Error>
where
    S: NotificationApplicationService,
{
    service.dismiss_notification(id)
}
