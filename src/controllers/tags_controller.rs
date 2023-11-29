use std::sync::Mutex;
use actix::ActorFutureExt;
use actix_web::{delete, get, HttpResponse, post, put, web};
use actix_web::web::{Data, Json};
use crate::DbPool;
use crate::models::color::Color;
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcasts::Podcast;
use crate::models::tag::Tag;
use crate::models::tags_podcast::TagsPodcast;
use crate::models::user::User;
use crate::mutex::LockResultExt;
use crate::service::mapping_service::MappingService;
use crate::utils::error::{CustomError};

#[derive(Debug, Serialize, Deserialize)]
pub struct TagCreate {
    pub name: String,
    pub description: Option<String>,
    pub color: Color
}

#[derive(Serialize, Deserialize)]
pub struct TagWithPodcast{
    pub tag: Tag,
    pub podcast: PodcastDto
}

#[post("/tags")]
pub async fn insert_tag(tag_create: Json<TagCreate>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let new_tag = Tag::new(tag_create.name.clone(), tag_create.description.clone(), tag_create.color.to_string(), requester.unwrap().username.clone());
    new_tag
        .insert_tag(&mut conn.get().unwrap())
        .map(|tag| HttpResponse::Ok().json(tag))
}

#[get("/tags")]
pub async fn get_tags(conn: Data<DbPool>, requester: Option<web::ReqData<User>>, _mapping_service: Data<Mutex<MappingService>>) ->
                                                                                  Result<HttpResponse, CustomError> {
    let tags = Tag::get_tags(&mut conn.get().unwrap(), requester.unwrap().username.clone())?;
    let mapping_service =  _mapping_service.lock().ignore_poison();
    let mapped_tags = tags.iter().map(|p|{
        TagWithPodcast{
            tag: p.0.clone(),
            podcast: mapping_service.map_podcast_to_podcast_dto_with_favorites(&(p.2.clone(),p.3.clone()))
        }
    }).collect::<Vec<TagWithPodcast>>();
    Ok(HttpResponse::Ok().json(mapped_tags))
}


#[delete("/tags/{tag_id}")]
pub async fn delete_tag(tag_id: web::Path<String>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&mut conn.get().unwrap(), &tag_id.into_inner(), &requester.unwrap().username.clone())?;
    match opt_tag{
        Some(tag) => {
            Tag::delete_tag(&mut conn.get().unwrap(), &tag.id)?;
            Ok(HttpResponse::Ok().finish())
        },
        None=>Err(CustomError::NotFound)
    }
}

#[put("/tags/{tag_id}")]
pub async fn update_tag(tag_id: web::Path<String>, tag_create: Json<TagCreate>, conn: Data<DbPool>, requester: Option<web::ReqData<User>>) ->
                                                                              Result<HttpResponse, CustomError> {
    let opt_tag = Tag::get_tag_by_id_and_username(&mut conn.get().unwrap(), &tag_id.into_inner(), &requester.unwrap().username.clone())?;
    match opt_tag{
        Some(tag) => {
            let updated_tag = Tag::update_tag(&mut conn.get().unwrap(), &tag.id, tag_create.name.clone(), tag_create.description.clone(), tag_create.color.to_string())?;
            Ok(HttpResponse::Ok().json(updated_tag))
        },
        None=>Err(CustomError::NotFound)
    }
}

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