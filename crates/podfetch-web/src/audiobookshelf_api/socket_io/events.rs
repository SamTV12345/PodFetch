//! Payload builders for audiobookshelf-compatible socket.io events.
//!
//! These are intentionally plain `serde_json::Value`-building helpers so the
//! exact JSON shapes can be unit-tested without standing up an actual
//! socket.io server. Field names and types mirror the upstream emissions
//! exactly — see `server/SocketAuthority.js` and the event-by-event survey
//! in the spec.

use chrono::{NaiveDateTime, Utc};
use podfetch_domain::audiobookshelf::library::Library;
use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use podfetch_domain::user::User;
use serde_json::{Value, json};

/// audiobookshelf event name for the connection-init payload.
pub const EVENT_INIT: &str = "init";
pub const EVENT_USER_UPDATED: &str = "user_updated";
pub const EVENT_USER_ITEM_PROGRESS_UPDATED: &str = "user_item_progress_updated";
pub const EVENT_LIBRARY_ADDED: &str = "library_added";
pub const EVENT_LIBRARY_UPDATED: &str = "library_updated";
pub const EVENT_LIBRARY_REMOVED: &str = "library_removed";
pub const EVENT_ITEM_ADDED: &str = "item_added";
pub const EVENT_ITEM_UPDATED: &str = "item_updated";
pub const EVENT_ITEMS_ADDED: &str = "items_added";
pub const EVENT_ITEMS_UPDATED: &str = "items_updated";

/// First payload after a successful socket.io handshake.
/// Mirrors `SocketAuthority.js:325` — `{ userId, username }`.
pub fn build_init_payload(user: &User) -> Value {
    json!({
        "userId": user.id.to_string(),
        "username": user.username,
    })
}

/// `user_item_progress_updated` payload. Sent after every successful sync /
/// close to the user's own sockets, so other devices update their UI in
/// real time. Shape mirrors `PlaybackSessionManager.js:265`.
pub fn build_progress_updated_payload(
    progress: &MediaProgress,
    session_id: &str,
    device_description: &str,
) -> Value {
    json!({
        "id": progress.id,
        "sessionId": session_id,
        "deviceDescription": device_description,
        "data": media_progress_to_json(progress),
    })
}

/// `user_updated` payload — superset of `/api/me`. Field set mirrors
/// `User.js:604 toOldJSONForBrowser()`.
pub fn build_user_updated_payload(user: &User, media_progress: &[MediaProgress]) -> Value {
    let progress_array: Vec<Value> = media_progress.iter().map(media_progress_to_json).collect();
    json!({
        "id": user.id.to_string(),
        "username": user.username,
        "type": user_role_to_type(&user.role),
        "token": user.api_key.clone().unwrap_or_default(),
        "isActive": true,
        "isLocked": false,
        "mediaProgress": progress_array,
        "seriesHideFromContinueListening": Value::Array(vec![]),
        "bookmarks": Value::Array(vec![]),
        "lastSeen": Utc::now().timestamp_millis(),
        "createdAt": user.created_at.and_utc().timestamp_millis(),
        "permissions": permissions_for_role(&user.role),
        "librariesAccessible": Value::Array(vec![]),
        "itemTagsSelected": Value::Array(vec![]),
        "hasOpenIDLink": false,
    })
}

/// `library_updated` / `library_added` / `library_removed` payload. Mirrors
/// `Library.js:202 toOldJSON()`. PodFetch keeps fewer settings than upstream;
/// missing ones are filled with audiobookshelf-spec defaults so mobile apps
/// see a complete object.
pub fn build_library_payload(library: &Library) -> Value {
    let folders: Vec<Value> = library
        .folder_paths
        .iter()
        .map(|path| {
            json!({
                "id": format!("fol_{}", short_hash(path)),
                "path": path,
                "libraryId": library.id,
                "createdAt": library.created_at.and_utc().timestamp_millis(),
                "lastUpdate": library.updated_at.and_utc().timestamp_millis(),
            })
        })
        .collect();

    json!({
        "id": library.id,
        "name": library.name,
        "folders": folders,
        "displayOrder": library.display_order,
        "icon": library.icon,
        "mediaType": library.media_type.as_str(),
        "provider": Value::Null,
        "settings": {
            "coverAspectRatio": 1,
            "disableWatcher": false,
            "autoScanCronExpression": Value::Null,
            "skipMatchingMediaWithAsin": false,
            "skipMatchingMediaWithIsbn": false,
            "audiobooksOnly": false,
            "metadataPrecedence": library.metadata_precedence,
            "markAsFinishedPercentComplete": Value::Null,
            "markAsFinishedTimeRemaining": 10,
        },
        "lastScan": Value::Null,
        "lastScanVersion": env!("CARGO_PKG_VERSION"),
        "createdAt": library.created_at.and_utc().timestamp_millis(),
        "lastUpdate": library.updated_at.and_utc().timestamp_millis(),
    })
}

fn media_progress_to_json(progress: &MediaProgress) -> Value {
    json!({
        "id": progress.id,
        "userId": progress.user_id.to_string(),
        "libraryItemId": progress.library_item_id,
        "episodeId": progress.episode_id,
        "mediaItemId": progress
            .episode_id
            .clone()
            .unwrap_or_else(|| progress.library_item_id.clone()),
        "mediaItemType": if progress.episode_id.is_some() {
            "podcastEpisode"
        } else if progress.media_type == "book" {
            "book"
        } else {
            "podcastEpisode"
        },
        "duration": progress.duration,
        "progress": progress.progress,
        "currentTime": progress.current_time,
        "isFinished": progress.is_finished,
        "hideFromContinueListening": progress.hide_from_continue_listening,
        "ebookLocation": Value::Null,
        "ebookProgress": Value::Null,
        "lastUpdate": ms(progress.last_update),
        "startedAt": ms(progress.started_at),
        "finishedAt": progress.finished_at.map(ms).map(Value::from).unwrap_or(Value::Null),
    })
}

fn ms(value: NaiveDateTime) -> i64 {
    value.and_utc().timestamp_millis()
}

fn user_role_to_type(role: &str) -> &'static str {
    if role.eq_ignore_ascii_case("admin") {
        "root"
    } else {
        "user"
    }
}

fn permissions_for_role(role: &str) -> Value {
    let is_admin = role.eq_ignore_ascii_case("admin");
    json!({
        "download": true,
        "update": is_admin,
        "delete": is_admin,
        "upload": is_admin,
        "accessAllLibraries": true,
        "accessAllTags": true,
        "accessExplicitContent": true,
    })
}

fn short_hash(value: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn fixture_user() -> User {
        let created = chrono::Utc
            .with_ymd_and_hms(2026, 5, 13, 12, 0, 0)
            .unwrap()
            .naive_utc();
        User {
            id: 7,
            username: "sam".to_string(),
            role: "admin".to_string(),
            password: None,
            explicit_consent: true,
            created_at: created,
            api_key: Some("abs_token_xyz".to_string()),
            country: None,
            language: None,
        }
    }

    fn fixture_progress() -> MediaProgress {
        let now = chrono::Utc
            .with_ymd_and_hms(2026, 5, 13, 13, 0, 0)
            .unwrap()
            .naive_utc();
        MediaProgress {
            id: "li_pod_42-ep_7".to_string(),
            user_id: 7,
            library_item_id: "li_pod_42".to_string(),
            episode_id: Some("ep_7".to_string()),
            media_type: "podcast".to_string(),
            duration: 300.0,
            current_time: 150.0,
            progress: 0.5,
            is_finished: false,
            hide_from_continue_listening: false,
            last_update: now,
            started_at: now,
            finished_at: None,
        }
    }

    #[test]
    fn init_payload_has_audiobookshelf_shape() {
        let user = fixture_user();
        let payload = build_init_payload(&user);
        assert_eq!(payload["userId"], json!("7"));
        assert_eq!(payload["username"], json!("sam"));
        // No extra fields beyond the upstream init payload (usersOnline elided)
        assert_eq!(payload.as_object().unwrap().len(), 2);
    }

    #[test]
    fn progress_updated_wraps_session_id_and_device() {
        let progress = fixture_progress();
        let payload =
            build_progress_updated_payload(&progress, "play_session_abc", "iPhone 15 / Safari");
        assert_eq!(payload["id"], json!("li_pod_42-ep_7"));
        assert_eq!(payload["sessionId"], json!("play_session_abc"));
        assert_eq!(payload["deviceDescription"], json!("iPhone 15 / Safari"));
        let data = &payload["data"];
        assert_eq!(data["libraryItemId"], json!("li_pod_42"));
        assert_eq!(data["episodeId"], json!("ep_7"));
        assert_eq!(data["mediaItemId"], json!("ep_7"));
        assert_eq!(data["mediaItemType"], json!("podcastEpisode"));
        assert_eq!(data["currentTime"], json!(150.0));
        assert_eq!(data["progress"], json!(0.5));
        assert_eq!(data["isFinished"], json!(false));
        assert!(data["lastUpdate"].is_i64());
        assert_eq!(data["finishedAt"], Value::Null);
        assert_eq!(data["ebookLocation"], Value::Null);
    }

    #[test]
    fn book_progress_reports_book_media_type() {
        let mut progress = fixture_progress();
        progress.episode_id = None;
        progress.media_type = "book".to_string();
        progress.library_item_id = "li_book_uuid".to_string();
        progress.id = "li_book_uuid".to_string();
        let payload = build_progress_updated_payload(&progress, "play_x", "Device");
        assert_eq!(payload["data"]["episodeId"], Value::Null);
        assert_eq!(payload["data"]["mediaItemId"], json!("li_book_uuid"));
        assert_eq!(payload["data"]["mediaItemType"], json!("book"));
    }

    #[test]
    fn user_updated_includes_token_and_progress_array() {
        let user = fixture_user();
        let progress = fixture_progress();
        let payload = build_user_updated_payload(&user, &[progress]);
        assert_eq!(payload["id"], json!("7"));
        assert_eq!(payload["username"], json!("sam"));
        assert_eq!(payload["type"], json!("root"));
        assert_eq!(payload["token"], json!("abs_token_xyz"));
        assert_eq!(payload["isActive"], json!(true));
        assert_eq!(payload["mediaProgress"].as_array().unwrap().len(), 1);
        assert!(
            payload["permissions"]["accessAllLibraries"]
                .as_bool()
                .unwrap()
        );
        assert!(payload["permissions"]["update"].as_bool().unwrap()); // admin
    }

    #[test]
    fn library_payload_contains_required_fields() {
        let now = chrono::Utc::now().naive_utc();
        let library = Library {
            id: "lib_xyz".to_string(),
            name: "Audiobooks".to_string(),
            media_type: podfetch_domain::audiobookshelf::library::MediaType::Book,
            icon: "audiobookshelf".to_string(),
            display_order: 2,
            folder_paths: vec!["/audiobooks".to_string()],
            metadata_precedence: vec!["folderStructure".to_string(), "audioMetatags".to_string()],
            created_at: now,
            updated_at: now,
        };
        let payload = build_library_payload(&library);
        assert_eq!(payload["id"], json!("lib_xyz"));
        assert_eq!(payload["name"], json!("Audiobooks"));
        assert_eq!(payload["mediaType"], json!("book"));
        assert_eq!(payload["displayOrder"], json!(2));
        let folders = payload["folders"].as_array().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0]["path"], json!("/audiobooks"));
        assert_eq!(folders[0]["libraryId"], json!("lib_xyz"));
        assert_eq!(
            payload["settings"]["metadataPrecedence"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(payload["lastScanVersion"], json!(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn event_name_constants_match_upstream() {
        // Pin the wire names so a typo is caught at compile time AND fails this test.
        assert_eq!(EVENT_INIT, "init");
        assert_eq!(EVENT_USER_UPDATED, "user_updated");
        assert_eq!(
            EVENT_USER_ITEM_PROGRESS_UPDATED,
            "user_item_progress_updated"
        );
        assert_eq!(EVENT_LIBRARY_ADDED, "library_added");
        assert_eq!(EVENT_LIBRARY_UPDATED, "library_updated");
        assert_eq!(EVENT_LIBRARY_REMOVED, "library_removed");
        assert_eq!(EVENT_ITEM_ADDED, "item_added");
        assert_eq!(EVENT_ITEM_UPDATED, "item_updated");
        assert_eq!(EVENT_ITEMS_ADDED, "items_added");
        assert_eq!(EVENT_ITEMS_UPDATED, "items_updated");
    }
}
