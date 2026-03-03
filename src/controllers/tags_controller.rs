use crate::models::color::Color;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::models::user::User;
use crate::utils::error::ErrorSeverity::Debug;
use crate::utils::error::{CustomError, CustomErrorInner};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagCreate {
    pub name: String,
    pub description: Option<String>,
    pub color: Color,
}

#[utoipa::path(
post,
path="/tags",
responses(
(status = 200, description = "Creates a new tag",
body = Tag)),
tag="tags"
)]
pub async fn insert_tag(
    Extension(requester): Extension<User>,
    Json(tag_create): Json<TagCreate>,
) -> Result<Json<Tag>, CustomError> {
    let new_tag = Tag::new(
        tag_create.name.clone(),
        tag_create.description.clone(),
        tag_create.color.to_string(),
        requester.username.clone(),
    );
    new_tag.insert_tag().map(Json)
}

#[utoipa::path(
get,
path="/tags",
responses(
(status = 200, description = "Gets all tags of a user", body=Vec<Tag>)),
tag="tags"
)]
pub async fn get_tags(requester: Extension<User>) -> Result<Json<Vec<Tag>>, CustomError> {
    let tags = Tag::get_tags(requester.username.clone())?;
    Ok(Json(tags))
}

#[utoipa::path(
delete,
path="/tags/{tag_id}",
responses(
(status = 200, description = "Deletes a tag by id")),
tag="tags"
)]
pub async fn delete_tag(
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&tag_id, &requester.username.clone())?;
    match opt_tag {
        Some(tag) => {
            TagsPodcast::delete_tag_podcasts(&tag.id)?;
            Tag::delete_tag(&tag.id)?;
            Ok(StatusCode::OK)
        }
        None => Err(CustomErrorInner::NotFound(Debug).into()),
    }
}

#[utoipa::path(
put,
path="/tags/{tag_id}",
responses(
(status = 200, description = "Updates a tag by id")),
tag="tags"
)]
pub async fn update_tag(
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
    Json(tag_create): Json<TagCreate>,
) -> Result<Json<Tag>, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&tag_id, &requester.username)?;
    match opt_tag {
        Some(tag) => {
            let updated_tag = Tag::update_tag(
                &tag.id,
                tag_create.name.clone(),
                tag_create.description.clone(),
                tag_create.color.to_string(),
            )?;
            Ok(Json(updated_tag))
        }
        None => Err(CustomErrorInner::NotFound(Debug).into()),
    }
}

#[utoipa::path(
post,
path="/tags/{tag_id}/{podcast_id}",
responses(
(status = 200, description = "Adds a podcast to a tag", body=TagsPodcast)),
tag="tags"
)]
pub async fn add_podcast_to_tag(
    Path(tag_id_to_convert): Path<(String, i32)>,
    requester: Extension<User>,
) -> Result<Json<TagsPodcast>, CustomError> {
    let (tag_id, podcast_id) = tag_id_to_convert;
    let opt_tag = Tag::get_tag_by_id_and_username(&tag_id, &requester.username.clone())?;
    match opt_tag {
        Some(tag) => {
            let podcast = TagsPodcast::add_podcast_to_tag(tag.id.clone(), podcast_id)?;
            Ok(Json(podcast))
        }
        None => Err(CustomErrorInner::NotFound(Debug).into()),
    }
}

#[utoipa::path(
delete,
path="/tags/{tag_id}/{podcast_id}",
responses(
(status = 200, description = "Deletes a podcast from a tag")),
tag="tags"
)]
pub async fn delete_podcast_from_tag(
    Path(tag_id): Path<(String, i32)>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    let (tag_id, podcast_id) = tag_id;

    let opt_tag = Tag::get_tag_by_id_and_username(&tag_id, &requester.username.clone())?;
    match opt_tag {
        Some(tag) => {
            TagsPodcast::delete_tag_podcasts_by_podcast_id_tag_id(podcast_id, &tag.id)?;
            Ok(StatusCode::OK)
        }
        None => Err(CustomErrorInner::NotFound(Debug).into()),
    }
}

pub fn get_tags_router() -> OpenApiRouter {
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
    use super::Tag;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
    use crate::models::podcasts::Podcast;
    use serde_json::json;
    use serial_test::serial;

    fn admin_username() -> String {
        ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string())
    }

    #[tokio::test]
    #[serial]
    async fn test_insert_update_and_delete_tag() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": "Backend",
                "description": "API related",
                "color": "Red"
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created_tag = create_response.json::<Tag>();
        assert_eq!(created_tag.name, "Backend");
        assert_eq!(created_tag.description, Some("API related".to_string()));

        let update_response = server
            .test_server
            .put(&format!("/api/v1/tags/{}", created_tag.id))
            .json(&json!({
                "name": "Backend Updated",
                "description": "API and DB",
                "color": "Blue"
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated_tag = update_response.json::<Tag>();
        assert_eq!(updated_tag.name, "Backend Updated");
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

        let podcast = Podcast::add_podcast_to_database(
            "Tagged Podcast",
            "tagged-podcast",
            "https://example.com/tagged-feed.xml",
            "http://localhost:8080/ui/default.jpg",
            "tagged-podcast",
        )
        .unwrap();

        let tag_response = server
            .test_server
            .post("/api/v1/tags")
            .json(&json!({
                "name": "Favorites",
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

        let tags_for_podcast = Tag::get_tags_of_podcast(podcast.id, &username).unwrap();
        assert_eq!(tags_for_podcast.len(), 1);
        assert_eq!(tags_for_podcast[0].id, tag.id);

        let remove_response = server
            .test_server
            .delete(&format!("/api/v1/tags/{}/{}", tag.id, podcast.id))
            .await;
        assert_eq!(remove_response.status_code(), 200);

        let tags_after_remove = Tag::get_tags_of_podcast(podcast.id, &username).unwrap();
        assert!(tags_after_remove.is_empty());
    }
}
