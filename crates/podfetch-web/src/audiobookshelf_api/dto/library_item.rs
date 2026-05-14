use serde::Serialize;

#[derive(Serialize, utoipa::ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileMetadataDto {
    pub path: String,
    pub filename: String,
    pub ext: String,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioFileDto {
    pub index: i32,
    pub ino: String,
    pub metadata: AudioFileMetadataDto,
    pub duration: f64,
    pub bit_rate: i32,
    pub language: Option<String>,
    pub codec: String,
    pub time_base: String,
    pub channels: i32,
    pub channel_layout: String,
    pub chapters: Vec<ChapterDto>,
    pub embedded_cover_art: Option<String>,
    pub mime_type: String,
}

#[derive(Serialize, utoipa::ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDto {
    pub id: i32,
    pub start: f64,
    pub end: f64,
    pub title: String,
}

/// 100 % audiobookshelf-shape per upstream `PodcastEpisode.toOldJSONExpanded()`
/// (`server/models/PodcastEpisode.js`).
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeDto {
    pub library_item_id: String,
    pub podcast_id: i32,
    pub id: String,
    pub old_episode_id: Option<String>,
    pub index: i32,
    pub season: Option<String>,
    pub episode: Option<String>,
    pub episode_type: Option<String>,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<EpisodeEnclosureDto>,
    pub guid: Option<String>,
    pub pub_date: Option<String>,
    pub chapters: Vec<ChapterDto>,
    pub audio_file: AudioFileDto,
    pub audio_track: AudioTrackInlineDto,
    pub published_at: Option<i64>,
    pub added_at: i64,
    pub updated_at: i64,
    pub duration: f64,
    pub size: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeEnclosureDto {
    pub url: String,
    #[serde(rename = "type")]
    pub r#type: String,
    /// audiobookshelf serialises this as a string when set; null otherwise.
    pub length: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioTrackInlineDto {
    pub index: i32,
    pub start_offset: f64,
    pub duration: f64,
    pub title: String,
    pub content_url: String,
    pub mime_type: String,
    pub codec: String,
    pub metadata: AudioFileMetadataDto,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastMetadataDto {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub feed_url: String,
    pub image_url: String,
    pub itunes_page_url: Option<String>,
    pub itunes_id: Option<String>,
    pub itunes_artist_id: Option<String>,
    pub explicit: bool,
    pub language: Option<String>,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastMediaDto {
    pub metadata: PodcastMetadataDto,
    pub cover_path: Option<String>,
    pub tags: Vec<String>,
    pub episodes: Vec<PodcastEpisodeDto>,
    pub auto_download_episodes: bool,
    pub auto_download_schedule: Option<String>,
    pub last_episode_check: i64,
    pub max_episodes_to_keep: i32,
    pub max_new_episodes_to_download: i32,
    pub num_episodes: i32,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemDto {
    pub id: String,
    pub ino: String,
    pub library_id: String,
    pub folder_id: Option<String>,
    pub path: String,
    pub rel_path: String,
    pub is_file: bool,
    pub mtime_ms: i64,
    pub ctime_ms: i64,
    pub birthtime_ms: i64,
    pub added_at: i64,
    pub updated_at: i64,
    pub last_scan: Option<i64>,
    pub scan_version: Option<String>,
    pub is_missing: bool,
    pub is_invalid: bool,
    pub media_type: String,
    pub media: PodcastMediaDto,
    pub num_files: i32,
    pub size: i64,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemsResponse {
    pub results: Vec<LibraryItemDto>,
    pub total: i64,
    pub limit: i64,
    pub page: i64,
}
