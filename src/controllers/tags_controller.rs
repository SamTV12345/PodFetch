use std::sync::Mutex;
use actix_web::{delete, get, HttpResponse, post, put, web};
use actix_web::web::{Data, Json};
use utoipa::ToSchema;
use crate::DbPool;
use crate::models::color::Color;
use crate::models::podcast_dto::PodcastDto;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::models::user::User;
use crate::service::mapping_service::MappingService;
use crate::utils::error::{CustomError};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TagCreate {
    pub name: String,
    pub description: Option<String>,
    pub color: Color
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TagWithPodcast{
    pub tag: Tag,
    pub podcast: PodcastDto
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Creates a new tag",
body = TagCreate)),
tag="tags"
)]
#[post("/tags")]
pub async fn insert_tag(tag_create: Json<TagCreate>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let new_tag = Tag::new(tag_create.name.clone(), tag_create.description.clone(), tag_create.color.to_string(), requester.unwrap().username.clone());
    new_tag
        .insert_tag(&mut conn.get().unwrap())
        .map(|tag| HttpResponse::Ok().json(tag))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all tags of a user", body=Vec<Tag>)),
tag="tags"
)]
#[get("/tags")]
pub async fn get_tags(conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                                  Result<HttpResponse, CustomError> {
    let tags = Tag::get_tags(&mut conn.get().unwrap(), requester.unwrap().username.clone())?;
    Ok(HttpResponse::Ok().json(tags))
}


#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a tag by id")),
tag="tags"
)]
#[delete("/tags/{tag_id}")]
pub async fn delete_tag(tag_id: web::Path<String>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&mut conn.get().unwrap(), &tag_id.into_inner(), &requester.unwrap().username.clone())?;
    match opt_tag{
        Some(tag) => {
            TagsPodcast::delete_tag_podcasts(&mut conn.get().unwrap(), &tag.id)?;
            Tag::delete_tag(&mut conn.get().unwrap(), &tag.id)?;
            Ok(HttpResponse::Ok().finish())
        },
        None=>Err(CustomError::NotFound)
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates a tag by id")),
tag="tags"
)]
#[put("/tags/{tag_id}")]
pub async fn update_tag(tag_id: web::Path<String>, tag_create: Json<TagCreate>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&mut conn.get().unwrap(), &tag_id.into_inner(), &requester.unwrap().username.clone())?;
    match opt_tag {
        Some(tag) => {
            let updated_tag = Tag::update_tag(&mut conn.get().unwrap(), &tag.id, tag_create.name.clone(),
                                              tag_create.description.clone(), tag_create.color.to_string())?;
            Ok(HttpResponse::Ok().json(updated_tag))
        },
        None=>Err(CustomError::NotFound)
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a podcast to a tag", body=TagsPodcast)),
tag="tags"
)]
#[post("/tags/{tag_id}/{podcast_id}")]
pub async fn add_podcast_to_tag(tag_id: web::Path<String>, podcast_id: web::Path<i32>, conn:
Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                                      Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username( &mut conn.get().unwrap(), &tag_id.into_inner(),
                                                &requester.unwrap().username.clone())?;
    match opt_tag{
        Some(tag) => {
            let podcast = TagsPodcast::add_podcast_to_tag(tag.id.clone(), podcast_id.into_inner(),
                                                       &mut conn.get().unwrap())?;
            Ok(HttpResponse::Ok().json(podcast))
        },
       None=>Err(CustomError::NotFound)
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a podcast from a tag")),
tag="tags"
)]
#[delete("/tags/{tag_id}/{podcast_id}")]
pub async fn delete_podcast_from_tag(tag_id: web::Path<String>, podcast_id: web::Path<i32>, conn:
Data<DbPool>, requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username( &mut conn.get().unwrap(), &tag_id.into_inner(),
                                                &requester.unwrap().username.clone())?;
    match opt_tag{
        Some(tag) => {
            TagsPodcast::delete_tag_podcasts_by_podcast_id_tag_id(&mut conn.get().unwrap(),  podcast_id.into_inner(),
                                                                  &tag.id)?;
            Ok(HttpResponse::Ok().finish())
        },
        None=>Err(CustomError::NotFound)
    }
}