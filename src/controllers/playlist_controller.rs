use axum::{Extension, Json};
use axum::extract::Path;
use reqwest::StatusCode;
use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::models::playlist::Playlist;
use crate::models::user::User;
use crate::utils::error::CustomError;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct PlaylistDtoPost {
    pub name: String,
    pub items: Vec<PlaylistItem>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
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
post,
path="/playlist",
responses(
(status = 200, description = "Adds a new playlist for the user",body= PlaylistDtoPost)),
tag="playlist"
)]
pub async fn add_playlist(
    Extension(requester): Extension<User>,
    Json(playlist): Json<PlaylistDtoPost>,
) -> Result<Json<PlaylistDto>, CustomError> {

    let res = Playlist::create_new_playlist(playlist, requester)?;

    Ok(Json(res))
}

#[utoipa::path(
put,
path="/playlist/{playlist_id}",
responses(
(status = 200, description = "Updates a playlist of the user",body= PlaylistDtoPost)),
tag="playlist"
)]
pub async fn update_playlist(
    Extension(requester): Extension<User>,
    Path(playlist_id): Path<String>,
    Json(playlist): Json<PlaylistDtoPost>,
) -> Result<Json<PlaylistDto>, CustomError> {

    let res = Playlist::update_playlist(playlist, playlist_id.clone(), requester)?;

    Ok(Json(res))
}

#[utoipa::path(
get,
path="/playlist",
responses(
(status = 200, description = "Gets all playlists of the user")),
tag="playlist"
)]
pub async fn get_all_playlists(Extension(requester): Extension<User>) ->
                                                                      Result<Json<Vec<Playlist>>,
    CustomError> {
    Playlist::get_playlists(requester.id)
        .map(Json)
}

#[utoipa::path(
get,
path="/playlist/{playlist_id}",
responses(
(status = 200, description = "Gets a specific playlist of a user", body = PlaylistDto)),
tag="playlist"
)]
pub async fn get_playlist_by_id(
    Extension(requester): Extension<User>,
    Path(playlist_id): Path<String>,
) -> Result<Json<PlaylistDto>, CustomError> {
    let playlist =
        Playlist::get_playlist_by_user_and_id(playlist_id.clone(), requester.clone())?;
    let playlist =
        Playlist::get_playlist_dto(playlist_id.clone(), playlist, requester.clone())?;
    Ok(Json(playlist))
}

#[utoipa::path(
delete,
path="/playlist/{playlist_id}",
responses(
(status = 200, description = "Deletes a specific playlist of a user")),
tag="playlist"
)]
pub async fn delete_playlist_by_id(
    requester: Extension<User>,
    Path(playlist_id): Path<String>,
) -> Result<StatusCode, CustomError> {
    let user_id = requester.id;
    Playlist::delete_playlist_by_id(playlist_id, user_id)?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
delete,
path="/playlist/{playlist_id}/episode/{episode_id}",
responses(
(status = 200, description = "Deletes a specific playlist item of a user")),
tag="playlist"
)]
pub async fn delete_playlist_item(
    requester: Extension<User>,
    Path(path): Path<(String, i32)>,
) -> Result<StatusCode, CustomError> {
    let user_id = requester.id;
    Playlist::delete_playlist_item(path.0, path.1, user_id).await?;
    Ok(StatusCode::OK)
}

pub fn get_playlist_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_all_playlists))
        .routes(routes!(add_playlist))
        .routes(routes!(get_playlist_by_id))
        .routes(routes!(update_playlist))
        .routes(routes!(delete_playlist_by_id))
        .routes(routes!(delete_playlist_item))
}