use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::models::playlist::Playlist;
use crate::models::user::User;
use crate::utils::error::CustomError;
use axum::extract::Path;
use axum::{Extension, Json};
use reqwest::StatusCode;
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
(status = 200, description = "Adds a new playlist for the user",body= PlaylistDto)),
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
(status = 200, description = "Updates a playlist of the user",body= PlaylistDto)),
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
(status = 200, description = "Gets all playlists of the user", body=Vec<PlaylistDto>)),
tag="playlist"
)]
pub async fn get_all_playlists(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PlaylistDto>>, CustomError> {
    Playlist::get_playlists(requester.id)
        .map(|p| {
            p.iter()
                .map(|p| Playlist::get_playlist_dto(p.id.clone(), p.clone(), requester.clone()))
                .collect::<Result<Vec<PlaylistDto>, CustomError>>()
                .unwrap()
        })
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
    let playlist = Playlist::get_playlist_by_user_and_id(playlist_id.clone(), requester.clone())?;
    let playlist = Playlist::get_playlist_dto(playlist_id.clone(), playlist, requester.clone())?;
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

#[cfg(test)]
mod tests {
    use crate::adapters::persistence::dbconfig::db::get_connection;
    use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl as pli_dsl;
    use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::models::podcasts::Podcast;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel::dsl::count_star;
    use serde_json::json;
    use serial_test::serial;

    fn insert_episode(
        podcast_id: i32,
        episode_id: &str,
        guid: &str,
        title: &str,
    ) -> PodcastEpisode {
        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::podcast_id.eq(podcast_id),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq(title.to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("playlist test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_playlist_lifecycle_with_empty_items() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": "Morning Playlist",
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created = create_response.json::<serde_json::Value>();
        let playlist_id = created["id"].as_str().unwrap().to_string();
        assert_eq!(created["name"], json!("Morning Playlist"));

        let list_response = server.test_server.get("/api/v1/playlist").await;
        assert_eq!(list_response.status_code(), 200);
        let playlists = list_response.json::<serde_json::Value>();
        assert_eq!(playlists.as_array().unwrap().len(), 1);
        assert_eq!(playlists[0]["id"], json!(playlist_id.clone()));

        let get_response = server
            .test_server
            .get(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(get_response.status_code(), 200);
        let fetched = get_response.json::<serde_json::Value>();
        assert_eq!(fetched["name"], json!("Morning Playlist"));

        let update_response = server
            .test_server
            .put(&format!("/api/v1/playlist/{playlist_id}"))
            .json(&json!({
                "name": "Updated Playlist",
                "items": []
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated = update_response.json::<serde_json::Value>();
        assert_eq!(updated["name"], json!("Updated Playlist"));

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 200);

        let list_after_delete = server.test_server.get("/api/v1/playlist").await;
        assert_eq!(list_after_delete.status_code(), 200);
        assert!(
            list_after_delete
                .json::<serde_json::Value>()
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_removes_row() {
        let server = handle_test_startup().await;

        let podcast = Podcast::add_podcast_to_database(
            "Playlist Podcast",
            "playlist-podcast",
            "https://example.com/playlist.xml",
            "http://localhost:8080/ui/default.jpg",
            "playlist-podcast",
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            "playlist-episode-1",
            "playlist-guid-1",
            "Playlist Episode 1",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": "Single Item Playlist",
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created = create_response.json::<serde_json::Value>();
        let playlist_id = created["id"].as_str().unwrap();

        let rows_before = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(rows_before, 1);

        let delete_item_response = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/{}",
                playlist_id, episode.id
            ))
            .await;
        assert_eq!(delete_item_response.status_code(), 200);

        let rows_after = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(rows_after, 0);
    }
}
