use actix_web::{HttpResponse, post, put, web};
use actix_web::web::Data;
use crate::DbPool;
use crate::models::playlist::Playlist;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::user::User;
use crate::utils::error::CustomError;

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistDtoPost {
    pub name: String,
    pub items: Vec<PlaylistItem>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistItem {
    pub episode: i32
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub items: Vec<PodcastEpisode>
}


#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a new playlist for the user",body= Vec<PlaylistDto>)),
tag="playlist"
)]
#[post("/playlist")]
pub async fn add_playlist(requester: Option<web::ReqData<User>>, conn: Data<DbPool>, playlist: web::Json<PlaylistDtoPost>)
    -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::create_new_playlist(&mut conn.get().unwrap(),
                                            playlist, user.id)?;


    return Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates a playlist of the user",body= Vec<PlaylistDto>)),
tag="playlist"
)]
#[put("/playlist/{playlist_id}")]
pub async fn update_playlist(requester: Option<web::ReqData<User>>, conn: Data<DbPool>,
                             playlist: web::Json<PlaylistDtoPost>, playlist_id: web::Path<String>)
                             -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::update_playlist(&mut conn.get().unwrap(),
                                            playlist, playlist_id.clone(),user.id)?;


    return Ok(HttpResponse::Ok().json(res))
}
