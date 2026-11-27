//! Converting episode audio with ffmpeg: HLS streams into one file, any
//! format into Opus.

use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::http::COMMON_USER_AGENT;

/// True when the downloaded body is an HLS playlist (`#EXTM3U`) instead of
/// audio — some hosts (e.g. podtoo.com, issue #1402) serve HLS behind a
/// `.mp3` enclosure URL, so the suffix check never sees it.
pub fn is_hls_playlist(bytes: &[u8]) -> bool {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(0);
    bytes[start..].starts_with(b"#EXTM3U")
}

/// Let ffmpeg fetch the HLS segments and remux them into a single audio
/// file. Returns `(suffix, bytes)` like a direct download, so the
/// rest of the download flow (tagging, S3 upload, opus transcode) runs
/// unchanged.
pub fn download_hls_episode(url: &str) -> Result<(String, Vec<u8>), CustomError> {
    // Pick a lossless remux container from the segment codec: raw MP3
    // segments stay .mp3, everything else (usually ADTS AAC) goes into
    // .m4a — ffmpeg inserts the aac_adtstoasc filter automatically.
    let codec = std::process::Command::new("ffprobe")
        .args([
            "-user_agent",
            COMMON_USER_AGENT,
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=codec_name",
            "-of",
            "csv=p=0",
            url,
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        // ffprobe lists HLS streams once per section (programs + streams),
        // so the codec line can appear twice — take the first.
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .map(|l| l.trim().to_string())
        });
    let suffix = match codec.as_deref() {
        Some("mp3") => "mp3",
        _ => "m4a",
    };

    let tmp_path = std::env::temp_dir()
        .join(format!("podfetch-hls-{}.{suffix}", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .to_string();

    let output = std::process::Command::new("ffmpeg")
        .args([
            "-user_agent",
            COMMON_USER_AGENT,
            "-i",
            url,
            "-c",
            "copy",
            "-vn",
            "-y",
            &tmp_path,
        ])
        .output()
        .map_err(|e| {
            CustomErrorInner::Conflict(
                format!("Episode is served as HLS; ffmpeg is required to download it: {e}"),
                ErrorSeverity::Error,
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&tmp_path);
        return Err(CustomErrorInner::Conflict(
            format!("ffmpeg HLS download failed for {url}: {stderr}"),
            ErrorSeverity::Error,
        )
        .into());
    }

    let bytes = std::fs::read(&tmp_path);
    let _ = std::fs::remove_file(&tmp_path);
    let bytes = bytes.map_err(|e| {
        CustomErrorInner::Conflict(
            format!("Could not read ffmpeg HLS output: {e}"),
            ErrorSeverity::Error,
        )
    })?;
    Ok((suffix.to_string(), bytes))
}

/// Re-encodes the file at `input_path` to Opus next to it and removes the
/// original. Returns the path of the Opus file.
pub fn transcode_to_opus(input_path: &str) -> Result<String, CustomError> {
    let opus_path = {
        let p = std::path::Path::new(input_path);
        p.with_extension("opus").to_string_lossy().to_string()
    };

    let output = std::process::Command::new("ffmpeg")
        .args([
            "-i", input_path, "-c:a", "libopus", "-b:a", "48k", "-vn", "-y", &opus_path,
        ])
        .output()
        .map_err(|e| {
            CustomErrorInner::Conflict(format!("Failed to run ffmpeg: {e}"), ErrorSeverity::Warning)
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CustomErrorInner::Conflict(
            format!("ffmpeg transcode failed: {stderr}"),
            ErrorSeverity::Warning,
        )
        .into());
    }

    // Remove the original file after successful transcoding
    if let Err(e) = std::fs::remove_file(input_path) {
        tracing::warn!("Could not remove original file after opus transcode: {e}");
    }

    tracing::info!("Transcoded to opus: {opus_path}");
    Ok(opus_path)
}

#[cfg(test)]
mod tests {
    use super::is_hls_playlist;

    #[test]
    fn is_hls_playlist_detects_m3u8_bodies_only() {
        // plain playlist, with UTF-8 BOM, and with leading whitespace
        assert!(is_hls_playlist(b"#EXTM3U\n#EXT-X-VERSION:3\n"));
        assert!(is_hls_playlist(b"\xEF\xBB\xBF#EXTM3U\n"));
        assert!(is_hls_playlist(b"\n  #EXTM3U\n"));
        // real audio bodies must not match
        assert!(!is_hls_playlist(b"ID3\x04\x00\x00\x00\x00\x00\x00"));
        assert!(!is_hls_playlist(b"\xFF\xFB\x90\x00"));
        assert!(!is_hls_playlist(b""));
    }
}
