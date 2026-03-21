use serde::Deserialize;
use utoipa::ToSchema;

use podfetch_domain::notification::Notification;

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
