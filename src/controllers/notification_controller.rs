use crate::app_state::AppState;
use common_infrastructure::error::CustomError;
use axum::Json;
use axum::extract::State;
use podfetch_web::notification::{self, Notification, NotificationId};
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
get,
path="/notifications/unread",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
pub async fn get_unread_notifications(
    State(state): State<AppState>,
) -> Result<Json<Vec<Notification>>, CustomError> {
    notification::get_unread_notifications(state.notification_service.as_ref()).map(Json)
}

#[utoipa::path(
put,
path="/notifications/dismiss",
responses(
(status = 200, description = "Dismisses a notification")),
tag="notifications"
)]
pub async fn dismiss_notifications(
    State(state): State<AppState>,
    Json(id): Json<NotificationId>,
) -> Result<StatusCode, CustomError> {
    notification::dismiss_notification(state.notification_service.as_ref(), id.id)?;
    Ok(StatusCode::OK)
}

pub fn get_notification_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_unread_notifications))
        .routes(routes!(dismiss_notifications))
}

#[cfg(test)]
mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::application::services::notification::service::NotificationService;
    use crate::test_utils::test_builder::notification_test_builder::tests::NotificationTestDataBuilder;
    use podfetch_web::notification::Notification;
    use serde_json::json;
    use serial_test::serial;

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

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
        NotificationService::create_notification(notification).unwrap();

        // when
        let response = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;

        // then
        assert!(response.status_code().is_success());
        assert_eq!(response.json::<Vec<Notification>>().len(), 1)
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_endpoint_marks_notification_as_dismissed() {
        let test_server = handle_test_startup().await;

        let notification = NotificationTestDataBuilder::new().build();
        NotificationService::create_notification(notification).unwrap();

        let before = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;
        assert!(before.status_code().is_success());
        let unread_before = before.json::<Vec<Notification>>();
        assert_eq!(unread_before.len(), 1);
        let notification_id = unread_before[0].id;

        let dismiss_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": notification_id }))
            .await;
        assert_eq!(dismiss_response.status_code(), 200);

        let after = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;
        assert!(after.status_code().is_success());
        assert_eq!(after.json::<Vec<Notification>>().len(), 0);
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_endpoint_with_unknown_id_is_noop() {
        let test_server = handle_test_startup().await;

        let notification = NotificationTestDataBuilder::new().build();
        NotificationService::create_notification(notification).unwrap();

        let dismiss_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": 999_999_999 }))
            .await;
        assert_eq!(dismiss_response.status_code(), 200);

        let after = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;
        assert!(after.status_code().is_success());
        assert_eq!(after.json::<Vec<Notification>>().len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_endpoint_rejects_invalid_payload() {
        let test_server = handle_test_startup().await;

        let wrong_type_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": "not-a-number" }))
            .await;
        assert_client_error_status(wrong_type_response.status_code().as_u16());

        let missing_id_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "otherField": 1 }))
            .await;
        assert_client_error_status(missing_id_response.status_code().as_u16());

        let null_payload_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!(null))
            .await;
        assert_client_error_status(null_payload_response.status_code().as_u16());

        let array_payload_response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!([1, 2, 3]))
            .await;
        assert_client_error_status(array_payload_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_unread_notifications_filters_out_dismissed_items() {
        let test_server = handle_test_startup().await;

        NotificationService::create_notification(Notification {
            id: 0,
            type_of_message: "Download".to_string(),
            message: "should-be-returned".to_string(),
            created_at: "2026-03-14 10:00:00".to_string(),
            status: "unread".to_string(),
        })
        .unwrap();
        NotificationService::create_notification(Notification {
            id: 0,
            type_of_message: "Download".to_string(),
            message: "should-be-filtered".to_string(),
            created_at: "2026-03-14 11:00:00".to_string(),
            status: "dismissed".to_string(),
        })
        .unwrap();

        let response = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;
        assert_eq!(response.status_code(), 200);

        let unread = response.json::<Vec<Notification>>();
        assert_eq!(unread.len(), 1);
        assert_eq!(unread[0].message, "should-be-returned");
    }

    #[tokio::test]
    #[serial]
    async fn test_get_unread_notifications_orders_by_created_at_desc() {
        let test_server = handle_test_startup().await;

        NotificationService::create_notification(Notification {
            id: 0,
            type_of_message: "Download".to_string(),
            message: "older-message".to_string(),
            created_at: "2026-03-14 08:00:00".to_string(),
            status: "unread".to_string(),
        })
        .unwrap();
        NotificationService::create_notification(Notification {
            id: 0,
            type_of_message: "Download".to_string(),
            message: "newer-message".to_string(),
            created_at: "2026-03-14 12:00:00".to_string(),
            status: "unread".to_string(),
        })
        .unwrap();

        let response = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await;
        assert_eq!(response.status_code(), 200);

        let unread = response.json::<Vec<Notification>>();
        assert_eq!(unread.len(), 2);
        assert_eq!(unread[0].message, "newer-message");
        assert_eq!(unread[1].message, "older-message");
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_endpoint_is_idempotent() {
        let test_server = handle_test_startup().await;

        NotificationService::create_notification(NotificationTestDataBuilder::new().build())
            .unwrap();

        let unread_before = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await
            .json::<Vec<Notification>>();
        assert_eq!(unread_before.len(), 1);
        let notification_id = unread_before[0].id;

        let first_dismiss = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": notification_id }))
            .await;
        assert_eq!(first_dismiss.status_code(), 200);

        let second_dismiss = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": notification_id }))
            .await;
        assert_eq!(second_dismiss.status_code(), 200);

        let unread_after = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await
            .json::<Vec<Notification>>();
        assert!(unread_after.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_notification_endpoints_return_client_error_for_wrong_http_method() {
        let test_server = handle_test_startup().await;

        let unread_with_put = test_server
            .test_server
            .put("/api/v1/notifications/unread")
            .await;
        assert_client_error_status(unread_with_put.status_code().as_u16());

        let dismiss_with_get = test_server
            .test_server
            .get("/api/v1/notifications/dismiss")
            .await;
        assert_client_error_status(dismiss_with_get.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_notification_endpoints_return_client_error_for_additional_wrong_methods() {
        let test_server = handle_test_startup().await;

        let unread_with_post = test_server
            .test_server
            .post("/api/v1/notifications/unread")
            .await;
        assert_client_error_status(unread_with_post.status_code().as_u16());

        let dismiss_with_delete = test_server
            .test_server
            .delete("/api/v1/notifications/dismiss")
            .await;
        assert_client_error_status(dismiss_with_delete.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_notification_endpoints_return_not_found_for_invalid_paths() {
        let test_server = handle_test_startup().await;

        let unknown_route = test_server
            .test_server
            .get("/api/v1/notifications/unknown")
            .await;
        assert_eq!(unknown_route.status_code(), 404);

        let dismiss_with_extra_segment = test_server
            .test_server
            .put("/api/v1/notifications/dismiss/1")
            .await;
        assert_eq!(dismiss_with_extra_segment.status_code(), 404);

        let near_miss_route = test_server
            .test_server
            .get("/api/v1/notification/unread")
            .await;
        assert_eq!(near_miss_route.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_endpoint_rejects_missing_body() {
        let test_server = handle_test_startup().await;

        let response = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_notification_with_zero_or_negative_id_is_noop() {
        let test_server = handle_test_startup().await;

        NotificationService::create_notification(NotificationTestDataBuilder::new().build())
            .unwrap();

        let dismiss_zero = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": 0 }))
            .await;
        assert_eq!(dismiss_zero.status_code(), 200);

        let dismiss_negative = test_server
            .test_server
            .put("/api/v1/notifications/dismiss")
            .json(&json!({ "id": -1 }))
            .await;
        assert_eq!(dismiss_negative.status_code(), 200);

        let unread_after = test_server
            .test_server
            .get("/api/v1/notifications/unread")
            .await
            .json::<Vec<Notification>>();
        assert_eq!(unread_after.len(), 1);
    }
}


