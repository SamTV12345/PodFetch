use diesel::dsl::insert_into;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::notification::notification::NotificationEntity;
use crate::domain::models::notification::notification::Notification;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};

pub struct NotificationRepository;

impl NotificationRepository {
    pub fn get_unread_notifications() -> Result<Vec<Notification>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        notifications
            .filter(status.eq("unread"))
            .order(created_at.desc())
            .load::<NotificationEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|notification_entities| notification_entities.into_iter().map(|n| n.into()).collect())
    }

    pub fn insert_notification(
        notification: Notification,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::notifications;
        use crate::adapters::persistence::dbconfig::schema::notifications::*;
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        insert_into(notifications)
            .values((
                type_of_message.eq(notification.type_of_message),
                message.eq(notification.message),
                status.eq(notification.status),
                created_at.eq(notification.created_at),
            ))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_status_of_notification(
        id_to_search: i32,
        status_update: &str,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        do_retry(|| {
            diesel::update(notifications.filter(id.eq(id_to_search)))
                .set(status.eq(status_update))
                .execute(&mut get_connection())
        })
            .map_err(map_db_error)?;
        Ok(())
    }
}
