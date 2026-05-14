//! Maps PodFetch's `Podcast` + `PodcastEpisode` domain entities to the
//! audiobookshelf-shaped `LibraryItem` DTO.

use crate::audiobookshelf_api::dto::library_item::{
    AudioFileDto, AudioFileMetadataDto, AudioTrackInlineDto, EpisodeEnclosureDto, LibraryItemDto,
    PodcastEpisodeDto, PodcastMediaDto, PodcastMetadataDto,
};
use chrono::{NaiveDateTime, Utc};
use podfetch_domain::audiobookshelf::library_item_id::{EpisodeId, LibraryItemId};
use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use std::path::Path;

pub fn map_podcast(
    podcast: &Podcast,
    episodes: &[PodcastEpisode],
    library_id: &str,
) -> LibraryItemDto {
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

    let episode_dtos: Vec<PodcastEpisodeDto> = episodes
        .iter()
        .filter(|e| !e.deleted)
        .enumerate()
        .map(|(idx, episode)| map_episode(episode, &item_id, idx as i32 + 1))
        .collect();
    let num_episodes = episode_dtos.len() as i32;

    LibraryItemDto {
        id: item_id.clone(),
        ino: format!("ino_pod_{}", podcast.id),
        library_id: library_id.to_string(),
        folder_id: None,
        path: podcast.directory_name.clone(),
        rel_path: podcast.directory_name.clone(),
        is_file: false,
        mtime_ms: updated_ms,
        ctime_ms: added_ms,
        birthtime_ms: added_ms,
        added_at: added_ms,
        updated_at: updated_ms,
        last_scan: Some(updated_ms),
        scan_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        is_missing: false,
        is_invalid: false,
        media_type: "podcast".to_string(),
        media: PodcastMediaDto {
            metadata: PodcastMetadataDto {
                title: podcast.name.clone(),
                author: podcast.author.clone(),
                description: podcast.summary.clone(),
                release_date: podcast.last_build_date.clone(),
                genres: podcast
                    .keywords
                    .as_deref()
                    .map(|k| {
                        k.split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default(),
                feed_url: podcast.rssfeed.clone(),
                image_url: podcast.image_url.clone(),
                itunes_page_url: None,
                itunes_id: None,
                itunes_artist_id: None,
                explicit: matches!(
                    podcast.explicit.as_deref(),
                    Some("yes") | Some("true") | Some("1")
                ),
                language: podcast.language.clone(),
            },
            cover_path: Some(format!("/api/items/{item_id}/cover")),
            tags: Vec::new(),
            episodes: episode_dtos,
            auto_download_episodes: podcast.active,
            auto_download_schedule: None,
            last_episode_check: updated_ms,
            max_episodes_to_keep: 0,
            max_new_episodes_to_download: 0,
            num_episodes,
        },
        num_files: num_episodes,
        size: 0,
    }
}

fn map_episode(
    episode: &PodcastEpisode,
    library_item_id: &str,
    index: i32,
) -> PodcastEpisodeDto {
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

    let audio_metadata = AudioFileMetadataDto {
        path: local_path.clone(),
        filename: filename.clone(),
        ext: ext.clone(),
    };
    let enclosure = if episode.url.is_empty() {
        None
    } else {
        Some(EpisodeEnclosureDto {
            url: episode.url.clone(),
            r#type: mime_type.clone(),
            length: None,
        })
    };
    PodcastEpisodeDto {
        library_item_id: library_item_id.to_string(),
        podcast_id: episode.podcast_id,
        id: EpisodeId(episode.id).as_string(),
        old_episode_id: None,
        index,
        season: None,
        episode: None,
        episode_type: Some("full".to_string()),
        title: episode.name.clone(),
        subtitle: None,
        description: Some(episode.description.clone()),
        enclosure,
        guid: Some(episode.guid.clone()).filter(|s| !s.is_empty()),
        pub_date: Some(episode.date_of_recording.clone()),
        chapters: Vec::new(),
        audio_file: AudioFileDto {
            index: 1,
            ino: format!("ino_ep_{}", episode.id),
            metadata: audio_metadata.clone(),
            duration,
            bit_rate: 0,
            language: None,
            codec: codec.clone(),
            time_base: "1/1000".to_string(),
            channels: 2,
            channel_layout: "stereo".to_string(),
            chapters: Vec::new(),
            embedded_cover_art: None,
            mime_type: mime_type.clone(),
        },
        audio_track: AudioTrackInlineDto {
            index: 1,
            start_offset: 0.0,
            duration,
            title: episode.name.clone(),
            content_url: format!(
                "/api/items/{library_item_id}/file/ino_ep_{}",
                episode.id
            ),
            mime_type,
            codec,
            metadata: audio_metadata,
        },
        published_at: published_at_ms,
        added_at: download_ms,
        updated_at: download_ms,
        duration,
        size: 0,
    }
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
