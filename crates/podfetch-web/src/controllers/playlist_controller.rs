use crate::app_state::AppState;
use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::playlist;
use crate::playlist::PlaylistDto as WebPlaylistDto;
pub use crate::playlist::PlaylistDtoPost;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use podfetch_domain::user::User;
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub type PlaylistDto = WebPlaylistDto<PodcastEpisodeWithHistory>;

#[utoipa::path(
post,
path="/playlist",
responses(
(status = 200, description = "Adds a new playlist for the user",body= PlaylistDto)),
tag="playlist"
)]
pub async fn add_playlist(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(playlist): Json<PlaylistDtoPost>,
) -> Result<Json<PlaylistDto>, CustomError> {
    playlist::add_playlist(
        state.playlist_service.as_ref(),
        requester.id,
        requester.username.clone(),
        playlist,
    )
    .map(Json)
}

#[utoipa::path(
put,
path="/playlist/{playlist_id}",
responses(
(status = 200, description = "Updates a playlist of the user",body= PlaylistDto)),
tag="playlist"
)]
pub async fn update_playlist(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Path(playlist_id): Path<String>,
    Json(playlist): Json<PlaylistDtoPost>,
) -> Result<Json<PlaylistDto>, CustomError> {
    playlist::update_playlist(
        state.playlist_service.as_ref(),
        requester.id,
        requester.username.clone(),
        playlist_id,
        playlist,
    )
    .map(Json)
}

#[utoipa::path(
get,
path="/playlist",
responses(
(status = 200, description = "Gets all playlists of the user", body=Vec<PlaylistDto>)),
tag="playlist"
)]
pub async fn get_all_playlists(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PlaylistDto>>, CustomError> {
    playlist::get_all_playlists(
        state.playlist_service.as_ref(),
        requester.id,
        requester.username.clone(),
    )
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
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Path(playlist_id): Path<String>,
) -> Result<Json<PlaylistDto>, CustomError> {
    playlist::get_playlist_by_id(
        state.playlist_service.as_ref(),
        requester.id,
        requester.username.clone(),
        playlist_id,
    )
    .map(Json)
}

#[utoipa::path(
delete,
path="/playlist/{playlist_id}",
responses(
(status = 200, description = "Deletes a specific playlist of a user")),
tag="playlist"
)]
pub async fn delete_playlist_by_id(
    State(state): State<AppState>,
    requester: Extension<User>,
    Path(playlist_id): Path<String>,
) -> Result<StatusCode, CustomError> {
    playlist::delete_playlist_by_id(state.playlist_service.as_ref(), requester.id, playlist_id)?;
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
    State(state): State<AppState>,
    requester: Extension<User>,
    Path(path): Path<(String, i32)>,
) -> Result<StatusCode, CustomError> {
    playlist::delete_playlist_item(
        state.playlist_service.as_ref(),
        requester.id,
        path.0,
        path.1,
    )?;
    Ok(StatusCode::OK)
}

pub fn get_playlist_router() -> OpenApiRouter<AppState> {
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
    use crate::app_state::AppState;
    use crate::controllers::playlist_controller::PlaylistDtoPost;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::extract::{Path, State};
    use axum::{Extension, Json};
    use common_infrastructure::error::CustomErrorInner;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use diesel::dsl::count_star;
    use podfetch_domain::user::User;
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
    use podfetch_persistence::schema::playlist_items::dsl as pli_dsl;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;

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

    fn build_other_user() -> User {
        let mut user = UserTestDataBuilder::new().build();
        user.id = 999_999;
        user
    }

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_playlist_lifecycle_with_empty_items() {
        let server = handle_test_startup().await;
        let playlist_name = unique_name("Morning Playlist");
        let updated_playlist_name = unique_name("Updated Playlist");

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created = create_response.json::<serde_json::Value>();
        let playlist_id = created["id"].as_str().unwrap().to_string();
        assert_eq!(created["name"], json!(playlist_name.clone()));

        let list_response = server.test_server.get("/api/v1/playlist").await;
        assert_eq!(list_response.status_code(), 200);
        let playlists = list_response.json::<serde_json::Value>();
        assert!(
            playlists
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["id"] == json!(playlist_id.clone()))
        );

        let get_response = server
            .test_server
            .get(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(get_response.status_code(), 200);
        let fetched = get_response.json::<serde_json::Value>();
        assert_eq!(fetched["name"], json!(playlist_name));

        let update_response = server
            .test_server
            .put(&format!("/api/v1/playlist/{playlist_id}"))
            .json(&json!({
                "name": updated_playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);
        let updated = update_response.json::<serde_json::Value>();
        assert_eq!(updated["name"], json!(updated_playlist_name));

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 200);

        let list_after_delete = server.test_server.get("/api/v1/playlist").await;
        assert_eq!(list_after_delete.status_code(), 200);
        assert!(
            !list_after_delete
                .json::<serde_json::Value>()
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["id"] == json!(playlist_id))
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_playlist_endpoints_return_not_found_for_unknown_playlist() {
        let server = handle_test_startup().await;
        let unknown_playlist_id = "playlist-does-not-exist";

        let get_response = server
            .test_server
            .get(&format!("/api/v1/playlist/{unknown_playlist_id}"))
            .await;
        assert_eq!(get_response.status_code(), 404);

        let update_response = server
            .test_server
            .put(&format!("/api/v1/playlist/{unknown_playlist_id}"))
            .json(&json!({
                "name": "Nope",
                "items": []
            }))
            .await;
        assert_eq!(update_response.status_code(), 404);

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{unknown_playlist_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 404);

        let delete_item_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{unknown_playlist_id}/episode/1"))
            .await;
        assert_eq!(delete_item_response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_playlist_returns_forbidden_for_other_user() {
        let server = handle_test_startup().await;
        let playlist_name = unique_name("Owner Playlist");

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let result = super::update_playlist(
            State(app_state()),
            Extension(build_other_user()),
            Path(playlist_id),
            Json(PlaylistDtoPost {
                name: "Hacker Rename".to_string(),
                items: vec![],
            }),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_by_id_returns_forbidden_for_other_user() {
        let server = handle_test_startup().await;
        let playlist_name = unique_name("Owner Delete Playlist");

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let result = super::delete_playlist_by_id(
            State(app_state()),
            Extension(build_other_user()),
            Path(playlist_id),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_returns_forbidden_for_other_user() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("forbidden-item-podcast-{unique}");
        let playlist_name = unique_name("Owner Item Playlist");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Forbidden Item Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("forbidden-item-episode-{unique}"),
            &format!("forbidden-item-guid-{unique}"),
            "Forbidden Item Episode 1",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let result = super::delete_playlist_item(
            State(app_state()),
            Extension(build_other_user()),
            Path((playlist_id, episode.id)),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_removes_row() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("playlist-podcast-{unique}");
        let playlist_name = unique_name("Single Item Playlist");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Playlist Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("playlist-episode-{unique}"),
            &format!("playlist-guid-{unique}"),
            "Playlist Episode 1",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
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

    #[tokio::test]
    #[serial]
    async fn test_add_playlist_with_same_name_returns_existing_playlist() {
        let server = handle_test_startup().await;
        let playlist_name = unique_name("Duplicate Name Playlist");

        let first_create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(first_create_response.status_code(), 200);
        let first_playlist_id = first_create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let second_create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(second_create_response.status_code(), 200);
        let second_create_body = second_create_response.json::<serde_json::Value>();
        assert_eq!(second_create_body["id"], json!(first_playlist_id.clone()));

        let list_response = server.test_server.get("/api/v1/playlist").await;
        assert_eq!(list_response.status_code(), 200);
        let playlists = list_response.json::<serde_json::Value>();
        let matching_count = playlists
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["id"] == json!(first_playlist_id.clone()))
            .count();
        assert_eq!(matching_count, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_update_playlist_replaces_playlist_items() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("update-items-podcast-{unique}");
        let playlist_name = unique_name("Replace Items Playlist");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Update Items Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();

        let first_episode = insert_episode(
            podcast.id,
            &format!("update-item-episode-1-{unique}"),
            &format!("update-item-guid-1-{unique}"),
            "Update Item Episode 1",
        );
        let second_episode = insert_episode(
            podcast.id,
            &format!("update-item-episode-2-{unique}"),
            &format!("update-item-guid-2-{unique}"),
            "Update Item Episode 2",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": [{"episode": first_episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let update_response = server
            .test_server
            .put(&format!("/api/v1/playlist/{playlist_id}"))
            .json(&json!({
                "name": unique_name("Replace Items Playlist Updated"),
                "items": [{"episode": second_episode.id}]
            }))
            .await;
        assert_eq!(update_response.status_code(), 200);

        let first_episode_rows = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id.clone()))
            .filter(pli_dsl::episode.eq(first_episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(first_episode_rows, 0);

        let second_episode_rows = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(second_episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(second_episode_rows, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_with_unknown_episode_is_noop() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("noop-delete-item-podcast-{unique}");
        let playlist_name = unique_name("Noop Delete Item Playlist");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Noop Delete Item Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("noop-delete-item-episode-{unique}"),
            &format!("noop-delete-item-guid-{unique}"),
            "Noop Delete Item Episode",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let delete_missing_item_response = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/{}",
                playlist_id,
                episode.id + 1_000_000
            ))
            .await;
        assert_eq!(delete_missing_item_response.status_code(), 200);

        let rows_after = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(rows_after, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_add_playlist_with_item_returns_item_in_response() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("add-item-playlist-podcast-{unique}");
        let playlist_name = unique_name("Add Item Response Playlist");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Add Item Response Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("add-item-response-episode-{unique}"),
            &format!("add-item-response-guid-{unique}"),
            "Add Item Response Episode",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created = create_response.json::<serde_json::Value>();
        let playlist_id = created["id"].as_str().unwrap();

        let inserted_rows = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(inserted_rows, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_playlist_by_id_returns_not_found_for_other_user() {
        let server = handle_test_startup().await;
        let playlist_name = unique_name("Owner Get Playlist");

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": playlist_name,
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let result = super::get_playlist_by_id(
            State(app_state()),
            Extension(build_other_user()),
            Path(playlist_id),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::NotFound(_))),
            Ok(_) => panic!("expected not found error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_all_playlists_returns_empty_for_user_without_playlists() {
        let server = handle_test_startup().await;
        let _guard = &server.mutex;

        let result =
            super::get_all_playlists(State(app_state()), Extension(build_other_user())).await;
        match result {
            Ok(Json(playlists)) => assert!(playlists.is_empty()),
            Err(err) => panic!("expected empty result, got error: {err}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_add_playlist_endpoint_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let missing_name_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "items": []
            }))
            .await;
        assert_client_error_status(missing_name_response.status_code().as_u16());

        let wrong_item_type_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Invalid Payload Playlist"),
                "items": [{"episode": "not-a-number"}]
            }))
            .await;
        assert_client_error_status(wrong_item_type_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_update_playlist_endpoint_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Update Invalid Payload Playlist"),
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let update_response = server
            .test_server
            .put(&format!("/api/v1/playlist/{playlist_id}"))
            .json(&json!({
                "name": 123,
                "items": []
            }))
            .await;
        assert_client_error_status(update_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_rejects_non_numeric_episode_id() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Bad Episode Path Playlist"),
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let delete_response = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/not-a-number",
                playlist_id
            ))
            .await;
        assert_client_error_status(delete_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_add_playlist_persists_item_positions_in_order() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("playlist-order-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Playlist Order Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let first_episode = insert_episode(
            podcast.id,
            &format!("playlist-order-episode-1-{unique}"),
            &format!("playlist-order-guid-1-{unique}"),
            "Playlist Order Episode 1",
        );
        let second_episode = insert_episode(
            podcast.id,
            &format!("playlist-order-episode-2-{unique}"),
            &format!("playlist-order-guid-2-{unique}"),
            "Playlist Order Episode 2",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Ordered Playlist"),
                "items": [
                    {"episode": second_episode.id},
                    {"episode": first_episode.id}
                ]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let persisted_items = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .order(pli_dsl::position.asc())
            .select((pli_dsl::episode, pli_dsl::position))
            .load::<(i32, i32)>(&mut get_connection())
            .unwrap();

        assert_eq!(persisted_items.len(), 2);
        assert_eq!(persisted_items[0], (second_episode.id, 0));
        assert_eq!(persisted_items[1], (first_episode.id, 1));
    }

    #[tokio::test]
    #[serial]
    async fn test_playlist_endpoints_return_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Method Mismatch Playlist"),
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let post_on_id_response = server
            .test_server
            .post(&format!("/api/v1/playlist/{playlist_id}"))
            .json(&json!({"name": "noop", "items": []}))
            .await;
        assert_client_error_status(post_on_id_response.status_code().as_u16());

        let get_delete_item_route_response = server
            .test_server
            .get(&format!("/api/v1/playlist/{playlist_id}/episode/1"))
            .await;
        assert_client_error_status(get_delete_item_route_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_rejects_episode_id_overflow() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Overflow Episode Path Playlist"),
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let response = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/2147483648",
                playlist_id
            ))
            .await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_playlist_by_id_returns_not_found_after_playlist_deletion() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Delete Then Get Playlist"),
                "items": []
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 200);

        let get_response = server
            .test_server
            .get(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(get_response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_returns_not_found_after_playlist_deletion() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("delete-item-after-delete-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Delete Item After Delete Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("delete-item-after-delete-episode-{unique}"),
            &format!("delete-item-after-delete-guid-{unique}"),
            "Delete Item After Delete Episode",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Delete Item After Delete Playlist"),
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let delete_playlist_response = server
            .test_server
            .delete(&format!("/api/v1/playlist/{playlist_id}"))
            .await;
        assert_eq!(delete_playlist_response.status_code(), 200);

        let delete_item_response = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/{}",
                playlist_id, episode.id
            ))
            .await;
        assert_eq!(delete_item_response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_playlist_item_endpoint_is_idempotent_for_same_episode() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("idempotent-delete-item-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Idempotent Delete Item Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("idempotent-delete-item-episode-{unique}"),
            &format!("idempotent-delete-item-guid-{unique}"),
            "Idempotent Delete Item Episode",
        );

        let create_response = server
            .test_server
            .post("/api/v1/playlist")
            .json(&json!({
                "name": unique_name("Idempotent Delete Item Playlist"),
                "items": [{"episode": episode.id}]
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let playlist_id = create_response.json::<serde_json::Value>()["id"]
            .as_str()
            .unwrap()
            .to_string();

        let first_delete = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/{}",
                playlist_id, episode.id
            ))
            .await;
        assert_eq!(first_delete.status_code(), 200);

        let second_delete = server
            .test_server
            .delete(&format!(
                "/api/v1/playlist/{}/episode/{}",
                playlist_id, episode.id
            ))
            .await;
        assert_eq!(second_delete.status_code(), 200);

        let rows_after = pli_dsl::playlist_items
            .filter(pli_dsl::playlist_id.eq(playlist_id))
            .filter(pli_dsl::episode.eq(episode.id))
            .select(count_star())
            .get_result::<i64>(&mut get_connection())
            .unwrap();
        assert_eq!(rows_after, 0);
    }
}
