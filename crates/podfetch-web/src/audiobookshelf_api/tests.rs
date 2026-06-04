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
    let media_types: Vec<&str> = libraries.iter().map(|l| l.media_type.as_str()).collect();
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
    assert!(
        libraries.len() >= 2,
        "expected >=2 libraries, got {libraries:#?}"
    );
    assert!(
        libraries.iter().any(|l| l["mediaType"] == json!("podcast")),
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
    assert_eq!(
        results.len(),
        1,
        "expected one library item for inserted podcast"
    );
    let expected_id = format!("li_pod_{}", podcast.id);
    assert_eq!(results[0]["id"], json!(expected_id));
    assert_eq!(results[0]["mediaType"], json!("podcast"));
    assert_eq!(
        results[0]["media"]["metadata"]["title"],
        json!(podcast.name)
    );
}

#[tokio::test]
#[serial]
async fn get_item_by_id_returns_podcast_with_episodes() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Detail test podcast", &state, user.id);
    insert_test_episode(&podcast.id, "Test Episode 1");

    login_audiobookshelf(&mut server, &user).await;
    let response = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert_eq!(body["mediaType"], json!("podcast"));
    let episodes = body["media"]["episodes"]
        .as_array()
        .expect("episodes array");
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
    let episode = insert_test_episode(&podcast.id, "Session Episode");

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
    // Body must be a JSON object: audiobookshelf-app's ApiHandler.makeRequest
    // parses the body as JSON and treats an empty body as "Invalid response
    // body", which turns every sync into a failed sync on the client.
    let sync_body: Value = sync_resp.json();
    assert!(
        sync_body.is_object(),
        "sync body must be a JSON object, got {sync_body}"
    );

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
    let close_body: Value = close_resp.json();
    assert!(
        close_body.is_object(),
        "close body must be a JSON object, got {close_body}"
    );

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
    let episode = insert_test_episode(&podcast.id, "XUser episode");

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
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode_with_path(&podcast.id, "Stream episode", &temp_path);

    let token = login_audiobookshelf(&mut server, &user).await;

    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode_with_path(&podcast.id, "Range episode", &temp_path);

    let token = login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode_with_path(&podcast.id, "Noauth episode", &temp_path);

    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    user_id: uuid::Uuid,
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
        id: created.id.to_string(),
        legacy_id: created.legacy_id,
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
        added_by: created.added_by.map(|u| u.to_string()),
    }
}

fn insert_test_episode(
    podcast_id: &str,
    title: &str,
) -> podfetch_domain::podcast_episode::PodcastEpisode {
    insert_test_episode_with_path(podcast_id, title, "/tmp/nonexistent.mp3")
}

fn insert_test_episode_with_path(
    podcast_id: &str,
    title: &str,
    file_path: &str,
) -> podfetch_domain::podcast_episode::PodcastEpisode {
    use chrono::Utc;
    use podfetch_domain::podcast_episode::{NewPodcastEpisode, PodcastEpisodeRepository};
    use podfetch_persistence::podcast_episode::DieselPodcastEpisodeRepository;

    let repo = DieselPodcastEpisodeRepository::new(podfetch_persistence::db::database());
    let new = NewPodcastEpisode {
        podcast_id: uuid::Uuid::parse_str(podcast_id).expect("valid podcast uuid"),
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
        repo.create(new).expect("create episode");
    domain_ep.file_episode_path = Some(file_path.to_string());
    domain_ep.download_location = Some(file_path.to_string());
    repo.update(&domain_ep).expect("update episode local path");
    domain_ep
}

fn generate_pseudo_audio_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

/// Stamps a pre-migration integer `legacy_id` directly onto the podcast row,
/// simulating a podcast that existed before the UUID migration (the migration
/// backfills `legacy_id` from the old integer pk). `DBType` derives
/// `diesel::MultiConnection`, so this runs against sqlite + postgres alike.
fn set_podcast_legacy_id(podcast_uuid: &str, legacy: i64) {
    use diesel::prelude::*;
    use podfetch_persistence::db::database;
    use podfetch_persistence::podcast::podcasts;
    let db = database();
    let mut conn = db.connection().expect("db connection");
    diesel::update(podcasts::table.filter(podcasts::id.eq(podcast_uuid.to_string())))
        .set(podcasts::legacy_id.eq(Some(legacy)))
        .execute(&mut conn)
        .expect("set podcast legacy_id");
}

/// Stamps a pre-migration integer `legacy_id` directly onto the episode row.
fn set_episode_legacy_id(episode_uuid: &str, legacy: i64) {
    use diesel::prelude::*;
    use podfetch_persistence::db::database;
    use podfetch_persistence::podcast_episode::podcast_episodes;
    let db = database();
    let mut conn = db.connection().expect("db connection");
    diesel::update(podcast_episodes::table.filter(podcast_episodes::id.eq(episode_uuid.to_string())))
        .set(podcast_episodes::legacy_id.eq(Some(legacy)))
        .execute(&mut conn)
        .expect("set episode legacy_id");
}

// ── Legacy ABS id backwards-compat (UUID migration) ────────────────────────

/// An ABS client that cached the pre-migration integer ids (`li_pod_{int}`,
/// `ino_ep_{int}`) must resolve to the *same* podcast/episode/file as the new
/// UUID-based ids. Proves incoming legacy id parsing while outgoing ids stay
/// UUID-shaped.
#[tokio::test]
#[serial]
async fn legacy_integer_ids_resolve_to_same_podcast_and_episode() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let bytes = generate_pseudo_audio_bytes(2048);
    let path = write_temp_audio_file(&bytes, "legacy-file.mp3");
    let podcast = insert_test_podcast("Legacy id podcast", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "Legacy ep", &path);

    // Simulate the migration backfilling old integer pks.
    let podcast_legacy: i64 = 4242;
    let episode_legacy: i64 = 7777;
    set_podcast_legacy_id(&podcast.id, podcast_legacy);
    set_episode_legacy_id(&episode.id.to_string(), episode_legacy);

    login_audiobookshelf(&mut server, &user).await;

    // (1) GET /api/items via the UUID form establishes the canonical payload.
    let uuid_resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    assert_eq!(uuid_resp.status_code().as_u16(), 200);
    let uuid_body: Value = uuid_resp.json();

    // (2) GET /api/items via the LEGACY integer form must resolve to the same
    //     podcast and emit the same UUID-shaped outgoing ids.
    let legacy_resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{podcast_legacy}"))
        .await;
    assert_eq!(
        legacy_resp.status_code().as_u16(),
        200,
        "legacy li_pod_{{int}} must resolve, got {}",
        legacy_resp.status_code()
    );
    let legacy_body: Value = legacy_resp.json();

    // Outgoing ids stay UUID-based and identical between both fetches.
    assert_eq!(legacy_body["id"], uuid_body["id"]);
    assert_eq!(legacy_body["id"], json!(format!("li_pod_{}", podcast.id)));
    assert_eq!(legacy_body["media"]["id"], json!(format!("pod_{}", podcast.id)));
    assert_eq!(
        legacy_body["media"]["metadata"]["title"],
        json!(podcast.name)
    );
    let legacy_episodes = legacy_body["media"]["episodes"].as_array().unwrap();
    assert_eq!(legacy_episodes.len(), 1);
    assert_eq!(
        legacy_episodes[0]["id"],
        json!(format!("ep_{}", episode.id))
    );

    // (3) The audio-file endpoint must accept BOTH a legacy library-item id and
    //     a legacy `ino_ep_{int}` and stream the same bytes as the UUID form.
    let uuid_file = server
        .test_server
        .get(&format!(
            "/api/items/li_pod_{}/file/ino_ep_{}",
            podcast.id, episode.id
        ))
        .await;
    assert_eq!(uuid_file.status_code().as_u16(), 200);
    assert_eq!(uuid_file.as_bytes().as_ref(), bytes.as_slice());

    let legacy_file = server
        .test_server
        .get(&format!(
            "/api/items/li_pod_{podcast_legacy}/file/ino_ep_{episode_legacy}"
        ))
        .await;
    assert_eq!(
        legacy_file.status_code().as_u16(),
        200,
        "legacy ino_ep_{{int}} must stream, got {}",
        legacy_file.status_code()
    );
    assert_eq!(legacy_file.as_bytes().as_ref(), bytes.as_slice());

    let _ = std::fs::remove_file(&path);
}

/// Starting a playback session with legacy `li_pod_{int}` / `ep_{int}` must
/// behave like the UUID form, and the resulting session must carry the
/// canonical UUID-shaped ids so the streaming endpoints (which re-parse the
/// stored id with the UUID-only parser) keep working.
#[tokio::test]
#[serial]
async fn legacy_integer_ids_start_session_with_canonical_uuid() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Legacy session pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Legacy session ep");
    let podcast_legacy: i64 = 909;
    let episode_legacy: i64 = 808;
    set_podcast_legacy_id(&podcast.id, podcast_legacy);
    set_episode_legacy_id(&episode.id.to_string(), episode_legacy);

    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{podcast_legacy}/play/ep_{episode_legacy}"
        ))
        .json(&json!({}))
        .await;
    assert_eq!(
        resp.status_code().as_u16(),
        200,
        "legacy play must succeed, got {}",
        resp.status_code()
    );
    let body: Value = resp.json();
    assert_eq!(body["mediaType"], json!("podcast"));
    // Stored/emitted ids are canonical UUID form, never the legacy integer.
    assert_eq!(
        body["libraryItemId"],
        json!(format!("li_pod_{}", podcast.id))
    );
    assert_eq!(body["episodeId"], json!(format!("ep_{}", episode.id)));
}

// ── Cover-redirect: real podcasts have remote artwork in the RSS feed ──────

#[tokio::test]
#[serial]
async fn podcast_cover_redirects_to_remote_url_when_no_local_file() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    // Use the repository directly so we can set original_image_url to a
    // proper https URL (the test podcast helper uses fake bookkeeping).
    use podfetch_domain::podcast::{NewPodcast, PodcastRepository};
    use podfetch_persistence::db::database;
    use podfetch_persistence::podcast::DieselPodcastRepository;
    let repo = DieselPodcastRepository::new(database());
    let created = repo
        .create(NewPodcast {
            name: "Remote Cover Pod".to_string(),
            directory_id: "remote-cover".to_string(),
            rssfeed: "https://example.com/feed.rss".to_string(),
            image_url: "podcasts/remote-cover/missing-local.jpg".to_string(),
            directory_name: "/tmp/podcasts/does-not-exist".to_string(),
            added_by: Some(user.id),
        })
        .expect("create podcast");
    // Backfill original_image_url to a real https URL via update_original_image_url.
    repo.update_original_image_url(created.id, "https://example.com/cover.png")
        .expect("set original_image_url");

    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{}/cover", created.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 302, "expected redirect");
    let location = resp
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    assert_eq!(location, "https://example.com/cover.png");
}

// ── Phase E: audiobookshelf compat fixes ────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_response_shape_matches_upstream_payload() {
    // Pins `Auth.getUserLoginResponsePayload`: user / userDefaultLibraryId /
    // serverSettings / ereaderDevices / Source. The capitalised `Source` is
    // significant - the mobile apps read it with that exact case.
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    server.test_server.clear_headers();
    let resp = server
        .test_server
        .post("/login")
        .json(&json!({ "username": user.username, "password": "password" }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    for top_field in [
        "user",
        "userDefaultLibraryId",
        "serverSettings",
        "ereaderDevices",
        "Source",
    ] {
        assert!(
            body.get(top_field).is_some(),
            "login response missing top-level field {top_field}"
        );
    }
    assert!(body["ereaderDevices"].is_array());
    assert!(body["Source"].is_string());
    let ss = &body["serverSettings"];
    for ss_field in [
        "id",
        "version",
        "buildNumber",
        "language",
        "dateFormat",
        "timeFormat",
        "authActiveAuthMethods",
        "chromecastEnabled",
        "bookshelfView",
        "homeBookshelfView",
        "sortingPrefixes",
    ] {
        assert!(
            ss.get(ss_field).is_some(),
            "serverSettings missing field {ss_field}"
        );
    }
}

#[tokio::test]
#[serial]
async fn recent_episodes_endpoint_returns_episodes_envelope() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()
        .unwrap()
        .unwrap();
    let podcast_a = insert_test_podcast("A pod", &state, user.id);
    let _ep_a1 = insert_test_episode(&podcast_a.id, "A1");
    let _ep_a2 = insert_test_episode(&podcast_a.id, "A2");
    let podcast_b = insert_test_podcast("B pod", &state, user.id);
    let _ep_b1 = insert_test_episode(&podcast_b.id, "B1");
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .get(&format!(
            "/api/libraries/{}/recent-episodes?limit=10",
            lib.id
        ))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert!(body["episodes"].is_array(), "missing episodes array");
    assert!(body["limit"].is_i64());
    assert!(body["page"].is_i64());
    let episodes = body["episodes"].as_array().unwrap();
    assert_eq!(episodes.len(), 3, "expected 3 episodes, got {episodes:#?}");
    for ep in episodes {
        assert!(
            ep["podcast"].is_object(),
            "episode missing podcast: {ep:#?}"
        );
        assert!(ep["libraryId"].is_string());
        assert!(ep["libraryItemId"].is_string());
        assert!(ep["title"].is_string());
        let pod = &ep["podcast"];
        assert!(pod["id"].as_str().unwrap().starts_with("pod_"));
        assert!(
            pod["libraryItemId"]
                .as_str()
                .unwrap()
                .starts_with("li_pod_")
        );
        assert!(pod["metadata"].is_object());
    }
}

#[tokio::test]
#[serial]
async fn recent_episodes_rejects_non_podcast_library() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/libraries/{}/recent-episodes", lib.id))
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx for book library, got {}",
        resp.status_code()
    );
}

#[tokio::test]
#[serial]
async fn personalized_endpoint_returns_empty_array_stub() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()
        .unwrap()
        .unwrap();
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/libraries/{}/personalized", lib.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn items_in_progress_returns_library_items_envelope() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Progress pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Ep");
    login_audiobookshelf(&mut server, &user).await;

    let empty = server.test_server.get("/api/me/items-in-progress").await;
    assert_eq!(empty.status_code().as_u16(), 200);
    let body: Value = empty.json();
    assert_eq!(body["libraryItems"].as_array().unwrap().len(), 0);

    let _ = server
        .test_server
        .patch(&format!(
            "/api/me/progress/li_pod_{}/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({"currentTime": 30.0, "duration": 300.0}))
        .await;
    let with = server.test_server.get("/api/me/items-in-progress").await;
    let body: Value = with.json();
    let items = body["libraryItems"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert!((items[0]["currentTime"].as_f64().unwrap() - 30.0).abs() < 0.01);
}

#[tokio::test]
#[serial]
async fn playback_session_shape_matches_upstream() {
    // Pins PlaybackSession.toJSONForClient() so the mobile apps can read
    // the session payload off /api/items/.../play without crashing.
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Shape session pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Shape ep");
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    for field in [
        "id",
        "userId",
        "libraryId",
        "libraryItemId",
        "bookId",
        "episodeId",
        "mediaType",
        "mediaMetadata",
        "chapters",
        "displayTitle",
        "displayAuthor",
        "coverPath",
        "duration",
        "playMethod",
        "mediaPlayer",
        "deviceInfo",
        "serverVersion",
        "date",
        "dayOfWeek",
        "timeListening",
        "startTime",
        "currentTime",
        "startedAt",
        "updatedAt",
        "audioTracks",
        "libraryItem",
    ] {
        assert!(
            body.get(field).is_some(),
            "playback-session payload missing field {field}"
        );
    }
    // Podcast sessions have a null bookId, episodeId set
    assert_eq!(body["bookId"], Value::Null);
    assert!(body["episodeId"].is_string());
    // date matches YYYY-MM-DD
    let date = body["date"].as_str().unwrap();
    assert_eq!(date.len(), 10, "expected YYYY-MM-DD, got {date}");
}

#[tokio::test]
#[serial]
async fn play_response_passes_android_kotlin_required_fields() {
    // Android app's AbsAudioPlayer deserialises via Jackson into Kotlin
    // data classes where these fields are NON-NULL. Sending null or
    // omitting them makes Jackson throw MissingKotlinParameterException;
    // the OkHttp callback never fires → playback spinner forever. Found
    // by reading C:\Users\samue\RustroverProjects\audiobookshelf-app\
    // android\app\src\main\java\com\audiobookshelf\app\data\.
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Android compat pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Compat ep");
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let body: Value = resp.json();

    let track = &body["audioTracks"][0];
    assert_eq!(
        track["isLocal"],
        json!(false),
        "audioTracks[0].isLocal must be a boolean (Kotlin AudioTrack.isLocal is non-null)"
    );

    let device_info = &body["deviceInfo"];
    assert!(
        device_info.is_object(),
        "deviceInfo must be an object, not null (Kotlin DeviceInfo is non-null). Got: {device_info:#?}"
    );
    for field in [
        "deviceId",
        "manufacturer",
        "model",
        "sdkVersion",
        "clientVersion",
    ] {
        assert!(
            device_info.get(field).is_some() && !device_info[field].is_null(),
            "deviceInfo.{field} must be non-null"
        );
    }

    assert!(
        body["timeListening"].is_i64(),
        "timeListening must be an integer JSON literal (Kotlin Long type)"
    );

    // Kotlin PodcastMetadata.genres is non-null MutableList<String>.
    let genres = &body["mediaMetadata"]["genres"];
    assert!(
        genres.is_array(),
        "mediaMetadata.genres must be a (possibly empty) array. Got: {genres:#?}"
    );

    // Kotlin LibraryItem.folderId is non-null String. The play response
    // embeds the LibraryItem.
    let library_item = &body["libraryItem"];
    assert!(library_item.is_object(), "libraryItem must be an object");
    let folder_id = &library_item["folderId"];
    assert!(
        folder_id.is_string() && !folder_id.as_str().unwrap().is_empty(),
        "libraryItem.folderId must be a non-empty string. Got: {folder_id:#?}"
    );

    // Kotlin FileMetadata has non-null filename/ext/path/relPath. Jackson
    // walks libraryItem.media.episodes[0].audioFile.metadata and crashes
    // if any of these are null.
    let ep_meta = &library_item["media"]["episodes"][0]["audioFile"]["metadata"];
    for field in ["filename", "ext", "path", "relPath"] {
        let value = &ep_meta[field];
        assert!(
            value.is_string(),
            "libraryItem.media.episodes[0].audioFile.metadata.{field} must be a non-null string (Kotlin FileMetadata). Got: {value:#?}"
        );
    }
}

#[tokio::test]
#[serial]
async fn book_playback_session_carries_book_id() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    let book = insert_test_book(&lib.id, "Session book", "Author");
    insert_test_audio_file_row(&book.id, 0, "/tmp/x.mp3", 60.0);
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post(&format!("/api/items/{}/play", book.id))
        .json(&json!({}))
        .await;
    let body: Value = resp.json();
    // For books, bookId equals the library item id and episodeId is null.
    assert_eq!(body["bookId"], json!(book.id));
    assert_eq!(body["episodeId"], Value::Null);
    assert_eq!(body["mediaType"], json!("book"));
}

#[tokio::test]
#[serial]
async fn status_carries_app_audiobookshelf_marker() {
    // Mobile apps probe /status and verify `app == "audiobookshelf"` before
    // committing to the server. Omitting it makes them refuse the connection.
    let server = handle_test_startup().await;
    let resp = server.test_server.get("/status").await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["app"], json!("audiobookshelf"));
    assert!(body["serverVersion"].is_string());
    assert_eq!(body["isInit"], json!(true));
    assert!(body["authMethods"].is_array());
    assert!(
        body["authFormData"].is_null() || body["authFormData"].is_object(),
        "authFormData must be null or object, got {:?}",
        body["authFormData"]
    );
}

#[tokio::test]
#[serial]
async fn podcast_library_item_shape_matches_upstream() {
    // Pins the full LibraryItem.toOldJSONExpanded() shape for podcasts so we
    // know we're byte-shape-compatible with upstream.
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Shape Item Pod", &state, user.id);
    let _ep = insert_test_episode(&podcast.id, "Episode");
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    // Top-level LibraryItem fields per LibraryItem.toOldJSONExpanded()
    for field in [
        "id",
        "ino",
        "oldLibraryItemId",
        "libraryId",
        "folderId",
        "path",
        "relPath",
        "isFile",
        "mtimeMs",
        "ctimeMs",
        "birthtimeMs",
        "addedAt",
        "updatedAt",
        "lastScan",
        "scanVersion",
        "isMissing",
        "isInvalid",
        "mediaType",
        "media",
        "libraryFiles",
        "size",
    ] {
        assert!(
            body.get(field).is_some(),
            "library-item missing field {field}"
        );
    }
    assert_eq!(body["mediaType"], json!("podcast"));
    assert_eq!(body["oldLibraryItemId"], Value::Null);
    assert!(body["libraryFiles"].is_array());

    // Media-level fields per Podcast.toOldJSONExpanded()
    let media = &body["media"];
    for field in [
        "id",
        "libraryItemId",
        "metadata",
        "coverPath",
        "tags",
        "episodes",
        "autoDownloadEpisodes",
        "autoDownloadSchedule",
        "lastEpisodeCheck",
        "maxEpisodesToKeep",
        "maxNewEpisodesToDownload",
        "size",
    ] {
        assert!(
            media.get(field).is_some(),
            "podcast media missing field {field}"
        );
    }
    assert!(media["id"].as_str().unwrap().starts_with("pod_"));
    assert_eq!(media["libraryItemId"], body["id"]);

    // Metadata-level fields per Podcast.oldMetadataToJSONExpanded()
    let metadata = &media["metadata"];
    for field in [
        "title",
        "titleIgnorePrefix",
        "author",
        "description",
        "releaseDate",
        "genres",
        "feedUrl",
        "imageUrl",
        "itunesPageUrl",
        "itunesId",
        "itunesArtistId",
        "explicit",
        "language",
        "type",
    ] {
        assert!(
            metadata.get(field).is_some(),
            "podcast metadata missing field {field}"
        );
    }
    assert_eq!(metadata["type"], json!("episodic"));
}

#[tokio::test]
#[serial]
async fn title_ignore_prefix_is_calculated_for_podcast() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast_named("The Daily Sample", &state, user.id);
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    let body: Value = resp.json();
    assert_eq!(
        body["media"]["metadata"]["titleIgnorePrefix"],
        json!("Daily Sample, The")
    );
}

fn insert_test_podcast_named(
    name: &str,
    state: &AppState,
    user_id: uuid::Uuid,
) -> podfetch_persistence::podcast::PodcastEntity {
    insert_test_podcast(name, state, user_id)
}

#[tokio::test]
#[serial]
async fn me_dto_includes_all_upstream_user_fields() {
    // Pins the audiobookshelf User.toOldJSONForBrowser() shape so missing
    // fields stop the mobile apps. If you ever change a field name here you
    // are breaking the wire compat.
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let resp = server.test_server.get("/api/me").await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    for field in [
        "id",
        "username",
        "email",
        "type",
        "token",
        "isOldToken",
        "isActive",
        "isLocked",
        "lastSeen",
        "createdAt",
        "permissions",
        "librariesAccessible",
        "itemTagsSelected",
        "bookmarks",
        "seriesHideFromContinueListening",
        "hasOpenIDLink",
        "mediaProgress",
    ] {
        assert!(
            body.get(field).is_some(),
            "user payload missing field {field}"
        );
    }
    assert!(body["bookmarks"].is_array());
    assert!(body["seriesHideFromContinueListening"].is_array());
    assert!(body["itemTagsSelected"].is_array());
    assert_eq!(body["isOldToken"], json!(false));
    assert_eq!(body["isLocked"], json!(false));
    assert_eq!(body["hasOpenIDLink"], json!(false));
}

#[tokio::test]
#[serial]
async fn podcast_episode_shape_matches_audiobookshelf() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Shape pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Shape ep");
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .get(&format!("/api/items/li_pod_{}", podcast.id))
        .await;
    let body: Value = resp.json();
    let ep = &body["media"]["episodes"][0];
    // All upstream-required keys must exist.
    for field in [
        "libraryItemId",
        "podcastId",
        "id",
        "oldEpisodeId",
        "index",
        "season",
        "episode",
        "episodeType",
        "title",
        "subtitle",
        "description",
        "enclosure",
        "guid",
        "pubDate",
        "chapters",
        "audioFile",
        "audioTrack",
        "publishedAt",
        "addedAt",
        "updatedAt",
        "duration",
        "size",
    ] {
        assert!(
            ep.get(field).is_some(),
            "episode payload missing field {field}: {ep:#?}"
        );
    }
    // podcastId is a uuid string
    assert!(ep["podcastId"].is_string());
    assert_eq!(ep["podcastId"], json!(podcast.id.to_string()));
    // audioTrack.contentUrl matches upstream's /api/items/<id>/file/<ino>
    let url = ep["audioTrack"]["contentUrl"].as_str().unwrap();
    let expected = format!(
        "/api/items/li_pod_{}/file/ino_ep_{}",
        podcast.id, episode.id
    );
    assert_eq!(url, expected, "audioTrack.contentUrl mismatch");
    // enclosure carries type + url + nullable length
    let enc = &ep["enclosure"];
    assert!(enc["url"].is_string());
    assert!(enc["type"].is_string());
    assert!(enc["length"].is_null() || enc["length"].is_string());
}

#[tokio::test]
#[serial]
async fn patch_me_progress_creates_progress_row_and_reflects_in_me() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Patch pod", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Patch ep");
    login_audiobookshelf(&mut server, &user).await;

    let li_id = format!("li_pod_{}", podcast.id);
    let ep_id = format!("ep_{}", episode.id);
    let resp = server
        .test_server
        .patch(&format!("/api/me/progress/{}/{}", li_id, ep_id))
        .json(&json!({
            "currentTime": 123.5,
            "duration": 300.0,
            "isFinished": false
        }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);

    // /api/me should now include the row
    let me_resp = server.test_server.get("/api/me").await;
    let body: Value = me_resp.json();
    let progress = body["mediaProgress"].as_array().unwrap();
    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0]["libraryItemId"], json!(li_id));
    assert_eq!(progress[0]["episodeId"], json!(ep_id));
    assert!((progress[0]["currentTime"].as_f64().unwrap() - 123.5).abs() < 0.01);
}

#[tokio::test]
#[serial]
async fn patch_me_progress_item_only_works_for_books() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .patch("/api/me/progress/li_book_someid")
        .json(&json!({ "currentTime": 50.0, "duration": 200.0 }))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);

    let me_resp = server.test_server.get("/api/me").await;
    let body: Value = me_resp.json();
    let progress = body["mediaProgress"].as_array().unwrap();
    assert_eq!(progress.len(), 1);
    assert_eq!(progress[0]["libraryItemId"], json!("li_book_someid"));
    assert!(progress[0]["episodeId"].is_null());
    assert_eq!(progress[0]["mediaItemType"], json!("book"));
}

#[tokio::test]
#[serial]
async fn patch_me_progress_batch_upserts_multiple() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Batch pod", &state, user.id);
    let ep_a = insert_test_episode(&podcast.id, "ep a");
    let ep_b = insert_test_episode(&podcast.id, "ep b");
    login_audiobookshelf(&mut server, &user).await;

    let body = json!([
        {
            "libraryItemId": format!("li_pod_{}", podcast.id),
            "episodeId": format!("ep_{}", ep_a.id),
            "currentTime": 5.0,
            "duration": 100.0
        },
        {
            "libraryItemId": format!("li_pod_{}", podcast.id),
            "episodeId": format!("ep_{}", ep_b.id),
            "currentTime": 50.0,
            "duration": 100.0,
            "isFinished": false
        }
    ]);
    let resp = server
        .test_server
        .patch("/api/me/progress/batch/update")
        .json(&body)
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);

    let me_resp = server.test_server.get("/api/me").await;
    let body: Value = me_resp.json();
    let progress = body["mediaProgress"].as_array().unwrap();
    assert_eq!(progress.len(), 2);
}

#[tokio::test]
#[serial]
async fn patch_me_progress_batch_rejects_empty_payload() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .patch("/api/me/progress/batch/update")
        .json(&json!([]))
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx for empty payload, got {}",
        resp.status_code()
    );
}

#[tokio::test]
#[serial]
async fn item_file_endpoint_streams_episode_audio() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let bytes = generate_pseudo_audio_bytes(2048);
    let path = write_temp_audio_file(&bytes, "item-file.mp3");
    let podcast = insert_test_podcast("Item file pod", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "ep", &path);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .get(&format!(
            "/api/items/li_pod_{}/file/ino_ep_{}",
            podcast.id, episode.id
        ))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    assert_eq!(resp.as_bytes().as_ref(), bytes.as_slice());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
#[serial]
async fn item_file_endpoint_honors_range() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let bytes = generate_pseudo_audio_bytes(4096);
    let path = write_temp_audio_file(&bytes, "range-file.mp3");
    let podcast = insert_test_podcast("Range item pod", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "ep", &path);
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .get(&format!(
            "/api/items/li_pod_{}/file/ino_ep_{}",
            podcast.id, episode.id
        ))
        .add_header("Range", "bytes=10-19")
        .await;
    assert_eq!(resp.status_code().as_u16(), 206);
    let body = resp.as_bytes();
    assert_eq!(body.len(), 10);
    assert_eq!(body.as_ref(), &bytes[10..20]);
    let _ = std::fs::remove_file(&path);
}

// ── Phase D3: file_watcher ─────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn file_watcher_starts_clean_when_no_book_library_has_folders() {
    use crate::services::audiobookshelf::file_watcher::AudiobookFileWatcher;
    let _server = handle_test_startup().await;
    let state = AppState::new();
    let watcher = AudiobookFileWatcher::new(
        state.audiobookshelf_library_service.clone(),
        state.audiobookshelf_scanner.clone(),
    );
    assert!(watcher.start().is_ok());
}

#[tokio::test]
#[serial]
async fn file_watcher_attaches_to_existing_folder_without_panicking() {
    use crate::services::audiobookshelf::file_watcher::AudiobookFileWatcher;
    let _server = handle_test_startup().await;
    let state = AppState::new();

    let tmp = std::env::temp_dir()
        .join("podfetch-abs-watcher")
        .join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&tmp).unwrap();
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    let mut updated = lib.clone();
    updated.folder_paths = vec![tmp.to_string_lossy().to_string()];
    use podfetch_domain::audiobookshelf::library::LibraryRepository;
    use podfetch_persistence::adapters::LibraryRepositoryImpl;
    use podfetch_persistence::db::database;
    LibraryRepositoryImpl::new(database())
        .upsert(updated)
        .unwrap();

    let watcher = AudiobookFileWatcher::new(
        state.audiobookshelf_library_service.clone(),
        state.audiobookshelf_scanner.clone(),
    );
    assert!(watcher.start().is_ok());
    let _ = std::fs::remove_dir_all(&tmp);
}

// ── Phase D: upload + scan reporting ────────────────────────────────────────

#[tokio::test]
#[serial]
async fn upload_writes_file_and_returns_target_dir() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    let upload_root = std::env::temp_dir()
        .join("podfetch-abs-uploads")
        .join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&upload_root).unwrap();
    let mut repo_lib = lib.clone();
    repo_lib.folder_paths = vec![upload_root.to_string_lossy().to_string()];
    use podfetch_domain::audiobookshelf::library::LibraryRepository;
    use podfetch_persistence::adapters::LibraryRepositoryImpl;
    use podfetch_persistence::db::database;
    LibraryRepositoryImpl::new(database())
        .upsert(repo_lib)
        .unwrap();

    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .post("/api/upload")
        .multipart(
            axum_test::multipart::MultipartForm::new()
                .add_text("library", lib.id.clone())
                .add_text("author", "Andy Weir")
                .add_text("title", "Hail Mary")
                .add_part(
                    "file",
                    axum_test::multipart::Part::bytes(b"fake-audio-bytes".to_vec())
                        .file_name("intro.mp3")
                        .mime_type("audio/mpeg"),
                ),
        )
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["libraryId"], json!(lib.id));
    let uploaded = body["uploadedFiles"].as_array().unwrap();
    assert_eq!(uploaded.len(), 1);
    assert_eq!(uploaded[0], json!("intro.mp3"));
    let target = body["targetDir"].as_str().unwrap();
    let intro_path = std::path::Path::new(target).join("intro.mp3");
    assert!(
        intro_path.is_file(),
        "expected file at {}",
        intro_path.display()
    );
    let content = std::fs::read(&intro_path).unwrap();
    assert_eq!(content, b"fake-audio-bytes");

    let _ = std::fs::remove_dir_all(&upload_root);
}

#[tokio::test]
#[serial]
async fn upload_without_library_returns_4xx() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .post("/api/upload")
        .multipart(
            axum_test::multipart::MultipartForm::new().add_part(
                "file",
                axum_test::multipart::Part::bytes(b"x".to_vec())
                    .file_name("a.mp3")
                    .mime_type("audio/mpeg"),
            ),
        )
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx, got {}",
        resp.status_code()
    );
}

#[tokio::test]
#[serial]
async fn upload_without_files_returns_4xx() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    login_audiobookshelf(&mut server, &user).await;

    let resp = server
        .test_server
        .post("/api/upload")
        .multipart(axum_test::multipart::MultipartForm::new().add_text("library", lib.id))
        .await;
    assert!(resp.status_code().is_client_error());
}

#[tokio::test]
#[serial]
async fn scan_report_exposes_added_updated_counters() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let lib = state
        .audiobookshelf_library_service
        .list()
        .unwrap()
        .into_iter()
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
        .unwrap();
    login_audiobookshelf(&mut server, &user).await;

    use podfetch_domain::audiobookshelf::library::LibraryRepository;
    use podfetch_persistence::adapters::LibraryRepositoryImpl;
    use podfetch_persistence::db::database;
    let empty = std::env::temp_dir()
        .join("podfetch-abs-empty-scan")
        .join(uuid::Uuid::new_v4().simple().to_string());
    std::fs::create_dir_all(&empty).unwrap();
    let mut updated = lib.clone();
    updated.folder_paths = vec![empty.to_string_lossy().to_string()];
    LibraryRepositoryImpl::new(database())
        .upsert(updated)
        .unwrap();

    let resp = server
        .test_server
        .post(&format!("/api/libraries/{}/scan", lib.id))
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["booksAdded"], json!(0));
    assert_eq!(body["booksUpdated"], json!(0));
    assert_eq!(body["scannedFolders"], json!(0));
    let _ = std::fs::remove_dir_all(&empty);
}

// ── Phase C: HLS + playMethod + listening-sessions ──────────────────────────

#[tokio::test]
#[serial]
async fn play_chooses_hls_when_client_lacks_source_codec() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS podcast", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "FLAC episode", "/tmp/episode.flac");

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
    assert_eq!(
        body["playMethod"],
        json!(1),
        "expected playMethod=1 (HLS), got body: {body}"
    );
    let tracks = body["audioTracks"].as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    let content_url = tracks[0]["contentUrl"].as_str().unwrap();
    assert!(
        content_url.ends_with("/master.m3u8"),
        "got contentUrl: {content_url}"
    );
    assert_eq!(
        tracks[0]["mimeType"],
        json!("application/vnd.apple.mpegurl")
    );
}

#[tokio::test]
#[serial]
async fn play_uses_direct_when_client_supports_source_codec() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("Direct podcast", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "MP3 episode", "/tmp/ep.mp3");

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
    // Upstream parity: direct-play tracks point at /api/items/<id>/file/<ino>
    // (PodcastEpisode.getAudioTrack), not at the /public/session share path.
    assert!(
        content_url.contains(&format!("/api/items/li_pod_{}/file/ino_ep_", podcast.id)),
        "got contentUrl: {content_url}"
    );
}

#[tokio::test]
#[serial]
async fn hls_master_playlist_returns_m3u8_for_owned_session() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);

    let podcast = insert_test_podcast("HLS pod", &state, user.id);
    let episode = insert_test_episode_with_path(&podcast.id, "HLS ep", "/tmp/x.flac");
    login_audiobookshelf(&mut server, &user).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode_with_path(&podcast.id, "Ep", "/tmp/x.flac");
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
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode_with_path(&podcast.id, "Ep", "/tmp/x.flac");
    login_audiobookshelf(&mut server, &user_a).await;
    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let episode = insert_test_episode(&podcast.id, "History ep");
    login_audiobookshelf(&mut server, &user).await;

    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({}))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
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
    assert_eq!(
        sessions.len(),
        1,
        "expected 1 listening session, got {sessions:#?}"
    );
    let entry = &sessions[0];
    assert_eq!(entry["mediaType"], json!("podcast"));
    assert_eq!(
        entry["libraryItemId"],
        json!(format!("li_pod_{}", podcast.id))
    );
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
    let episode = insert_test_episode(&podcast.id, "Iso ep");

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
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
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
    assert_eq!(
        sessions.len(),
        0,
        "user_b should see no sessions, got {sessions:#?}"
    );
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
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
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
    assert_eq!(
        results[0]["media"]["metadata"]["title"],
        json!("Project Hail Mary")
    );
    let authors = results[0]["media"]["metadata"]["authors"]
        .as_array()
        .unwrap();
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
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
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
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
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
        .find(|l| {
            matches!(
                l.media_type,
                podfetch_domain::audiobookshelf::library::MediaType::Book
            )
        })
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
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

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
    let author_repo: Arc<dyn AuthorRepository<Error = common_infrastructure::error::CustomError>> =
        Arc::new(AuthorRepositoryImpl::new(database()));

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

    let repo: Arc<dyn BookAudioFileRepository<Error = common_infrastructure::error::CustomError>> =
        Arc::new(BookAudioFileRepositoryImpl::new(database()));
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

    let repo: Arc<dyn BookChapterRepository<Error = common_infrastructure::error::CustomError>> =
        Arc::new(BookChapterRepositoryImpl::new(database()));
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

// ── /api/search/podcast + /api/podcasts/feed + /api/podcasts ────────────────

#[tokio::test]
#[serial]
async fn search_podcast_without_token_is_unauthorized() {
    let server = handle_test_startup().await;
    let response = server
        .test_server
        .get("/api/search/podcast?term=test")
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx without bearer, got {}",
        response.status_code()
    );
}

#[tokio::test]
#[serial]
async fn search_podcast_without_term_returns_empty_array() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server.test_server.get("/api/search/podcast").await;
    assert_eq!(response.status_code().as_u16(), 200);
    let body: Value = response.json();
    assert!(
        body.is_array(),
        "search must return a top-level JSON array (audiobookshelf-app reads results[0] without an envelope), got {body}"
    );
    assert_eq!(body.as_array().map(|v| v.len()), Some(0));
}

#[tokio::test]
#[serial]
async fn create_podcast_feed_rejects_non_http_url() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server
        .test_server
        .post("/api/podcasts/feed")
        .json(&json!({ "rssFeed": "ftp://example.com/feed.xml" }))
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx for non-http feed URL, got {}",
        response.status_code()
    );
}

#[tokio::test]
#[serial]
async fn create_podcast_rejects_empty_feed_url() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let response = server
        .test_server
        .post("/api/podcasts")
        .json(&json!({
            "media": { "metadata": { "feedUrl": "" } },
            "libraryId": "lib_default_podcasts"
        }))
        .await;
    assert!(
        response.status_code().is_client_error(),
        "expected 4xx for empty feedUrl, got {}",
        response.status_code()
    );
}

#[test]
fn parse_duration_seconds_handles_all_itunes_formats() {
    use crate::audiobookshelf_api::controllers::podcasts::parse_duration_seconds_for_test as p;
    assert_eq!(p(""), None);
    assert_eq!(p("garbage"), None);
    assert_eq!(p("3771"), Some(3771));
    assert_eq!(p("02:30"), Some(150));
    assert_eq!(p("01:02:30"), Some(3750));
}

#[test]
fn map_itunes_result_pins_audiobookshelf_shape() {
    use crate::audiobookshelf_api::controllers::search::map_itunes_result_for_test as m;
    use crate::podcast::ItunesModel;
    let r = ItunesModel {
        artist_id: Some(42),
        description: Some("HTML desc".to_string()),
        artist_view_url: None,
        kind: None,
        wrapper_type: None,
        collection_id: 9001,
        track_id: None,
        collection_censored_name: None,
        track_censored_name: None,
        artwork_url30: Some("http://img/30".to_string()),
        artwork_url60: Some("http://img/60".to_string()),
        artwork_url600: Some("http://img/600".to_string()),
        collection_price: None,
        track_price: None,
        release_date: Some("2024-01-02".to_string()),
        collection_explicitness: None,
        track_explicitness: Some("explicit".to_string()),
        track_count: Some(123),
        country: None,
        currency: None,
        primary_genre_name: None,
        content_advisory_rating: None,
        feed_url: Some("https://feeds/x".to_string()),
        collection_view_url: Some("https://itunes/x".to_string()),
        collection_hd_price: None,
        artist_name: Some("Alice".to_string()),
        track_name: None,
        collection_name: Some("Cool Podcast".to_string()),
        artwork_url_100: None,
        preview_url: None,
        track_view_url: "https://x".to_string(),
        track_time_millis: None,
        genre_ids: vec!["1310".to_string()],
        genres: vec!["Music".to_string(), "Comedy".to_string()],
    };
    let v = m(&r);
    // Pin the exact keys the Vue + Kotlin clients read. Renaming any of
    // these breaks the Add-Podcast search results page.
    assert_eq!(v["id"], json!(9001));
    assert_eq!(v["artistId"], json!(42));
    assert_eq!(v["title"], json!("Cool Podcast"));
    assert_eq!(v["artistName"], json!("Alice"));
    assert_eq!(v["releaseDate"], json!("2024-01-02"));
    assert_eq!(v["genres"], json!(["Music", "Comedy"]));
    assert_eq!(v["cover"], json!("http://img/600"));
    assert_eq!(v["trackCount"], json!(123));
    assert_eq!(v["feedUrl"], json!("https://feeds/x"));
    assert_eq!(v["pageUrl"], json!("https://itunes/x"));
    assert_eq!(v["explicit"], json!(true));
}

// ── /api/me/listening-stats ────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn listening_stats_returns_empty_buckets_for_new_user() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server.test_server.get("/api/me/listening-stats").await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    // Pin every key the upstream Vue dashboard reads — missing any of these
    // makes the chart panel throw on `undefined.totalTime` etc.
    assert_eq!(body["totalTime"], json!(0.0));
    assert_eq!(body["today"], json!(0.0));
    assert!(body["items"].is_object());
    assert!(body["days"].is_object());
    assert!(body["dayOfWeek"].is_object());
    assert!(body["recentSessions"].is_array());
}

#[tokio::test]
#[serial]
async fn listening_stats_aggregates_play_close_cycle() {
    use chrono::{Datelike, Utc};
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Stats Podcast", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Stats Episode");
    login_audiobookshelf(&mut server, &user).await;

    let play_resp = server
        .test_server
        .post(&format!(
            "/api/items/li_pod_{}/play/ep_{}",
            podcast.id, episode.id
        ))
        .json(&json!({ "mediaPlayer": "test", "supportedMimeTypes": ["audio/mpeg"] }))
        .await;
    let session_id = play_resp.json::<Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let close = server
        .test_server
        .post(&format!("/api/session/{session_id}/close"))
        .json(&json!({
            "currentTime": 90.0, "timeListened": 90.0, "duration": episode.total_time
        }))
        .await;
    assert_eq!(close.status_code().as_u16(), 200);

    let stats = server.test_server.get("/api/me/listening-stats").await;
    assert_eq!(stats.status_code().as_u16(), 200);
    let body: Value = stats.json();
    assert!(
        body["totalTime"].as_f64().unwrap() >= 89.0,
        "totalTime should include the 90s close, got {}",
        body["totalTime"]
    );
    let item_key = format!("li_pod_{}", podcast.id);
    assert!(
        body["items"][&item_key].is_object(),
        "per-libraryItem bucket missing for {item_key}, items={}",
        body["items"]
    );
    assert!(body["items"][&item_key]["timeListening"].as_f64().unwrap() >= 89.0);
    let today = Utc::now().format("%Y-%m-%d").to_string();
    assert!(
        body["days"][&today].is_number(),
        "today's day bucket missing, days={}",
        body["days"]
    );
    let weekday_key = match Utc::now().weekday() {
        chrono::Weekday::Mon => "Monday",
        chrono::Weekday::Tue => "Tuesday",
        chrono::Weekday::Wed => "Wednesday",
        chrono::Weekday::Thu => "Thursday",
        chrono::Weekday::Fri => "Friday",
        chrono::Weekday::Sat => "Saturday",
        chrono::Weekday::Sun => "Sunday",
    };
    assert!(
        body["dayOfWeek"][weekday_key].is_number(),
        "weekday bucket missing for {weekday_key}, dayOfWeek={}",
        body["dayOfWeek"]
    );
    assert_eq!(
        body["recentSessions"].as_array().map(|a| a.len()),
        Some(1),
        "exactly one closed session, recent={}",
        body["recentSessions"]
    );
}

// ── /api/playlists ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn list_playlists_starts_empty() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let resp = server.test_server.get("/api/playlists").await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert!(body["playlists"].is_array());
    assert_eq!(body["playlists"].as_array().unwrap().len(), 0);
}

#[tokio::test]
#[serial]
async fn create_playlist_returns_audiobookshelf_shape() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Playlist Podcast", &state, user.id);
    let episode = insert_test_episode(&podcast.id, "Playlist Episode");
    login_audiobookshelf(&mut server, &user).await;

    let create = server
        .test_server
        .post("/api/playlists")
        .json(&json!({
            "name": "My Listening Queue",
            "libraryId": "lib_default_podcasts",
            "items": [
                { "libraryItemId": format!("li_pod_{}", podcast.id),
                  "episodeId": format!("ep_{}", episode.id) }
            ]
        }))
        .await;
    assert_eq!(create.status_code().as_u16(), 200);
    let body: Value = create.json();
    // Pin shape used by audiobookshelf-web's playlist UI + Vue client.
    assert!(body["id"].as_str().is_some_and(|s| !s.is_empty()));
    assert_eq!(body["name"], json!("My Listening Queue"));
    assert_eq!(body["libraryId"], json!("lib_default_podcasts"));
    assert_eq!(body["userId"], json!(user.id.to_string()));
    let items = body["items"].as_array().expect("items array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["episodeId"], json!(format!("ep_{}", episode.id)));
    assert_eq!(
        items[0]["libraryItemId"],
        json!(format!("li_pod_{}", podcast.id))
    );
    assert!(items[0]["episode"].is_object());
    assert_eq!(items[0]["libraryItem"]["mediaType"], json!("podcast"));
}

#[tokio::test]
#[serial]
async fn create_playlist_rejects_missing_episode_id() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    let resp = server
        .test_server
        .post("/api/playlists")
        .json(&json!({
            "name": "Only Books",
            "items": [{ "libraryItemId": "li_book_xyz" }]
        }))
        .await;
    assert!(
        resp.status_code().is_client_error(),
        "expected 4xx without episodeId, got {}",
        resp.status_code()
    );
}

#[tokio::test]
#[serial]
async fn batch_add_and_remove_round_trip() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    let podcast = insert_test_podcast("Batch Podcast", &state, user.id);
    let ep1 = insert_test_episode(&podcast.id, "Episode 1");
    let ep2 = insert_test_episode(&podcast.id, "Episode 2");
    login_audiobookshelf(&mut server, &user).await;

    let created = server
        .test_server
        .post("/api/playlists")
        .json(&json!({ "name": "Empty", "items": [] }))
        .await;
    let playlist_id = created.json::<Value>()["id"]
        .as_str()
        .expect("playlist id")
        .to_string();

    let add = server
        .test_server
        .post(&format!("/api/playlists/{playlist_id}/batch/add"))
        .json(&json!({
            "items": [
                { "libraryItemId": format!("li_pod_{}", podcast.id),
                  "episodeId": format!("ep_{}", ep1.id) },
                { "libraryItemId": format!("li_pod_{}", podcast.id),
                  "episodeId": format!("ep_{}", ep2.id) }
            ]
        }))
        .await;
    assert_eq!(add.status_code().as_u16(), 200);
    let body: Value = add.json();
    assert_eq!(body["items"].as_array().map(|a| a.len()), Some(2));

    let remove = server
        .test_server
        .post(&format!("/api/playlists/{playlist_id}/batch/remove"))
        .json(&json!({
            "items": [
                { "libraryItemId": format!("li_pod_{}", podcast.id),
                  "episodeId": format!("ep_{}", ep1.id) }
            ]
        }))
        .await;
    assert_eq!(remove.status_code().as_u16(), 200);
    let after_remove: Value = remove.json();
    assert_eq!(after_remove["items"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(
        after_remove["items"][0]["episodeId"],
        json!(format!("ep_{}", ep2.id))
    );
}

#[tokio::test]
#[serial]
async fn delete_playlist_removes_it() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;

    let created = server
        .test_server
        .post("/api/playlists")
        .json(&json!({ "name": "Doomed", "items": [] }))
        .await;
    let playlist_id = created.json::<Value>()["id"]
        .as_str()
        .expect("playlist id")
        .to_string();

    let del = server
        .test_server
        .delete(&format!("/api/playlists/{playlist_id}"))
        .await;
    assert_eq!(del.status_code().as_u16(), 200);

    let list = server.test_server.get("/api/playlists").await;
    assert_eq!(
        list.json::<Value>()["playlists"]
            .as_array()
            .map(|a| a.len()),
        Some(0)
    );
}

#[tokio::test]
#[serial]
async fn playlist_belongs_only_to_owner() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let alice = create_user_for_audiobookshelf(&state);
    let bob = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &alice).await;
    let created = server
        .test_server
        .post("/api/playlists")
        .json(&json!({ "name": "Alice only", "items": [] }))
        .await;
    let playlist_id = created.json::<Value>()["id"]
        .as_str()
        .expect("playlist id")
        .to_string();

    login_audiobookshelf(&mut server, &bob).await;
    let bob_view = server
        .test_server
        .get(&format!("/api/playlists/{playlist_id}"))
        .await;
    assert!(
        bob_view.status_code().is_client_error(),
        "bob must not see alice's playlist, got {}",
        bob_view.status_code()
    );
    let bob_list = server.test_server.get("/api/playlists").await;
    assert_eq!(
        bob_list.json::<Value>()["playlists"]
            .as_array()
            .map(|a| a.len()),
        Some(0)
    );
}

#[tokio::test]
#[serial]
async fn library_playlists_endpoint_paginates() {
    let mut server = handle_test_startup().await;
    let state = AppState::new();
    let user = create_user_for_audiobookshelf(&state);
    login_audiobookshelf(&mut server, &user).await;
    for n in 0..3 {
        server
            .test_server
            .post("/api/playlists")
            .json(&json!({ "name": format!("PL {n}"), "items": [] }))
            .await;
    }
    let resp = server
        .test_server
        .get("/api/libraries/lib_default_podcasts/playlists")
        .await;
    assert_eq!(resp.status_code().as_u16(), 200);
    let body: Value = resp.json();
    assert_eq!(body["total"], json!(3));
    assert_eq!(body["results"].as_array().map(|a| a.len()), Some(3));
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
