use crate::models::notification::Notification;
use crate::service::notification_service::NotificationService;
use crate::utils::error::CustomError;
use axum::Json;
use reqwest::StatusCode;

use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
get,
path="/notifications/unread",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
pub async fn get_unread_notifications() -> Result<Json<Vec<Notification>>, CustomError> {
    let notifications = NotificationService::get_unread_notifications()?;
    Ok(Json(notifications))
}

#[derive(Deserialize, ToSchema)]
pub struct NotificationId {
    id: i32,
}

#[utoipa::path(
put,
path="/notifications/dismiss",
responses(
(status = 200, description = "Dismisses a notification")),
tag="notifications"
)]
pub async fn dismiss_notifications(
    Json(id): Json<NotificationId>,
) -> Result<StatusCode, CustomError> {
    NotificationService::update_status_of_notification(id.id, "dismissed")?;
    Ok(StatusCode::OK)
}

pub fn get_notification_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_unread_notifications))
        .routes(routes!(dismiss_notifications))
}

#[cfg(test)]
mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::models::notification::Notification;
    use crate::utils::test_builder::notification_test_builder::tests::NotificationTestDataBuilder;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_get_unread_notifications() {
        // given
        let test_server = handle_test_startup().await;

        // when
        let response = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;

        // then
        assert!(response.status_code().is_success());
        assert_eq!(response.json::<Vec<Notification>>().len(), 0)
    }

    #[tokio::test]
    #[serial]
    async fn test_get_dismiss_notifications() {
        let test_server = handle_test_startup().await;
        // given
        let notification = NotificationTestDataBuilder::new().build();
        Notification::insert_notification(notification).unwrap();

        // when
        let response = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;

        // then
        assert!(response.status_code().is_success());
        assert_eq!(response.json::<Vec<Notification>>().len(), 1)
    }
}
