//! Integration tests for the audiobookshelf-compatible API.
//!
//! Exercised against a real database (sqlite by default, postgres when the
//! `postgresql` feature is active) via the shared `TestServerWrapper` harness.

#![cfg(test)]

use crate::app_state::AppState;
use crate::audiobookshelf_api::test_support::tests::login_audiobookshelf;
use crate::test_support::tests::handle_test_startup;
use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
use serde_json::{Value, json};
use serial_test::serial;

fn create_user_for_audiobookshelf(state: &AppState) -> podfetch_domain::user::User {
    let user = state
        .user_admin_service
        .create_user(UserTestDataBuilder::new().build())
        .expect("create test user");
    // `UserTestDataBuilder` defaults every user to api_key="api_key" — colliding
    // tokens would let user B authenticate as user A. Stamp a unique api_key
    // per test user so the bearer middleware resolves to the right account.
    let mut updated = user.clone();
    updated.api_key = Some(format!("test_apikey_{}", uuid::Uuid::new_v4().simple()));
    state
        .user_admin_service
        .update_user(updated)
        .expect("assign unique api_key for test user")
}

// ── /ping + /status (unauthenticated) ───────────────────────────────────────

#[tokio::test]
#[serial]
async fn ping_returns_success_true() {
    let server = handle_test_startup().await;
    let response = server.test_server.get("/ping").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["success"], json!(true));
}

#[tokio::test]
#[serial]
async fn status_returns_server_payload() {
    let server = handle_test_startup().await;
    let response = server.test_server.get("/status").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["isInit"], json!(true));
    assert!(body["authMethods"].is_array());
    assert!(body["serverVersion"].is_string());
}

// ── /login ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_happy_path_returns_bearer_and_libraries_bootstrap() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let token = login_audiobookshelf(&mut server, &user).await;
    assert!(!token.is_empty(), "expected non-empty token");
    // Token must authenticate subsequent calls
    let me_resp = server.test_server.get("/api/me").await;
    assert_eq!(me_resp.status_code().as_u16(), 200);

    // Default-libraries bootstrap ran during build_server_router(): list contains
    // a Podcasts library plus an Audiobooks one.
    let libraries = state
        .audiobookshelf_library_service
        .list()
        .expect("list libraries");
    let media_types: Vec<&str> = libraries
        .iter()
        .map(|l| l.media_type.as_str())
        .collect();
    assert!(media_types.contains(&"podcast"));
    assert!(media_types.contains(&"book"));
}

#[tokio::test]
#[serial]
async fn login_wrong_password_is_rejected() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    server.test_server.clear_headers();
    let response = server
        .test_server
        .post("/login")
        .json(&json!({ "username": user.username, "password": "not-the-password" }))
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx, got {}",
        response.status_code()
    );
}

#[tokio::test]
#[serial]
async fn login_unknown_user_is_rejected() {
    let mut server = handle_test_startup().await;
    let _state = AppState::new();
    server.test_server.clear_headers();
    let response = server
        .test_server
        .post("/login")
        .json(&json!({ "username": "ghost-user-xyz", "password": "irrelevant" }))
        .await;
    assert!(response.status_code().is_client_error());
}

// ── Bearer middleware: protected routes refuse missing / bad tokens ────────

#[tokio::test]
#[serial]
async fn protected_route_without_token_is_unauthorized() {
    let mut server = handle_test_startup().await;
    server.test_server.clear_headers();
    let response = server.test_server.get("/api/libraries").await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx, got {}",
        response.status_code()
    );
}

#[tokio::test]
#[serial]
async fn protected_route_with_bogus_token_is_unauthorized() {
    let mut server = handle_test_startup().await;
    server.test_server.clear_headers();
    server
        .test_server
        .add_header("Authorization", "Bearer not-a-real-token");
    let response = server.test_server.get("/api/libraries").await;
    assert!(response.status_code().is_client_error());
}

// ── /api/authorize and /logout ─────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn authorize_with_valid_token_returns_user() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let _token = login_audiobookshelf(&mut server, &user).await;
    let response = server.test_server.post("/api/authorize").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["user"]["username"], json!(user.username));
}

#[tokio::test]
#[serial]
async fn logout_returns_ok_for_authenticated_user() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server.test_server.post("/logout").await;
    assert_eq!(response.status_code().as_u16(), 200);
}

// ── /api/libraries ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn list_libraries_returns_default_bootstrap() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let response = server.test_server.get("/api/libraries").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    let libraries = body["libraries"].as_array().expect("libraries array");
    assert!(libraries.len() >= 2, "expected >=2 libraries, got {libraries:#?}");
    assert!(
        libraries
            .iter()
            .any(|l| l["mediaType"] == json!("podcast")),
        "no podcast library found in {libraries:#?}"
    );
}

#[tokio::test]
#[serial]
async fn get_library_by_id_returns_dto() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let podcasts_library = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()
        .expect("repo call")
        .expect("bootstrap created podcast library");
    let response = server
        .test_server
        .get(&format!("/api/libraries/{}", podcasts_library.id))
        .await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["id"], json!(podcasts_library.id));
    assert_eq!(body["mediaType"], json!("podcast"));
}

#[tokio::test]
#[serial]
async fn get_library_nonexistent_is_404() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server
        .test_server
        .get("/api/libraries/lib_does_not_exist")
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx, got {}",
        response.status_code()
    );
}

// ── /api/me ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn me_returns_current_user_with_token() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let token = login_audiobookshelf(&mut server, &user).await;

    let response = server.test_server.get("/api/me").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["username"], json!(user.username));
    assert_eq!(body["token"], json!(token));
    assert!(body["mediaProgress"].is_array());
    assert!(body["permissions"]["accessAllLibraries"].as_bool().unwrap());
}

// ── /api/libraries/:id/items (over existing podcasts) ──────────────────────

#[tokio::test]
#[serial]
async fn list_library_items_lists_podcasts_as_library_items() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("List test podcast", &state, user.id);

    login_audiobookshelf(&mut server, &user).await;
    let lib = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()
        .unwrap()
        .unwrap();
    let response = server
        .test_server
        .get(&format!("/api/libraries/{}/items", lib.id))
        .await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    let results = body["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1, "expected one library item for inserted podcast");
    let expected_id = format!("li_pod_{}", podcast.id);
    assert_eq!(results[0]["id"], json!(expected_id));
    assert_eq!(results[0]["mediaType"], json!("podcast"));
    assert_eq!(results[0]["media"]["metadata"]["title"], json!(podcast.name));
}

#[tokio::test]
#[serial]
async fn get_item_by_id_returns_podcast_with_episodes() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Detail test podcast", &state, user.id);
    insert_test_episode(podcast.id, "Test Episode 1");

    login_audiobookshelf(&mut server, &user).await;
    let response = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["mediaType"], json!("podcast"));
    let episodes = body["media"]["episodes"].as_array().expect("episodes array");
    assert_eq!(episodes.len(), 1);
    assert_eq!(episodes[0]["title"], json!("Test Episode 1"));
    assert!(
        episodes[0]["id"].as_str().unwrap().starts_with("ep_"),
        "episode id format: {:?}",
        episodes[0]["id"]
    );
}

#[tokio::test]
#[serial]
async fn get_item_unknown_id_is_404() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server.test_server.get("/api/items/li_pod_99999").await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx, got {}",
        response.status_code()
    );
}

// ── Sessions: play → sync → close → media_progress persisted ───────────────

#[tokio::test]
#[serial]
async fn play_sync_close_updates_media_progress() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Session test podcast", &state, user.id);
    let episode = insert_test_episode(podcast.id, "Session Episode");

    login_audiobookshelf(&mut server, &user).await;

    // Start session
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "mediaPlayer": "test", "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    assert_eq!(play_resp.status_code().as_u16(), 200);
    let play_body: Value = play_resp.json();
    let session_id = play_body["id"].as_str().expect("session id").to_string();
    assert_eq!(play_body["playMethod"], json!(0));
    assert_eq!(play_body["mediaType"], json!("podcast"));

    // Sync progress
    let sync_resp = server
        .test_server
        .post(&format!("/api/session/{session_id}/sync"))
        .json(&json!({
            "currentTime": 42.5,
            "timeListened": 42.5,
            "duration": episode.total_time
        }))
        .await;
    assert_eq!(sync_resp.status_code().as_u16(), 200);

    // Close session
    let close_resp = server
        .test_server
        .post(&format!("/api/session/{session_id}/close"))
        .json(&json!({
            "currentTime": 90.0,
            "timeListened": 47.5,
            "duration": episode.total_time
        }))
        .await;
    assert_eq!(close_resp.status_code().as_u16(), 200);

    // Verify media_progress row was written
    let lib_item_id = format!("li_pod_{}", podcast.id);
    let ep_id = format!("ep_{}", episode.id);
    let progress = state
        .audiobookshelf_media_progress_service
        .find(user.id, &lib_item_id, Some(&ep_id))
        .expect("query progress")
        .expect("progress row exists after close");
    assert!((progress.current_time - 90.0).abs() < 0.01);
    assert!(progress.progress > 0.0);
    assert!(progress.duration > 0.0);

    // /api/me should now include the progress row
    let me_resp = server.test_server.get("/api/me").await;
    let me_body: Value = me_resp.json();
    let mp = me_body["mediaProgress"].as_array().unwrap();
    assert_eq!(mp.len(), 1);
    assert_eq!(mp[0]["libraryItemId"], json!(lib_item_id));
    assert_eq!(mp[0]["episodeId"], json!(ep_id));
}

#[tokio::test]
#[serial]
async fn session_cross_user_access_is_forbidden() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user_a = create_user_for_audiobookshelf(&state);
    let user_b = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("XUser podcast", &state, user_a.id);
    let episode = insert_test_episode(podcast.id, "XUser episode");

    // User A opens a session
    login_audiobookshelf(&mut server, &user_a).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    // User B tries to sync that session — must be rejected
    login_audiobookshelf(&mut server, &user_b).await;
    let sync_resp = server
        .test_server
        .post(&format!("/api/session/{session_id}/sync"))
        .json(&json!({ "currentTime": 1.0, "timeListened": 0.5, "duration": episode.total_time }))
        .await;
    assert!(
        sync_resp.status_code().is_client_error(),
        "expected 4xx for cross-user sync, got {}",
        sync_resp.status_code()
    );
}

// ── /public/session/:sid/track/:idx (Range streaming) ──────────────────────

#[tokio::test]
#[serial]
async fn stream_track_serves_full_file_when_no_range() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let bytes = generate_pseudo_audio_bytes(4096);
    let temp_path = write_temp_audio_file(&bytes, "stream-full.mp3");

    let podcast = insert_test_podcast("Stream podcast", &state, user.id);
    let episode = insert_test_episode_with_path(podcast.id, "Stream episode", &temp_path);

    let token = login_audiobookshelf(&mut server, &user).await;

    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    // Clear bearer header — public endpoint must accept ?token=
    server.test_server.clear_headers();
    let response = server
        .test_server
        .get(&format!(
            "/public/session/{session_id}/track/0?token={token}"
        ))
        .await;
    assert_eq!(response.status_code().as_u16(), 200);
    assert_eq!(response.as_bytes().as_ref(), bytes.as_slice());

    let _ = std::fs::remove_file(&temp_path);
}

#[tokio::test]
#[serial]
async fn stream_track_honors_byte_range() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let bytes = generate_pseudo_audio_bytes(8192);
    let temp_path = write_temp_audio_file(&bytes, "stream-range.mp3");

    let podcast = insert_test_podcast("Range podcast", &state, user.id);
    let episode = insert_test_episode_with_path(podcast.id, "Range episode", &temp_path);

    let token = login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    server.test_server.clear_headers();
    let response = server
        .test_server
        .get(&format!(
            "/public/session/{session_id}/track/0?token={token}"
        ))
        .add_header("Range", "bytes=100-199")
        .await;
    assert_eq!(response.status_code().as_u16(), 206);
    let content_range = response
        .headers()
        .get("content-range")
        .expect("content-range header")
        .to_str()
        .unwrap();
    assert_eq!(content_range, format!("bytes 100-199/{}", bytes.len()));
    let body = response.as_bytes();
    assert_eq!(body.len(), 100);
    assert_eq!(body.as_ref(), &bytes[100..200]);

    let _ = std::fs::remove_file(&temp_path);
}

#[tokio::test]
#[serial]
async fn stream_track_without_token_is_unauthorized() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let bytes = generate_pseudo_audio_bytes(1024);
    let temp_path = write_temp_audio_file(&bytes, "stream-noauth.mp3");
    let podcast = insert_test_podcast("Noauth podcast", &state, user.id);
    let episode = insert_test_episode_with_path(podcast.id, "Noauth episode", &temp_path);

    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    server.test_server.clear_headers();
    let response = server
        .test_server
        .get(&format!("/public/session/{session_id}/track/0"))
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx without token, got {}",
        response.status_code()
    );

    let _ = std::fs::remove_file(&temp_path);
}

// ── helpers ────────────────────────────────────────────────────────────────

fn insert_test_podcast(
    name: &str,
    _state: &AppState,
    user_id: i32,
) -> podfetch_persistence::podcast::PodcastEntity {
    use podfetch_domain::podcast::{NewPodcast, PodcastRepository};
    use podfetch_persistence::podcast::DieselPodcastRepository;

    let repo = DieselPodcastRepository::new(podfetch_persistence::db::database());
    let new = NewPodcast {
        name: name.to_string(),
        directory_id: format!("dir-{}", uuid::Uuid::new_v4().simple()),
        rssfeed: format!("https://example.com/rss-{}", uuid::Uuid::new_v4().simple()),
        image_url: "https://example.com/cover.jpg".to_string(),
        directory_name: format!("/tmp/podcasts/{name}"),
        added_by: Some(user_id),
    };
    let created = repo.create(new).expect("create podcast");
    podfetch_persistence::podcast::PodcastEntity {
        id: created.id,
        name: created.name,
        directory_id: created.directory_id,
        rssfeed: created.rssfeed,
        image_url: created.image_url,
        summary: created.summary,
        language: created.language,
        explicit: created.explicit,
        keywords: created.keywords,
        last_build_date: created.last_build_date,
        author: created.author,
        active: created.active,
        original_image_url: created.original_image_url,
        directory_name: created.directory_name,
        download_location: created.download_location,
        guid: created.guid,
        added_by: created.added_by,
    }
}

fn insert_test_episode(
    podcast_id: i32,
    title: &str,
) -> podfetch_domain::podcast_episode::PodcastEpisode {
    insert_test_episode_with_path(podcast_id, title, "/tmp/nonexistent.mp3")
}

fn insert_test_episode_with_path(
    podcast_id: i32,
    title: &str,
    file_path: &str,
) -> podfetch_domain::podcast_episode::PodcastEpisode {
    use chrono::Utc;
    use podfetch_domain::podcast_episode::{NewPodcastEpisode, PodcastEpisodeRepository};
    use podfetch_persistence::podcast_episode::DieselPodcastEpisodeRepository;

    let repo = DieselPodcastEpisodeRepository::new(podfetch_persistence::db::database());
    let new = NewPodcastEpisode {
        podcast_id,
        episode_id: format!("epid-{}", uuid::Uuid::new_v4().simple()),
        name: title.to_string(),
        url: format!("https://example.com/{title}.mp3"),
        date_of_recording: Utc::now().to_rfc3339(),
        image_url: "https://example.com/ep-cover.jpg".to_string(),
        total_time: 300,
        description: format!("description of {title}"),
        guid: format!("guid-{}", uuid::Uuid::new_v4().simple()),
    };
    let mut domain_ep: podfetch_domain::podcast_episode::PodcastEpisode =
        repo.create(new).expect("create episode").into();
    domain_ep.file_episode_path = Some(file_path.to_string());
    domain_ep.download_location = Some(file_path.to_string());
    repo.update(&domain_ep).expect("update episode local path");
    domain_ep
}

fn generate_pseudo_audio_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

// ── Phase C: HLS + playMethod + listening-sessions ──────────────────────────

#[tokio::test]
#[serial]
async fn play_chooses_hls_when_client_lacks_source_codec() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS podcast", &state, user.id);
    let episode = insert_test_episode_with_path(
        podcast.id,
        "FLAC episode",
        "/tmp/episode.flac",
    );

    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({
            "mediaPlayer": "test",
            "supportedMimeTypes": ["audio/mpeg", "audio/mp4"]
        }))
        .await;
    assert_eq!(play_resp.status_code().as_u16(), 200);
    let body: Value = play_resp.json();
    assert_eq!(body["playMethod"], json!(1), "expected playMethod=1 (HLS), got body: {body}");
    let tracks = body["audioTracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    let content_url = tracks[0]["contentUrl"].as_str().unwrap();
    assert!(content_url.ends_with("/master.m3u8"), "got contentUrl: {content_url}");
    assert_eq!(tracks[0]["mimeType"], json!("application/vnd.apple.mpegurl"));
}

#[tokio::test]
#[serial]
async fn play_uses_direct_when_client_supports_source_codec() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Direct podcast", &state, user.id);
    let episode = insert_test_episode_with_path(
        podcast.id,
        "MP3 episode",
        "/tmp/ep.mp3",
    );

    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({
            "supportedMimeTypes": ["audio/mpeg", "audio/mp4"]
        }))
        .await;
    let body: Value = play_resp.json();
    assert_eq!(body["playMethod"], json!(0));
    let content_url = body["audioTracks"][0]["contentUrl"].as_str().unwrap();
    assert!(content_url.starts_with("/public/session/"), "got: {content_url}");
}

#[tokio::test]
#[serial]
async fn hls_master_playlist_returns_m3u8_for_owned_session() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS pod", &state, user.id);
    let episode = insert_test_episode_with_path(podcast.id, "HLS ep", "/tmp/x.flac");
    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    let master_resp = server
        .test_server
        .get(&format!("/hls/{session_id}/master.m3u8"))
        .await;
    assert_eq!(master_resp.status_code().as_u16(), 200);
    let ct = master_resp
        .headers()
        .get("content-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    assert!(ct.contains("mpegurl"), "got content-type: {ct}");
    let body = master_resp.text();
    assert!(body.starts_with("#EXTM3U"));
    assert!(body.contains(&format!("/hls/{session_id}/index.m3u8")));
}

#[tokio::test]
#[serial]
async fn hls_media_playlist_lists_segments_for_duration() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS-Index pod", &state, user.id);
    let episode = insert_test_episode_with_path(podcast.id, "Ep", "/tmp/x.flac");
    // total_time = 300 → expect 50 segments at 6 sec each
    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    let resp = server
        .test_server
        .get(&format!("/hls/{session_id}/index.m3u8"))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body = resp.text();
    assert!(body.contains("#EXT-X-TARGETDURATION:6"));
    assert!(body.contains(&format!("/hls/{session_id}/seg-0.ts")));
    assert!(body.contains(&format!("/hls/{session_id}/seg-49.ts")));
    assert!(body.contains("#EXT-X-ENDLIST"));
    let _ = episode;
}

#[tokio::test]
#[serial]
async fn hls_playlist_cross_user_access_is_forbidden() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user_a = create_user_for_audiobookshelf(&state);
    let user_b = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS XUser pod", &state, user_a.id);
    let episode = insert_test_episode_with_path(podcast.id, "Ep", "/tmp/x.flac");
    login_audiobookshelf(&mut server, &user_a).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    login_audiobookshelf(&mut server, &user_b).await;
    let resp = server
        .test_server
        .get(&format!("/hls/{session_id}/master.m3u8"))
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx, got {}",
        resp.status_code()
    );
}

#[tokio::test]
#[serial]
async fn close_session_persists_listening_session_history() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("History pod", &state, user.id);
    let episode = insert_test_episode(podcast.id, "History ep");
    login_audiobookshelf(&mut server, &user).await;

    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();
    server
        .test_server
        .post(&format!("/api/session/{session_id}/close"))
        .json(&json!({
            "currentTime": 60.0,
            "timeListened": 60.0,
            "duration": episode.total_time
        }))
        .await;

    // Now query /api/me/listening-sessions
    let resp = server
        .test_server
        .get("/api/me/listening-sessions?limit=10")
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    let sessions = body["sessions"].as_array().expect("sessions array");
    assert_eq!(sessions.len(), 1, "expected 1 listening session, got {sessions:#?}");
    let entry = &sessions[0];
    assert_eq!(entry["mediaType"], json!("podcast"));
    assert_eq!(entry["libraryItemId"], json!(format!("li_pod_{}", podcast.id)));
    assert_eq!(entry["episodeId"], json!(format!("ep_{}", episode.id)));
    assert!((entry["timeListening"].as_f64().unwrap() - 60.0).abs() < 0.1);
}

#[tokio::test]
#[serial]
async fn listening_sessions_isolated_per_user() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user_a = create_user_for_audiobookshelf(&state);
    let user_b = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Isolation pod", &state, user_a.id);
    let episode = insert_test_episode(podcast.id, "Iso ep");

    // user_a opens and closes a session
    login_audiobookshelf(&mut server, &user_a).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();
    server
        .test_server
        .post(&format!("/api/session/{session_id}/close"))
        .json(&json!({"currentTime": 10.0, "timeListened": 10.0, "duration": episode.total_time}))
        .await;

    // user_b has no listening history
    login_audiobookshelf(&mut server, &user_b).await;
    let resp = server.test_server.get("/api/me/listening-sessions").await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    let sessions = body["sessions"].as_array().unwrap();
    assert_eq!(sessions.len(), 0, "user_b should see no sessions, got {sessions:#?}");
}

// ── Book-side integration tests (Phase B) ───────────────────────────────────

#[tokio::test]
#[serial]
async fn list_audiobooks_library_items_returns_books() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| matches!(l.media_type, podfetch_domain::audiobookshelf::library::MediaType::Book))
        .expect("audiobooks library bootstrapped");
    let book = insert_test_book(&lib.id, "Project Hail Mary", "Andy Weir");
    insert_test_audio_file_row(&book.id, 0, "/tmp/nonexistent.m4b", 120.0);

    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/libraries/{}/items", lib.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    let results = body["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["mediaType"], json!("book"));
    assert_eq!(results[0]["media"]["metadata"]["title"], json!("Project Hail Mary"));
    let authors = results[0]["media"]["metadata"]["authors"].as_array().unwrap();
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0]["name"], json!("Andy Weir"));
}

#[tokio::test]
#[serial]
async fn get_book_item_includes_chapters_and_audio_files() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| matches!(l.media_type, podfetch_domain::audiobookshelf::library::MediaType::Book))
        .unwrap();
    let book = insert_test_book(&lib.id, "Mistborn", "Brandon Sanderson");
    insert_test_audio_file_row(&book.id, 0, "/tmp/mb-1.mp3", 100.0);
    insert_test_audio_file_row(&book.id, 1, "/tmp/mb-2.mp3", 200.0);
    insert_test_chapter_row(&book.id, 0, 0.0, 100.0, "Chapter 1");
    insert_test_chapter_row(&book.id, 1, 100.0, 300.0, "Chapter 2");

    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/items/{}", book.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["mediaType"], json!("book"));
    let chapters = body["media"]["chapters"].as_array().unwrap();
    assert_eq!(chapters.len(), 2);
    assert_eq!(chapters[0]["title"], json!("Chapter 1"));
    let audio_files = body["media"]["audioFiles"].as_array().unwrap();
    assert_eq!(audio_files.len(), 2);
}

#[tokio::test]
#[serial]
async fn play_book_session_returns_tracks_per_audio_file() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| matches!(l.media_type, podfetch_domain::audiobookshelf::library::MediaType::Book))
        .unwrap();
    let book = insert_test_book(&lib.id, "Three Body", "Cixin Liu");
    insert_test_audio_file_row(&book.id, 0, "/tmp/tb-1.mp3", 50.0);
    insert_test_audio_file_row(&book.id, 1, "/tmp/tb-2.mp3", 75.0);

    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post(&format!("/api/items/{}/play", book.id))
        .json(&json!({}))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["mediaType"], json!("book"));
    let tracks = body["audioTracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0]["index"], json!(0));
    assert_eq!(tracks[1]["index"], json!(1));
    assert!((tracks[1]["startOffset"].as_f64().unwrap() - 50.0).abs() < 0.01);
}

#[tokio::test]
#[serial]
async fn stream_book_track_serves_correct_audio_file() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| matches!(l.media_type, podfetch_domain::audiobookshelf::library::MediaType::Book))
        .unwrap();

    let bytes_0 = generate_pseudo_audio_bytes(2048);
    let bytes_1 = generate_pseudo_audio_bytes(3072);
    let path_0 = write_temp_audio_file(&bytes_0, "book-0.mp3");
    let path_1 = write_temp_audio_file(&bytes_1, "book-1.mp3");
    let book = insert_test_book(&lib.id, "Stream Book", "Stream Author");
    insert_test_audio_file_row(&book.id, 0, &path_0, 30.0);
    insert_test_audio_file_row(&book.id, 1, &path_1, 45.0);

    let token = login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!("/api/items/{}/play", book.id))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"].as_str().unwrap().to_string();

    server.test_server.clear_headers();
    // Track 1 (second file) — verify routing to correct file
    let resp = server
        .test_server
        .get(&format!(
            "/public/session/{session_id}/track/1?token={token}"
        ))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    assert_eq!(resp.as_bytes().as_ref(), bytes_1.as_slice());

    let _ = std::fs::remove_file(&path_0);
    let _ = std::fs::remove_file(&path_1);
}

fn insert_test_book(
    library_id: &str,
    title: &str,
    author: &str,
) -> podfetch_domain::audiobookshelf::book::Book {
    use chrono::Utc;
    use podfetch_domain::audiobookshelf::book::{AuthorRepository, Book, BookRepository};
    use podfetch_persistence::adapters::{AuthorRepositoryImpl, BookRepositoryImpl};
    use podfetch_persistence::audiobookshelf::book::new_book_id;
    use podfetch_persistence::db::database;
    use std::sync::Arc;

    let repo: Arc<dyn BookRepository<Error = common_infrastructure::error::CustomError>> =
        Arc::new(BookRepositoryImpl::new(database()));
    let author_repo: Arc<
        dyn AuthorRepository<Error = common_infrastructure::error::CustomError>,
    > = Arc::new(AuthorRepositoryImpl::new(database()));

    let now = Utc::now().naive_utc();
    let book = Book {
        id: new_book_id(),
        library_id: library_id.to_string(),
        title: title.to_string(),
        subtitle: None,
        description: None,
        publisher: None,
        published_year: None,
        published_date: None,
        isbn: None,
        asin: None,
        language: None,
        explicit: false,
        cover_path: None,
        duration_seconds: 0.0,
        ino: None,
        folder_path: format!("/tmp/abs-tests/{}", uuid::Uuid::new_v4().simple()),
        last_scan: Some(now),
        added_at: now,
        updated_at: now,
    };
    let stored = repo.upsert(book).expect("upsert book");
    let author_entity = author_repo.upsert_by_name(author).expect("upsert author");
    author_repo
        .link(&stored.id, &author_entity.id)
        .expect("link author");
    stored
}

fn insert_test_audio_file_row(book_id: &str, idx: i32, path: &str, duration: f64) {
    use podfetch_domain::audiobookshelf::book::{BookAudioFile, BookAudioFileRepository};
    use podfetch_persistence::adapters::BookAudioFileRepositoryImpl;
    use podfetch_persistence::db::database;
    use std::sync::Arc;

    let repo: Arc<
        dyn BookAudioFileRepository<Error = common_infrastructure::error::CustomError>,
    > = Arc::new(BookAudioFileRepositoryImpl::new(database()));
    let mut existing = repo.list_for_book(book_id).unwrap_or_default();
    existing.push(BookAudioFile {
        id: format!("af_{}", uuid::Uuid::new_v4().simple()),
        book_id: book_id.to_string(),
        idx,
        ino: None,
        path: path.to_string(),
        relative_path: path.to_string(),
        ext: "mp3".to_string(),
        mime_type: "audio/mpeg".to_string(),
        duration,
        bitrate: 128000,
        codec: "mp3".to_string(),
        channels: 2,
        sample_rate: 44100,
        track_num: Some(idx + 1),
        disc_num: None,
        embedded_cover_path: None,
    });
    repo.replace_for_book(book_id, existing).unwrap();
}

fn insert_test_chapter_row(book_id: &str, idx: i32, start: f64, end: f64, title: &str) {
    use podfetch_domain::audiobookshelf::book::{BookChapter, BookChapterRepository};
    use podfetch_persistence::adapters::BookChapterRepositoryImpl;
    use podfetch_persistence::db::database;
    use std::sync::Arc;

    let repo: Arc<
        dyn BookChapterRepository<Error = common_infrastructure::error::CustomError>,
    > = Arc::new(BookChapterRepositoryImpl::new(database()));
    let mut existing = repo.list_for_book(book_id).unwrap_or_default();
    existing.push(BookChapter {
        id: format!("chp_{}", uuid::Uuid::new_v4().simple()),
        book_id: book_id.to_string(),
        idx,
        start_time: start,
        end_time: end,
        title: title.to_string(),
    });
    repo.replace_for_book(book_id, existing).unwrap();
}

fn write_temp_audio_file(bytes: &[u8], filename: &str) -> String {
    let dir = std::env::temp_dir().join("podfetch-abs-tests");
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let unique = format!(
        "{}-{}-{filename}",
        std::process::id(),
        uuid::Uuid::new_v4().simple()
    );
    let path = dir.join(unique);
    std::fs::write(&path, bytes).expect("write temp file");
    path.to_string_lossy().to_string()
}
