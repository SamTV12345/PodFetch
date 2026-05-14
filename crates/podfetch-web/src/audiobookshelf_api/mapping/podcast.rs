//! Maps PodFetch's `Podcast` + `PodcastEpisode` domain entities to the
//! audiobookshelf-shaped `LibraryItem` JSON for `mediaType: "podcast"`.
//!
//! Returns `serde_json::Value` directly because the exact byte-shape comes
//! straight from upstream and not from any of our own structs.
//! References:
//!   - `server/models/LibraryItem.js::toOldJSONExpanded()`
//!   - `server/models/Podcast.js::toOldJSONExpanded(libraryItemId)`
//!   - `server/models/Podcast.js::oldMetadataToJSONExpanded()`
//!   - `server/models/PodcastEpisode.js::toOldJSONExpanded(libraryItemId)`
//!   - `server/models/PodcastEpisode.js::getAudioTrack(libraryItemId)`

use chrono::{NaiveDateTime, Utc};
use podfetch_domain::audiobookshelf::library_item_id::{EpisodeId, LibraryItemId};
use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use serde_json::{Value, json};
use std::path::Path;

/// Builds the inner `media`-style podcast JSON without the episodes array.
/// Upstream's `Podcast.toOldJSON()` is similar but explicitly clears
/// `podcastEpisodes` before serialising for some endpoints (e.g.
/// `recent-episodes` embeds it on each episode). Use this where you only
/// need the parent metadata.
pub fn map_podcast_without_episodes(podcast: &Podcast, library_id: &str) -> Value {
    let item_id = LibraryItemId::Podcast(podcast.id).as_string();
    let updated_ms = Utc::now().naive_utc().and_utc().timestamp_millis();
    json!({
        "id": format!("pod_{}", podcast.id),
        "libraryItemId": item_id,
        "libraryId": library_id,
        "metadata": metadata_json(podcast),
        "coverPath": format!("/api/items/{item_id}/cover"),
        "tags": Value::Array(vec![]),
        "episodes": Value::Array(vec![]),
        "autoDownloadEpisodes": podcast.active,
        "autoDownloadSchedule": Value::Null,
        "lastEpisodeCheck": updated_ms,
        "maxEpisodesToKeep": 0,
        "maxNewEpisodesToDownload": 0,
        "size": 0,
    })
}

/// Builds a single episode JSON with the parent `podcast` info embedded.
/// Used by `GET /api/libraries/:id/recent-episodes` to mirror upstream's
/// `libraryItemsPodcastFilters.getRecentEpisodes` output shape.
pub fn map_episode_for_recent(
    podcast: &Podcast,
    episode: &PodcastEpisode,
    index: i32,
    library_id: &str,
) -> Value {
    let item_id = LibraryItemId::Podcast(podcast.id).as_string();
    let mut ep_json = map_episode(episode, &item_id, podcast.id, index);
    if let Some(obj) = ep_json.as_object_mut() {
        obj.insert(
            "podcast".to_string(),
            map_podcast_without_episodes(podcast, library_id),
        );
        obj.insert("libraryId".to_string(), Value::from(library_id.to_string()));
    }
    ep_json
}

pub fn map_podcast(
    podcast: &Podcast,
    episodes: &[PodcastEpisode],
    library_id: &str,
) -> Value {
    let item_id = LibraryItemId::Podcast(podcast.id).as_string();
    let added_ms = episodes
        .iter()
        .filter_map(|e| e.download_time)
        .min()
        .unwrap_or_else(|| Utc::now().naive_utc())
        .and_utc()
        .timestamp_millis();
    let updated_ms = episodes
        .iter()
        .filter_map(|e| e.download_time)
        .max()
        .unwrap_or_else(|| Utc::now().naive_utc())
        .and_utc()
        .timestamp_millis();
    let active_episodes: Vec<&PodcastEpisode> =
        episodes.iter().filter(|e| !e.deleted).collect();
    let episodes_json: Vec<Value> = active_episodes
        .iter()
        .enumerate()
        .map(|(idx, ep)| map_episode(ep, &item_id, podcast.id, idx as i32 + 1))
        .collect();
    let num_episodes = episodes_json.len();

    let media = json!({
        "id": format!("pod_{}", podcast.id),
        "libraryItemId": item_id,
        "metadata": metadata_json(podcast),
        "coverPath": format!("/api/items/{item_id}/cover"),
        "tags": Value::Array(vec![]),
        "episodes": episodes_json,
        "autoDownloadEpisodes": podcast.active,
        "autoDownloadSchedule": Value::Null,
        "lastEpisodeCheck": updated_ms,
        "maxEpisodesToKeep": 0,
        "maxNewEpisodesToDownload": 0,
        "size": 0,
        // PodFetch-specific helper field; mobile apps ignore unknown keys.
        "numEpisodes": num_episodes,
    });

    json!({
        "id": item_id,
        "ino": format!("ino_pod_{}", podcast.id),
        "oldLibraryItemId": Value::Null,
        "libraryId": library_id,
        "folderId": Value::Null,
        "path": podcast.directory_name,
        "relPath": podcast.directory_name,
        "isFile": false,
        "mtimeMs": updated_ms,
        "ctimeMs": added_ms,
        "birthtimeMs": added_ms,
        "addedAt": added_ms,
        "updatedAt": updated_ms,
        "lastScan": updated_ms,
        "scanVersion": env!("CARGO_PKG_VERSION"),
        "isMissing": false,
        "isInvalid": false,
        "mediaType": "podcast",
        "media": media,
        "libraryFiles": Value::Array(vec![]),
        "numFiles": num_episodes,
        "size": 0,
    })
}

fn metadata_json(podcast: &Podcast) -> Value {
    let genres: Vec<Value> = podcast
        .keywords
        .as_deref()
        .map(|k| {
            k.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .map(Value::from)
                .collect()
        })
        .unwrap_or_default();
    let title_ignore_prefix = title_ignore_prefix(&podcast.name);
    json!({
        "title": podcast.name,
        "titleIgnorePrefix": title_ignore_prefix,
        "author": podcast.author,
        "description": podcast.summary,
        "releaseDate": podcast.last_build_date,
        "genres": genres,
        "feedUrl": podcast.rssfeed,
        "imageUrl": podcast.image_url,
        "itunesPageUrl": Value::Null,
        "itunesId": Value::Null,
        "itunesArtistId": Value::Null,
        "explicit": matches!(
            podcast.explicit.as_deref(),
            Some("yes") | Some("true") | Some("1")
        ),
        "language": podcast.language,
        "type": "episodic",
    })
}

/// Mirrors audiobookshelf `getTitlePrefixAtEnd` in `server/utils/index.js`:
/// "The Witcher" -> "Witcher, The".
fn title_ignore_prefix(title: &str) -> String {
    for prefix in ["The ", "A ", "An "] {
        if let Some(rest) = title.strip_prefix(prefix) {
            return format!("{rest}, {}", prefix.trim());
        }
    }
    title.to_string()
}

pub fn map_episode(
    episode: &PodcastEpisode,
    library_item_id: &str,
    podcast_id: i32,
    index: i32,
) -> Value {
    let published_at_naive: Option<NaiveDateTime> =
        chrono::DateTime::parse_from_rfc3339(&episode.date_of_recording)
            .ok()
            .map(|dt| dt.naive_utc())
            .or_else(|| {
                chrono::DateTime::parse_from_rfc2822(&episode.date_of_recording)
                    .ok()
                    .map(|dt| dt.naive_utc())
            });
    let published_at_ms = published_at_naive.map(|n| n.and_utc().timestamp_millis());
    let download_ms = episode
        .download_time
        .unwrap_or_else(|| Utc::now().naive_utc())
        .and_utc()
        .timestamp_millis();
    let local_path = episode
        .file_episode_path
        .clone()
        .or_else(|| episode.download_location.clone())
        .unwrap_or_else(|| episode.url.clone());
    let filename = Path::new(&local_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let ext = Path::new(&local_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime_type = mime_for_ext(&ext);
    let codec = codec_for_ext(&ext);
    let duration = episode.total_time as f64;
    let ino = format!("ino_ep_{}", episode.id);
    let audio_file_metadata = json!({
        "path": local_path,
        "filename": filename,
        "ext": ext,
    });
    let audio_file = json!({
        "index": 1,
        "ino": ino,
        "metadata": audio_file_metadata,
        "duration": duration,
        "bitRate": 0,
        "language": Value::Null,
        "codec": codec,
        "timeBase": "1/1000",
        "channels": 2,
        "channelLayout": "stereo",
        "chapters": Value::Array(vec![]),
        "embeddedCoverArt": Value::Null,
        "mimeType": mime_type,
    });
    let audio_track = json!({
        "index": 1,
        "ino": ino,
        "startOffset": 0.0,
        "title": filename,
        "duration": duration,
        "contentUrl": format!("/api/items/{library_item_id}/file/{ino}"),
        "mimeType": mime_type,
        "codec": codec,
        "metadata": audio_file_metadata.clone(),
    });
    let enclosure = if episode.url.is_empty() {
        Value::Null
    } else {
        json!({
            "url": episode.url,
            "type": mime_type,
            "length": Value::Null,
        })
    };
    json!({
        "libraryItemId": library_item_id,
        "podcastId": podcast_id,
        "id": EpisodeId(episode.id).as_string(),
        "oldEpisodeId": Value::Null,
        "index": index,
        "season": Value::Null,
        "episode": Value::Null,
        "episodeType": "full",
        "title": episode.name,
        "subtitle": Value::Null,
        "description": episode.description,
        "enclosure": enclosure,
        "guid": Some(episode.guid.clone()).filter(|s| !s.is_empty()).map(Value::from).unwrap_or(Value::Null),
        "pubDate": episode.date_of_recording,
        "chapters": Value::Array(vec![]),
        "audioFile": audio_file,
        "audioTrack": audio_track,
        "publishedAt": published_at_ms,
        "addedAt": download_ms,
        "updatedAt": download_ms,
        "duration": duration,
        "size": 0,
    })
}

fn mime_for_ext(ext: &str) -> String {
    match ext {
        "mp3" => "audio/mpeg",
        "m4a" | "m4b" | "mp4" | "aac" => "audio/mp4",
        "flac" => "audio/flac",
        "opus" | "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        "webm" | "webma" => "audio/webm",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn codec_for_ext(ext: &str) -> String {
    match ext {
        "mp3" => "mp3",
        "m4a" | "m4b" | "mp4" | "aac" => "aac",
        "flac" => "flac",
        "opus" => "opus",
        "ogg" | "oga" => "vorbis",
        "wav" => "pcm_s16le",
        _ => "unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_ignore_prefix_strips_articles() {
        assert_eq!(title_ignore_prefix("The Witcher"), "Witcher, The".to_string());
        assert_eq!(title_ignore_prefix("A Memory"), "Memory, A".to_string());
        assert_eq!(title_ignore_prefix("An Echo"), "Echo, An".to_string());
        assert_eq!(title_ignore_prefix("Witcher"), "Witcher".to_string());
    }

    #[test]
    fn mime_for_ext_handles_common_audio_types() {
        assert_eq!(mime_for_ext("mp3"), "audio/mpeg");
        assert_eq!(mime_for_ext("m4a"), "audio/mp4");
        assert_eq!(mime_for_ext("flac"), "audio/flac");
        assert_eq!(mime_for_ext("opus"), "audio/ogg");
        assert_eq!(mime_for_ext("xyz"), "application/octet-stream");
    }
}
