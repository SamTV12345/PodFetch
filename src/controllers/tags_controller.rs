use axum::{Extension, Json, Router};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use crate::models::color::Color;
use crate::models::podcast_dto::PodcastDto;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::models::user::User;
use crate::utils::error::{CustomError, CustomErrorInner};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagCreate {
    pub name: String,
    pub description: Option<String>,
    pub color: Color,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TagWithPodcast {
    pub tag: Tag,
    pub podcast: PodcastDto,
}

#[utoipa::path(
post,
path="/tags",
context_path="/api/v1",
responses(
(status = 200, description = "Creates a new tag",
body = TagCreate)),
tag="tags"
)]
pub async fn insert_tag(
    tag_create: Json<TagCreate>,
    requester: Extension<User>,
) -> Result<Json<Tag>, CustomError> {
    let new_tag = Tag::new(
        tag_create.name.clone(),
        tag_create.description.clone(),
        tag_create.color.to_string(),
        requester.username.clone(),
    );
    new_tag.insert_tag().map(|tag| Json(tag))
}

#[utoipa::path(
get,
path="/tags",
context_path="/api/v1",
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
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a tag by id")),
tag="tags"
)]
pub async fn delete_tag(
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    let opt_tag =
        Tag::get_tag_by_id_and_username(&tag_id, &requester.username.clone())?;
    match opt_tag {
        Some(tag) => {
            TagsPodcast::delete_tag_podcasts(&tag.id)?;
            Tag::delete_tag(&tag.id)?;
            Ok(StatusCode::OK)
        }
        None => Err(CustomErrorInner::NotFound.into()),
    }
}

#[utoipa::path(
put,
path="/tags/{tag_id}",
context_path="/api/v1",
responses(
(status = 200, description = "Updates a tag by id")),
tag="tags"
)]
pub async fn update_tag(
    tag_id: Path<String>,
    tag_create: Json<TagCreate>,
    Extension(requester): Extension<User>,
) -> Result<Json<Tag>, CustomError> {
    let opt_tag =
        Tag::get_tag_by_id_and_username(&tag_id, &requester.username)?;
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
        None => Err(CustomErrorInner::NotFound.into()),
    }
}

#[utoipa::path(
post,
path="/tags/{tag_id}/{podcast_id}",
context_path="/api/v1",
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
        None => Err(CustomErrorInner::NotFound.into()),
    }
}

#[utoipa::path(
delete,
path="/tags/{tag_id}/{podcast_id}",
context_path="/api/v1",
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
        None => Err(CustomErrorInner::NotFound.into()),
    }
}


pub fn get_tags_router() -> Router {
    Router::new()
        .route("/tags", post(insert_tag))
        .route("/tags", get(get_tags))
        .route("/tags/{tag_id}", delete(delete_tag))
        .route("/tags/{tag_id}", put(update_tag))
        .route("/tags/{tag_id}/{podcast_id}", post(add_podcast_to_tag))
        .route("/tags/{tag_id}/{podcast_id}", delete(delete_podcast_from_tag))
}
