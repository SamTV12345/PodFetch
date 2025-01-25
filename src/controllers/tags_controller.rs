use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use crate::models::color::Color;
use crate::models::podcast_dto::PodcastDto;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::models::user::User;
use crate::utils::error::{CustomError, CustomErrorInner};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
responses(
(status = 200, description = "Creates a new tag",
body = TagCreate)),
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
responses(
(status = 200, description = "Updates a tag by id")),
tag="tags"
)]
pub async fn update_tag(
    Path(tag_id): Path<String>,
    Extension(requester): Extension<User>,
    Json(tag_create): Json<TagCreate>,
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


pub fn get_tags_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(insert_tag))
        .routes(routes!(get_tags))
        .routes(routes!(delete_tag))
        .routes(routes!(update_tag))
        .routes(routes!(add_podcast_to_tag))
        .routes(routes!(delete_podcast_from_tag))
}
