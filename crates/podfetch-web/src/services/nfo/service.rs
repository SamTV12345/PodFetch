//! Resolves the NFO format from settings and writes NFO files. All writes are
//! best-effort: failures are logged, never propagated (must not fail a
//! download or a settings save).

use super::NfoFormat;
use super::builders;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use podfetch_persistence::podcast::PodcastEntity;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity;
use podfetch_storage::{FileHandleWrapper, FileRequest};
use std::str::FromStr;
use uuid::Uuid;

const COVER_CANDIDATES: &[&str] = &["image", "cover", "folder", "poster"];

/// Resolve the effective NFO format: per-podcast override (when activated)
/// falls back to the global setting.
pub fn resolve_nfo_format(podcast_id: Uuid) -> NfoFormat {
    let global = SettingsService::shared()
        .get_settings()
        .ok()
        .flatten()
        .map(|s| s.nfo_format)
        .unwrap_or_default();
    let raw = match PodcastSettingsService::get_settings_for_podcast(podcast_id) {
        Ok(Some(ps)) if ps.activated => ps.nfo_format,
        _ => global,
    };
    NfoFormat::from_str(&raw).unwrap_or_default()
}

/// Resolve the effective cover base name (empty -> "image").
pub fn resolve_cover_filename(podcast_id: Uuid) -> String {
    let global = SettingsService::shared()
        .get_settings()
        .ok()
        .flatten()
        .map(|s| s.cover_filename)
        .unwrap_or_default();
    let raw = match PodcastSettingsService::get_settings_for_podcast(podcast_id) {
        Ok(Some(ps)) if ps.activated => ps.cover_filename,
        _ => global,
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "image".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Derive the sidecar `.nfo` path from a media file path by swapping its
/// extension. If the path has no extension, append `.nfo`.
pub fn nfo_path_for(audio_path: &str) -> String {
    let last_sep = audio_path.rfind(['/', '\\']);
    match audio_path.rfind('.') {
        Some(dot) if last_sep.is_none_or(|sep| dot > sep) => {
            format!("{}.nfo", &audio_path[..dot])
        }
        _ => format!("{audio_path}.nfo"),
    }
}

fn write_nfo_file(path: &str, xml: &str) {
    let mut bytes = xml.as_bytes().to_vec();
    if let Err(err) = FileHandleWrapper::write_file(
        path,
        bytes.as_mut_slice(),
        &ENVIRONMENT_SERVICE.default_file_handler,
    ) {
        tracing::warn!("Failed to write NFO file {path}: {err}");
    }
}

/// Generate NFO for one episode (and refresh the podcast-level NFO). Non-fatal.
/// `audio_path` is the FINAL media path (post-transcode), so the per-episode
/// `.nfo` basename matches the audio file.
pub fn regenerate_for_episode(
    podcast_entity: &PodcastEntity,
    episode_entity: &PodcastEpisodeEntity,
    audio_path: &str,
) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    match resolve_nfo_format(podcast_uuid) {
        NfoFormat::Off => {}
        NfoFormat::Tvshow => {
            let podcast = Podcast::from(podcast_entity.clone());
            let episode = PodcastEpisode::from(episode_entity.clone());
            let position = PodcastEpisodeService::get_position_of_episode(
                &episode.date_of_recording,
                podcast_uuid,
            )
            .unwrap_or(0) as i64;
            write_nfo_file(
                &nfo_path_for(audio_path),
                &builders::build_episodedetails_nfo(&podcast, &episode, position),
            );
            write_nfo_file(
                &format!("{}/tvshow.nfo", podcast_entity.directory_name),
                &builders::build_tvshow_nfo(&podcast),
            );
        }
        NfoFormat::Album => write_album_nfo(podcast_entity),
    }
}

/// Rewrite `album.nfo` from the podcast's full downloaded-episode set. Non-fatal.
fn write_album_nfo(podcast_entity: &PodcastEntity) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    let podcast = Podcast::from(podcast_entity.clone());
    let mut episodes = match PodcastEpisodeService::get_episodes_by_podcast_id(podcast_uuid) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!("album.nfo: could not load episodes: {err}");
            return;
        }
    };
    episodes.retain(|e| e.download_time.is_some());
    episodes.sort_by(|a, b| a.date_of_recording.cmp(&b.date_of_recording));
    let tracks: Vec<(PodcastEpisode, i64)> = episodes
        .into_iter()
        .enumerate()
        .map(|(i, e)| (PodcastEpisode::from(e), (i as i64) + 1))
        .collect();
    write_nfo_file(
        &format!("{}/album.nfo", podcast_entity.directory_name),
        &builders::build_album_nfo(&podcast, &tracks),
    );
}

/// Rename the on-disk podcast cover to the configured base name when needed.
/// Best-effort, Local backend only (rename is unsupported on S3).
pub fn ensure_cover_filename(podcast_entity: &PodcastEntity) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    let target = resolve_cover_filename(podcast_uuid);
    let dir = &podcast_entity.directory_name;
    if dir.is_empty() {
        return;
    }
    let Some(ext) = std::path::Path::new(&podcast_entity.image_url)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
    else {
        return;
    };

    let fh = &ENVIRONMENT_SERVICE.default_file_handler;
    let target_path = format!("{dir}/{target}.{ext}");
    if FileHandleWrapper::path_exists(&target_path, FileRequest::File, fh) {
        return; // already correct
    }
    for cand in COVER_CANDIDATES {
        if *cand == target {
            continue;
        }
        let cand_path = format!("{dir}/{cand}.{ext}");
        if !FileHandleWrapper::path_exists(&cand_path, FileRequest::File, fh) {
            continue;
        }
        match FileHandleWrapper::rename_file(&cand_path, &target_path, fh) {
            Ok(()) => {
                tracing::info!("Renamed cover {cand_path} -> {target_path}");
                let new_url = format!(
                    "{}/{}.{}",
                    PodcastEpisodeService::map_to_local_url(dir),
                    target,
                    ext
                );
                if let Err(err) =
                    PodcastEpisodeService::update_podcast_image(&podcast_entity.id, &new_url)
                {
                    tracing::warn!("Cover renamed but DB image_url update failed: {err}");
                }
            }
            Err(err) => tracing::warn!("Cover rename {cand_path} -> {target_path} failed: {err}"),
        }
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::nfo_path_for;

    #[test]
    fn nfo_path_swaps_extension_or_appends() {
        assert_eq!(nfo_path_for("podcasts/x/audio.mp3"), "podcasts/x/audio.nfo");
        assert_eq!(
            nfo_path_for("podcasts/My Show/2023-09-07 - Ep.mp3"),
            "podcasts/My Show/2023-09-07 - Ep.nfo"
        );
        // dot only in a directory component -> treat as no extension
        assert_eq!(
            nfo_path_for("podcasts/My.Show/episode"),
            "podcasts/My.Show/episode.nfo"
        );
        // windows separators
        assert_eq!(nfo_path_for("podcasts\\x\\audio.opus"), "podcasts\\x\\audio.nfo");
    }
}
