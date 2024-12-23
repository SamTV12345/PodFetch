use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use diesel::insert_into;
use diesel::Queryable;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Queryable, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}

impl Notification {
    pub fn get_unread_notifications() -> Result<Vec<Notification>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let result = notifications
            .filter(status.eq("unread"))
            .order(created_at.desc())
            .load::<Notification>(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(result)
    }

    pub fn insert_notification(notification: Notification) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::notifications::dsl::notifications;
        use crate::adapters::persistence::dbconfig::schema::notifications::*;
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        insert_into(notifications)
            .values((
                type_of_message.eq(notification.clone().type_of_message),
                message.eq(notification.clone().message),
                status.eq(notification.clone().status),
                created_at.eq(notification.clone().created_at),
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
