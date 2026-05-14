//! HLS playlist + on-demand segment generation.
//!
//! Phase C v1: single source audio file, six-second segments, transcoded via
//! `ffmpeg`. Segments cached on disk per session; the cache is bounded by
//! `audiobookshelf_hls_cache_max_mb` and capped concurrent transcodes by
//! `audiobookshelf_transcoder_max_concurrent`.
//!
//! The 100 % audiobookshelf-app compatible decision: if the client's
//! `supportedMimeTypes` does NOT include the source's MIME (or it sends
//! `forceTranscode=true`), the play handler picks playMethod=1 (HLS); else
//! playMethod=0 (direct).

use common_infrastructure::config::EnvironmentService;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Semaphore;

pub const SEGMENT_DURATION_SECONDS: f64 = 6.0;

/// Decides whether to transcode based on the mobile app's
/// `supportedMimeTypes` list and the source MIME.
pub fn should_use_hls(
    source_mime: &str,
    supported_mime_types: &[String],
    force_transcode: bool,
    force_direct: bool,
) -> bool {
    if force_direct {
        return false;
    }
    if force_transcode {
        return true;
    }
    if supported_mime_types.is_empty() {
        // Conservative default: if the client didn't tell us, stream directly.
        return false;
    }
    !supported_mime_types
        .iter()
        .any(|m| m.eq_ignore_ascii_case(source_mime))
}

/// Number of segments needed to cover the given duration.
pub fn segment_count(duration_seconds: f64) -> u32 {
    if duration_seconds <= 0.0 {
        return 0;
    }
    (duration_seconds / SEGMENT_DURATION_SECONDS).ceil() as u32
}

/// Audiobookshelf-shaped master playlist with a single variant.
pub fn build_master_playlist(stream_id: &str) -> String {
    format!(
        "#EXTM3U\n\
         #EXT-X-VERSION:3\n\
         #EXT-X-STREAM-INF:BANDWIDTH=128000,CODECS=\"mp4a.40.2\"\n\
         /hls/{stream_id}/index.m3u8\n"
    )
}

/// Media playlist (index.m3u8) listing each segment URL.
pub fn build_media_playlist(stream_id: &str, duration_seconds: f64) -> String {
    let total_segments = segment_count(duration_seconds);
    let target_duration = SEGMENT_DURATION_SECONDS.ceil() as u32;
    let mut out = format!(
        "#EXTM3U\n\
         #EXT-X-VERSION:3\n\
         #EXT-X-TARGETDURATION:{target_duration}\n\
         #EXT-X-MEDIA-SEQUENCE:0\n\
         #EXT-X-PLAYLIST-TYPE:VOD\n"
    );
    for idx in 0..total_segments {
        let segment_dur = if idx == total_segments.saturating_sub(1) {
            let remainder = duration_seconds - SEGMENT_DURATION_SECONDS * (idx as f64);
            remainder.clamp(0.0, SEGMENT_DURATION_SECONDS)
        } else {
            SEGMENT_DURATION_SECONDS
        };
        out.push_str(&format!(
            "#EXTINF:{segment_dur:.3},\n/hls/{stream_id}/seg-{idx}.ts\n"
        ));
    }
    out.push_str("#EXT-X-ENDLIST\n");
    out
}

#[derive(Clone)]
pub struct HlsTranscoder {
    pub environment: Arc<EnvironmentService>,
    pub concurrency: Arc<Semaphore>,
}

impl HlsTranscoder {
    pub fn new(environment: Arc<EnvironmentService>) -> Self {
        let max = environment.audiobookshelf_transcoder_max_concurrent.max(1) as usize;
        Self {
            environment,
            concurrency: Arc::new(Semaphore::new(max)),
        }
    }

    pub fn cache_root(&self) -> PathBuf {
        PathBuf::from(&self.environment.audiobookshelf_data_dir).join("hls")
    }

    pub fn segment_path(&self, stream_id: &str, segment: u32) -> PathBuf {
        self.cache_root()
            .join(stream_id)
            .join(format!("seg-{segment}.ts"))
    }

    /// Produce (or return cached) TS bytes for the requested segment.
    pub async fn ensure_segment(
        &self,
        stream_id: &str,
        source_audio: &Path,
        segment: u32,
    ) -> Result<PathBuf, TranscodeError> {
        let segment_file = self.segment_path(stream_id, segment);
        if segment_file.is_file() {
            return Ok(segment_file);
        }
        if let Some(parent) = segment_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TranscodeError::Io(e.to_string()))?;
        }

        let _permit = self
            .concurrency
            .acquire()
            .await
            .map_err(|_| TranscodeError::Other("semaphore closed".to_string()))?;

        if segment_file.is_file() {
            // Another concurrent caller produced it while we waited.
            return Ok(segment_file);
        }

        let start = (segment as f64) * SEGMENT_DURATION_SECONDS;
        let status = Command::new("ffmpeg")
            .args(["-v", "error", "-ss", &format!("{start}"), "-t"])
            .arg(format!("{SEGMENT_DURATION_SECONDS}"))
            .args(["-i"])
            .arg(source_audio)
            .args([
                "-map", "0:a:0", "-c:a", "aac", "-b:a", "128k", "-f", "mpegts", "-y",
            ])
            .arg(&segment_file)
            .status()
            .map_err(|e| TranscodeError::Spawn(e.to_string()))?;
        if !status.success() {
            return Err(TranscodeError::FfmpegFailed(status.code().unwrap_or(-1)));
        }
        if !segment_file.is_file() {
            return Err(TranscodeError::Other(
                "ffmpeg produced no segment output".to_string(),
            ));
        }
        self.evict_if_over_budget();
        Ok(segment_file)
    }

    pub fn evict_if_over_budget(&self) {
        let max_bytes = self.environment.audiobookshelf_hls_cache_max_mb * 1024 * 1024;
        let root = self.cache_root();
        let mut files: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
        collect_segment_files(&root, &mut files);
        let mut total: u64 = files.iter().map(|f| f.1).sum();
        if total <= max_bytes {
            return;
        }
        files.sort_by_key(|f| f.2);
        for (path, size, _) in files {
            if total <= max_bytes {
                break;
            }
            if std::fs::remove_file(&path).is_ok() {
                total = total.saturating_sub(size);
            }
        }
    }
}

fn collect_segment_files(root: &Path, out: &mut Vec<(PathBuf, u64, SystemTime)>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                out.push((path, metadata.len(), modified));
            } else if metadata.is_dir() {
                collect_segment_files(&path, out);
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TranscodeError {
    #[error("could not spawn ffmpeg: {0}")]
    Spawn(String),
    #[error("ffmpeg exited with status {0}")]
    FfmpegFailed(i32),
    #[error("io error: {0}")]
    Io(String),
    #[error("transcoder error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_use_hls_respects_force_direct_over_force_transcode() {
        assert!(!should_use_hls(
            "audio/flac",
            &["audio/mpeg".to_string()],
            true,
            true
        ));
    }

    #[test]
    fn should_use_hls_when_force_transcode() {
        assert!(should_use_hls(
            "audio/mpeg",
            &["audio/mpeg".to_string()],
            true,
            false
        ));
    }

    #[test]
    fn should_use_hls_when_codec_unsupported() {
        assert!(should_use_hls(
            "audio/flac",
            &["audio/mpeg".to_string(), "audio/mp4".to_string()],
            false,
            false
        ));
    }

    #[test]
    fn should_not_use_hls_when_codec_supported() {
        assert!(!should_use_hls(
            "audio/mpeg",
            &["audio/mpeg".to_string()],
            false,
            false
        ));
    }

    #[test]
    fn should_not_use_hls_when_client_list_empty() {
        assert!(!should_use_hls("audio/flac", &[], false, false));
    }

    #[test]
    fn segment_count_rounds_up() {
        assert_eq!(segment_count(0.0), 0);
        assert_eq!(segment_count(1.0), 1);
        assert_eq!(segment_count(6.0), 1);
        assert_eq!(segment_count(6.1), 2);
        assert_eq!(segment_count(12.0), 2);
        assert_eq!(segment_count(120.0), 20);
    }

    #[test]
    fn master_playlist_is_valid_m3u8() {
        let m = build_master_playlist("session_abc");
        assert!(m.starts_with("#EXTM3U"));
        assert!(m.contains("#EXT-X-STREAM-INF"));
        assert!(m.contains("/hls/session_abc/index.m3u8"));
    }

    #[test]
    fn media_playlist_lists_each_segment() {
        let m = build_media_playlist("session_xyz", 14.0);
        assert!(m.starts_with("#EXTM3U"));
        assert!(m.contains("#EXT-X-TARGETDURATION:6"));
        assert!(m.contains("/hls/session_xyz/seg-0.ts"));
        assert!(m.contains("/hls/session_xyz/seg-1.ts"));
        assert!(m.contains("/hls/session_xyz/seg-2.ts"));
        assert!(m.contains("#EXT-X-ENDLIST"));
        // last segment is 2 seconds long, formatted with 3 decimals
        assert!(m.contains("#EXTINF:2."));
    }

    #[test]
    fn media_playlist_handles_exact_segment_boundary() {
        let m = build_media_playlist("session_q", 12.0);
        assert!(m.contains("/hls/session_q/seg-0.ts"));
        assert!(m.contains("/hls/session_q/seg-1.ts"));
        assert!(!m.contains("/hls/session_q/seg-2.ts"));
    }
}
