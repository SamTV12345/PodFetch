use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::models::playlist::Playlist;
use crate::models::user::User;
use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix_web::web::Data;
use actix_web::{delete, get, post, put, web, HttpResponse};
use std::ops::DerefMut;

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistDtoPost {
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistItem {
    pub episode: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub items: Vec<PodcastEpisodeWithHistory>,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a new playlist for the user",body= Vec<PlaylistDto>)),
tag="playlist"
)]
#[post("/playlist")]
pub async fn add_playlist(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
    playlist: web::Json<PlaylistDtoPost>,
) -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::create_new_playlist(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        playlist,
        user,
    )?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates a playlist of the user",body= Vec<PlaylistDto>)),
tag="playlist"
)]
#[put("/playlist/{playlist_id}")]
pub async fn update_playlist(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
    playlist: web::Json<PlaylistDtoPost>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::update_playlist(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        playlist,
        playlist_id.clone(),
        user,
    )?;

    Ok(HttpResponse::Ok().json(res))
}

#[get("/playlist")]
pub async fn get_all_playlists(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    Playlist::get_playlists(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        requester.unwrap().into_inner().id,
    )
    .map(|playlists| HttpResponse::Ok().json(playlists))
}

#[get("/playlist/{playlist_id}")]
pub async fn get_playlist_by_id(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap();
    let playlist = Playlist::get_playlist_by_user_and_id(
        playlist_id.clone(),
        user_id.clone().into_inner(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    let playlist = Playlist::get_playlist_dto(
        playlist_id.clone(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        playlist,
        user_id.clone().into_inner(),
    )?;
    Ok(HttpResponse::Ok().json(playlist))
}

#[delete("/playlist/{playlist_id}")]
pub async fn delete_playlist_by_id(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap().id;
    Playlist::delete_playlist_by_id(
        playlist_id.clone(),
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        user_id,
    )?;
    Ok(HttpResponse::Ok().json(()))
}

#[delete("/playlist/{playlist_id}/episode/{episode_id}")]
pub async fn delete_playlist_item(
    requester: Option<web::ReqData<User>>,
    conn: Data<DbPool>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap().id;
    let unwrapped_path = path.into_inner();
    Playlist::delete_playlist_item(
        unwrapped_path.0,
        unwrapped_path.1,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        user_id,
    )
    .await?;
    Ok(HttpResponse::Ok().json(()))
}
