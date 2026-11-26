use crate::domain::{Notification, NotificationRepository};
use common_infrastructure::db::{Database, PersistenceError};
use common_infrastructure::error::CustomError;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

diesel::table! {
    notifications (id) {
        id -> Text,
        type_of_message -> Text,
        message -> Text,
        created_at -> Text,
        status -> Text,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = notifications)]
struct NotificationEntity {
    id: String,
    type_of_message: String,
    message: String,
    created_at: String,
    status: String,
}

impl From<NotificationEntity> for Notification {
    fn from(value: NotificationEntity) -> Self {
        Self {
            id: Uuid::parse_str(&value.id).expect("valid uuid in db"),
            type_of_message: value.type_of_message,
            message: value.message,
            created_at: value.created_at,
            status: value.status,
        }
    }
}

pub struct DieselNotificationRepository {
    database: Database,
}

impl DieselNotificationRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl NotificationRepository for DieselNotificationRepository {
    type Error = PersistenceError;

    fn create(&self, notification: Notification) -> Result<Notification, Self::Error> {
        use self::notifications::dsl::*;

        diesel::insert_into(notifications)
            .values((
                id.eq(Uuid::now_v7().to_string()),
                type_of_message.eq(notification.type_of_message),
                message.eq(notification.message),
                created_at.eq(notification.created_at),
                status.eq(notification.status),
            ))
            .get_result::<NotificationEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        use self::notifications::dsl::*;

        notifications
            .filter(status.eq("unread"))
            .order(created_at.desc())
            .load::<NotificationEntity>(&mut self.database.connection()?)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update_status_of_notification(
        &self,
        id_to_update: Uuid,
        status_update: &str,
    ) -> Result<(), Self::Error> {
        use self::notifications::dsl::*;

        diesel::update(notifications.filter(id.eq(id_to_update.to_string())))
            .set(status.eq(status_update))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn update_status_of_all_unread(&self, status_update: &str) -> Result<usize, Self::Error> {
        use self::notifications::dsl::*;

        diesel::update(notifications.filter(status.eq("unread")))
            .set(status.eq(status_update))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}

/// [`NotificationRepository`] with the error type the web layer works with.
pub struct NotificationRepositoryImpl {
    inner: DieselNotificationRepository,
}

impl NotificationRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselNotificationRepository::new(database),
        }
    }
}

impl NotificationRepository for NotificationRepositoryImpl {
    type Error = CustomError;

    fn create(&self, notification: Notification) -> Result<Notification, Self::Error> {
        self.inner.create(notification).map_err(Into::into)
    }

    fn get_unread_notifications(&self) -> Result<Vec<Notification>, Self::Error> {
        self.inner.get_unread_notifications().map_err(Into::into)
    }

    fn update_status_of_notification(&self, id: Uuid, status: &str) -> Result<(), Self::Error> {
        self.inner
            .update_status_of_notification(id, status)
            .map_err(Into::into)
    }

    fn update_status_of_all_unread(&self, status: &str) -> Result<usize, Self::Error> {
        self.inner
            .update_status_of_all_unread(status)
            .map_err(Into::into)
    }
}
