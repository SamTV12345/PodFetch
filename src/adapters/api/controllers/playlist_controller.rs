use crate::models::playlist::Playlist;
use crate::models::user::User;
use crate::utils::error::CustomError;
use actix_web::{delete, get, post, put, web, HttpResponse};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistDtoPost {
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Serialize, Deserialize, Clone,ToSchema)]
pub struct PlaylistItem {
    pub episode: i32,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistDto {
    pub id: String,
    pub name: String,
    pub items: Vec<PodcastEpisodeWithHistory>,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a new playlist for the user",body= PlaylistDtoPost)),
tag="playlist"
)]
#[post("/playlist")]
pub async fn add_playlist(
    requester: Option<web::ReqData<User>>,
    playlist: web::Json<PlaylistDtoPost>,
) -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::create_new_playlist(
        playlist,
        user,
    )?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Updates a playlist of the user",body= PlaylistDtoPost)),
tag="playlist"
)]
#[put("/playlist/{playlist_id}")]
pub async fn update_playlist(
    requester: Option<web::ReqData<User>>,
    playlist: web::Json<PlaylistDtoPost>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let playlist = playlist.into_inner();

    let res = Playlist::update_playlist(
        playlist,
        playlist_id.clone(),
        user,
    )?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all playlists of the user")),
tag="playlist"
)]
#[get("/playlist")]
pub async fn get_all_playlists(
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    Playlist::get_playlists(
        requester.unwrap().into_inner().id,
    )
    .map(|playlists| HttpResponse::Ok().json(playlists))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets a specific playlist of a user")),
tag="playlist"
)]
#[get("/playlist/{playlist_id}")]
pub async fn get_playlist_by_id(
    requester: Option<web::ReqData<User>>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap();
    let playlist = Playlist::get_playlist_by_user_and_id(
        playlist_id.clone(),
        user_id.clone().into_inner(),
    )?;
    let playlist = Playlist::get_playlist_dto(
        playlist_id.clone(),
        playlist,
        user_id.clone().into_inner(),
    )?;
    Ok(HttpResponse::Ok().json(playlist))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a specific playlist of a user")),
tag="playlist"
)]
#[delete("/playlist/{playlist_id}")]
pub async fn delete_playlist_by_id(
    requester: Option<web::ReqData<User>>,
    playlist_id: web::Path<String>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap().id;
    Playlist::delete_playlist_by_id(
        playlist_id.clone(),
        user_id,
    )?;
    Ok(HttpResponse::Ok().json(()))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a specific playlist item of a user")),
tag="playlist"
)]
#[delete("/playlist/{playlist_id}/episode/{episode_id}")]
pub async fn delete_playlist_item(
    requester: Option<web::ReqData<User>>,
    path: web::Path<(String, i32)>,
) -> Result<HttpResponse, CustomError> {
    let user_id = requester.clone().unwrap().id;
    let unwrapped_path = path.into_inner();
    Playlist::delete_playlist_item(
        unwrapped_path.0,
        unwrapped_path.1,
        user_id,
    )
    .await?;
    Ok(HttpResponse::Ok().json(()))
}
