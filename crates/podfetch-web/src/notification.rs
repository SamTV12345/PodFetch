use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}

impl From<podfetch_domain::notification::Notification> for Notification {
    fn from(value: podfetch_domain::notification::Notification) -> Self {
        Self {
            id: value.id.to_string(),
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
            id: Uuid::parse_str(&value.id).unwrap_or_else(|_| Uuid::nil()),
            type_of_message: value.type_of_message,
            message: value.message,
            created_at: value.created_at,
            status: value.status,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct NotificationId {
    pub id: String,
}

pub trait NotificationApplicationService {
    type Error;

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error>;
    fn dismiss_notification(&self, id: Uuid) -> Result<(), Self::Error>;
}

pub fn get_unread_notifications<S>(service: &S) -> Result<Vec<Notification>, S::Error>
where
    S: NotificationApplicationService,
{
    service.get_unread_notifications()
}

pub fn dismiss_notification<S>(service: &S, id: Uuid) -> Result<(), S::Error>
where
    S: NotificationApplicationService,
{
    service.dismiss_notification(id)
}
