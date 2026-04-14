use crate::app_state::AppState;
use crate::tags;
pub use crate::tags::{Tag, TagCreate, TagsPodcast};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use podfetch_domain::user::User;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
post,
path="/tags",
responses(
(status = 200, description = "Creates a new tag",
body = Tag)),
tag="tags"
)]
pub async fn insert_tag(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(tag_create): Json<TagCreate>,
) -> Result<Json<Tag>, CustomError> {
    tags::create_tag(
        state.tag_service.as_ref(),
        requester.id,
        tag_create,
    )
    .map(Json)
}

#[utoipa::path(
get,
path="/tags",
responses(
(status = 200, description = "Gets all tags of a user", body=Vec<Tag>)),
tag="tags"
)]
pub async fn get_tags(
    State(state): State<AppState>,
    requester: Extension<User>,
) -> Result<Json<Vec<Tag>>, CustomError> {
    tags::get_tags(state.tag_service.as_ref(), requester.id).map(Json)
}

#[utoipa::path(
delete,
path="/tags/{tag_id}",
responses(
(status = 200, description = "Deletes a tag by id")),
tag="tags"
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    tags::delete_tag(state.tag_service.as_ref(), requester.id, &tag_id)
        .map(|_| StatusCode::OK)
}

#[utoipa::path(
put,
path="/tags/{tag_id}",
responses(
(status = 200, description = "Updates a tag by id")),
tag="tags"
)]
pub async fn update_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
    Json(tag_create): Json<TagCreate>,
) -> Result<Json<Tag>, CustomError> {
    tags::update_tag(
        state.tag_service.as_ref(),
        requester.id,
        &tag_id,
        tag_create,
    )
    .map(Json)
}

#[utoipa::path(
post,
path="/tags/{tag_id}/{podcast_id}",
responses(
(status = 200, description = "Adds a podcast to a tag", body=TagsPodcast)),
tag="tags"
)]
pub async fn add_podcast_to_tag(
    State(state): State<AppState>,
    Path(tag_id_to_convert): Path<(String, i32)>,
    requester: Extension<User>,
) -> Result<Json<TagsPodcast>, CustomError> {
    let (tag_id, podcast_id) = tag_id_to_convert;
    tags::add_podcast_to_tag(
        state.tag_service.as_ref(),
        requester.id,
        &tag_id,
        podcast_id,
    )
    .map(Json)
}

#[utoipa::path(
delete,
path="/tags/{tag_id}/{podcast_id}",
responses(
(status = 200, description = "Deletes a podcast from a tag")),
tag="tags"
)]
pub async fn delete_podcast_from_tag(
    State(state): State<AppState>,
    Path(tag_id): Path<(String, i32)>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    let (tag_id, podcast_id) = tag_id;

    tags::delete_podcast_from_tag(
        state.tag_service.as_ref(),
        requester.id,
        &tag_id,
        podcast_id,
    )
    .map(|_| StatusCode::OK)
}

pub fn get_tags_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(insert_tag))
        .routes(routes!(get_tags))
        .routes(routes!(delete_tag))
        .routes(routes!(update_tag))
        .routes(routes!(add_podcast_to_tag))
        .routes(routes!(delete_podcast_from_tag))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::tags::{Color, Tag};
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::extract::{Path, State};
    use axum::{Extension, Json};
    use common_infrastructure::error::CustomErrorInner;
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;

    fn admin_username() -> String {
        ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string())
    }

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn other_user() -> podfetch_domain::user::User {
        let mut user = UserTestDataBuilder::new().build();
        user.id = 999_999;
        user
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_update_and_delete_tag() {
        let server = handle_test_startup().await;
        let tag_name = unique_name("Backend");
        let updated_tag_name = unique_name("Backend Updated");

        let create_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": tag_name,
                "description": "API related",
                "color": "Red"
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created_tag = create_response.json::<Tag>();
        assert_eq!(created_tag.name, tag_name);
        assert_eq!(created_tag.description, Some("API related".to_string()));

        let update_response = server
            .test_server
            .put(&format!("/api/v1/tags/{}", created_tag.id))
            .json(&json!({
                "name": updated_tag_name,
                "description": "API and DB",
                "color": "Blue"
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated_tag = update_response.json::<Tag>();
        assert_eq!(updated_tag.name, updated_tag_name);
        assert_eq!(updated_tag.description, Some("API and DB".to_string()));
        assert_eq!(updated_tag.color, "Blue");

        let list_response = server.test_server.get("/api/v1/tags").await;
        assert_eq!(list_response.status_code(), 200);
        let tags = list_response.json::<Vec<Tag>>();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].id, created_tag.id);

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/tags/{}", created_tag.id))
            .await;
        assert_eq!(delete_response.status_code(), 200);

        let list_after_delete = server.test_server.get("/api/v1/tags").await;
        assert_eq!(list_after_delete.status_code(), 200);
        assert!(list_after_delete.json::<Vec<Tag>>().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_add_and_remove_podcast_from_tag() {
        let server = handle_test_startup().await;
        let username = admin_username();
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("tagged-podcast-{unique}");
        let tag_name = unique_name("Favorites");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Tagged Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();

        let tag_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": tag_name,
                "description": "Pinned",
                "color": "Green"
            }))
            .await;
        assert_eq!(tag_response.status_code(), 200);
        let tag = tag_response.json::<Tag>();

        let add_response = server
            .test_server
            .post(&format!("/api/v1/tags/{}/{}", tag.id, podcast.id))
            .await;
        assert_eq!(add_response.status_code(), 200);

        let tags_for_podcast = app_state()
            .tag_service
            .get_tags_of_podcast(podcast.id, 9999)
            .unwrap();
        assert_eq!(tags_for_podcast.len(), 1);
        assert_eq!(tags_for_podcast[0].id, tag.id);

        let remove_response = server
            .test_server
            .delete(&format!("/api/v1/tags/{}/{}", tag.id, podcast.id))
            .await;
        assert_eq!(remove_response.status_code(), 200);

        let tags_after_remove = app_state()
            .tag_service
            .get_tags_of_podcast(podcast.id, 9999)
            .unwrap();
        assert!(tags_after_remove.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_and_delete_unknown_tag_return_not_found() {
        let server = handle_test_startup().await;
        let unknown_tag_id = "tag-does-not-exist";

        let update_response = server
            .test_server
            .put(&format!("/api/v1/tags/{unknown_tag_id}"))
            .json(&json!({
                "name": unique_name("Unknown Update"),
                "description": "no-op",
                "color": "Blue"
            }))
            .await;
        assert_eq!(update_response.status_code(), 404);

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/tags/{unknown_tag_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_add_and_remove_podcast_with_unknown_tag_return_not_found() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("unknown-tag-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Unknown Tag Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();

        let add_response = server
            .test_server
            .post(&format!("/api/v1/tags/tag-does-not-exist/{}", podcast.id))
            .await;
        assert_eq!(add_response.status_code(), 404);

        let remove_response = server
            .test_server
            .delete(&format!("/api/v1/tags/tag-does-not-exist/{}", podcast.id))
            .await;
        assert_eq!(remove_response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_tag_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let invalid_color_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": unique_name("Invalid Color"),
                "description": "invalid color payload",
                "color": "Invisible"
            }))
            .await;
        assert_client_error_status(invalid_color_response.status_code().as_u16());

        let missing_name_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "description": "missing name",
                "color": "Red"
            }))
            .await;
        assert_client_error_status(missing_name_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_tag_handlers_return_not_found_for_other_user_tag_access() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": unique_name("Owner Tag"),
                "description": "owned by current user",
                "color": "Red"
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created_tag = create_response.json::<Tag>();

        let update_result = super::update_tag(
            State(app_state()),
            Path(created_tag.id.clone()),
            Extension(other_user()),
            Json(super::TagCreate {
                name: unique_name("Hacker Rename"),
                description: Some("forbidden".to_string()),
                color: Color::Blue,
            }),
        )
        .await;
        match update_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::NotFound(_))),
            Ok(_) => panic!("expected not found for update_tag with other user"),
        }

        let delete_result = super::delete_tag(
            State(app_state()),
            Path(created_tag.id),
            Extension(other_user()),
        )
        .await;
        match delete_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::NotFound(_))),
            Ok(_) => panic!("expected not found for delete_tag with other user"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_add_podcast_to_tag_rejects_non_numeric_podcast_id() {
        let server = handle_test_startup().await;

        let tag_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": unique_name("Path Rejection Tag"),
                "description": "path rejection",
                "color": "Green"
            }))
            .await;
        assert_eq!(tag_response.status_code(), 200);
        let tag = tag_response.json::<Tag>();

        let add_response = server
            .test_server
            .post(&format!("/api/v1/tags/{}/not-a-number", tag.id))
            .await;
        assert_client_error_status(add_response.status_code().as_u16());

        let remove_response = server
            .test_server
            .delete(&format!("/api/v1/tags/{}/not-a-number", tag.id))
            .await;
        assert_client_error_status(remove_response.status_code().as_u16());
    }
}
