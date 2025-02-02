use axum::Json;
use reqwest::StatusCode;
use crate::models::notification::Notification;
use crate::service::notification_service::NotificationService;
use crate::utils::error::CustomError;

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
    use axum_test::TestServer;
    use serial_test::serial;
    use testcontainers::runners::AsyncRunner;
    use crate::commands::startup::handle_config_for_server_startup;
    use crate::models::notification::Notification;
    use crate::test_utils::test::init;
    use crate::utils::test_builder::notification_test_builder::tests::NotificationTestDataBuilder;

    #[tokio::test]
    #[serial]
    async fn test_get_unread_notifications() {
        // given
        let container = init();
        let _container = container.start().await.unwrap();
        let router = handle_config_for_server_startup();

        // when
        let ts_server = TestServer::new(router).unwrap();
        let response = ts_server.get("/api/v1/notifications/unread").await;

        // then
        assert_eq!(response.status_code().is_success(), true);
        assert_eq!(response.json::<Vec<Notification>>().len(), 0)
    }

    #[tokio::test]
    #[serial]
    async fn test_get_dismiss_notifications() {
        // given
        let container = init();
        let _container = container.start().await.unwrap();
        let router = handle_config_for_server_startup();
        let notification = NotificationTestDataBuilder::new().build();
        Notification::insert_notification(notification).unwrap();

        // when
        let ts_server = TestServer::new(router).unwrap();
        let response = ts_server.get("/api/v1/notifications/unread").await;

        // then
        assert_eq!(response.status_code().is_success(), true);
        assert_eq!(response.json::<Vec<Notification>>().len(), 1)
    }
}