use crate::DbConnection;
use diesel::Queryable;
use utoipa::ToSchema;
use crate::utils::do_retry::do_retry;
use diesel::insert_into;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Serialize, Deserialize, Queryable,Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}

impl Notification{

    pub fn get_unread_notifications(conn: &mut DbConnection) -> Result<Vec<Notification>,
        CustomError> {
        use diesel::QueryDsl;
        use crate::dbconfig::schema::notifications::dsl::*;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;

        let result = notifications
            .filter(status.eq("unread"))
            .order(created_at.desc())
            .load::<Notification>(conn)
            .map_err(map_db_error)?;
        Ok(result)
    }

    pub fn insert_notification(notification: Notification, conn: &mut DbConnection) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::notifications::dsl::notifications;
        use crate::dbconfig::schema::notifications::*;
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        insert_into(notifications)
            .values((
                type_of_message.eq(notification.clone().type_of_message),
                message.eq(notification.clone().message),
                status.eq(notification.clone().status),
                created_at.eq(notification.clone().created_at),
            ))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_status_of_notification(
        id_to_search: i32,
        status_update: &str,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::notifications::dsl::*;
        use diesel::QueryDsl;
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;
        do_retry(|| {
            diesel::update(notifications.filter(id.eq(id_to_search)))
                .set(status.eq(status_update))
                .execute(conn)
        })
        .map_err(map_db_error)?;
        Ok(())
    }
}