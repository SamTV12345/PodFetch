//! Minimal `ffprobe` wrapper. Returns the JSON document; callers map fields to
//! the audiobookshelf domain shapes.

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ProbedAudioFile {
    pub duration: f64,
    pub bitrate: i32,
    pub codec: String,
    pub channels: i32,
    pub sample_rate: i32,
    pub tags: ProbedTags,
    pub chapters: Vec<ProbedChapter>,
    pub has_embedded_artwork: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ProbedTags {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub track: Option<String>,
    pub disc: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub series: Option<String>,
    pub series_part: Option<String>,
    pub grouping: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub publisher: Option<String>,
    pub explicit: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProbedChapter {
    pub start_time: f64,
    pub end_time: f64,
    pub title: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ProbeError {
    #[error("ffprobe binary not available or failed: {0}")]
    Spawn(String),
    #[error("ffprobe returned non-zero status: {0}")]
    ProbeFailed(String),
    #[error("could not parse ffprobe output: {0}")]
    ParseFailed(String),
}

pub fn probe_audio_file(path: &Path) -> Result<ProbedAudioFile, ProbeError> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            "-show_chapters",
        ])
        .arg(path)
        .output()
        .map_err(|e| ProbeError::Spawn(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ProbeError::ProbeFailed(stderr));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&stdout)
}

pub fn parse_ffprobe_json(json: &str) -> Result<ProbedAudioFile, ProbeError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| ProbeError::ParseFailed(e.to_string()))?;

    let format = value.get("format").cloned().unwrap_or_default();
    let duration = format
        .get("duration")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
        .or_else(|| format.get("duration").and_then(|v| v.as_f64()))
        .unwrap_or(0.0);
    let bitrate = format
        .get("bit_rate")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        .map(|v| v as i32)
        .unwrap_or(0);

    let streams = value
        .get("streams")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let audio_stream = streams
        .iter()
        .find(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("audio"))
        .cloned()
        .unwrap_or_default();
    let codec = audio_stream
        .get("codec_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let channels = audio_stream
        .get("channels")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(0);
    let sample_rate = audio_stream
        .get("sample_rate")
        .and_then(|v| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        .map(|v| v as i32)
        .unwrap_or(0);
    let has_embedded_artwork = streams
        .iter()
        .any(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("video"));

    let mut tags = ProbedTags::default();
    if let Some(tag_obj) = format.get("tags").and_then(|v| v.as_object()) {
        for (key, value) in tag_obj {
            apply_tag(&mut tags, key, value);
        }
    }
    if let Some(tag_obj) = audio_stream.get("tags").and_then(|v| v.as_object()) {
        for (key, value) in tag_obj {
            apply_tag(&mut tags, key, value);
        }
    }

    let mut chapters = Vec::new();
    if let Some(arr) = value.get("chapters").and_then(|v| v.as_array()) {
        for chapter in arr {
            let start = parse_time_field(chapter.get("start_time"));
            let end = parse_time_field(chapter.get("end_time"));
            let title = chapter
                .get("tags")
                .and_then(|t| t.get("title"))
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            chapters.push(ProbedChapter {
                start_time: start,
                end_time: end,
                title,
            });
        }
    }

    Ok(ProbedAudioFile {
        duration,
        bitrate,
        codec,
        channels,
        sample_rate,
        tags,
        chapters,
        has_embedded_artwork,
    })
}

fn apply_tag(tags: &mut ProbedTags, key: &str, value: &serde_json::Value) {
    let value = match value.as_str() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => return,
    };
    match key.to_ascii_lowercase().as_str() {
        "title" => tags.title.get_or_insert(value),
        "album" => tags.album.get_or_insert(value),
        "artist" => tags.artist.get_or_insert(value),
        "album_artist" => tags.album_artist.get_or_insert(value),
        "composer" => tags.composer.get_or_insert(value),
        "date" => tags.date.get_or_insert(value),
        "year" | "tyer" => tags.year.get_or_insert(value),
        "track" | "tracknumber" => tags.track.get_or_insert(value),
        "disc" | "discnumber" => tags.disc.get_or_insert(value),
        "genre" => tags.genre.get_or_insert(value),
        "comment" => tags.comment.get_or_insert(value),
        "description" => tags.description.get_or_insert(value),
        "series" | "show" | "tvshow" => tags.series.get_or_insert(value),
        "series-part" | "series_part" | "episode_id" | "tves" => tags.series_part.get_or_insert(value),
        "grouping" => tags.grouping.get_or_insert(value),
        "language" => tags.language.get_or_insert(value),
        "isbn" => tags.isbn.get_or_insert(value),
        "asin" => tags.asin.get_or_insert(value),
        "publisher" | "label" => tags.publisher.get_or_insert(value),
        "rating" | "itunesadvisory" | "explicit" => tags.explicit.get_or_insert(value),
        _ => return,
    };
}

fn parse_time_field(value: Option<&serde_json::Value>) -> f64 {
    match value {
        Some(v) => v
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v.as_f64())
            .unwrap_or(0.0),
        None => 0.0,
    }
}

pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &[
    "m4b", "m4a", "mp3", "mp4", "aac", "flac", "opus", "ogg", "oga", "wav", "webm", "webma", "wma",
    "aif", "aiff", "mka", "awb", "caf", "mpg", "mpeg",
];

pub fn is_supported_audio(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    SUPPORTED_AUDIO_EXTENSIONS.iter().any(|&s| s == ext)
}

pub fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "m4a" | "m4b" | "mp4" | "aac" => "audio/mp4",
        "flac" => "audio/flac",
        "opus" | "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        "webm" | "webma" => "audio/webm",
        "wma" => "audio/x-ms-wma",
        "aif" | "aiff" => "audio/aiff",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_ffprobe_json() {
        let json = r#"{
            "format": {"duration": "120.5", "bit_rate": "128000", "tags": {"title": "Demo", "artist": "Author"}},
            "streams": [{"codec_type": "audio", "codec_name": "mp3", "channels": 2, "sample_rate": "44100"}],
            "chapters": [{"start_time": "0.0", "end_time": "60.0", "tags": {"title": "Chapter 1"}}]
        }"#;
        let probed = parse_ffprobe_json(json).unwrap();
        assert!((probed.duration - 120.5).abs() < 0.01);
        assert_eq!(probed.bitrate, 128000);
        assert_eq!(probed.codec, "mp3");
        assert_eq!(probed.channels, 2);
        assert_eq!(probed.sample_rate, 44100);
        assert_eq!(probed.tags.title.as_deref(), Some("Demo"));
        assert_eq!(probed.tags.artist.as_deref(), Some("Author"));
        assert_eq!(probed.chapters.len(), 1);
        assert_eq!(probed.chapters[0].title, "Chapter 1");
    }

    #[test]
    fn detects_supported_extension() {
        assert!(is_supported_audio(std::path::Path::new("/foo/bar.m4b")));
        assert!(is_supported_audio(std::path::Path::new("/foo/bar.MP3")));
        assert!(!is_supported_audio(std::path::Path::new("/foo/bar.txt")));
    }

    #[test]
    fn detects_embedded_artwork_via_video_stream() {
        let json = r#"{
            "format": {"duration": "10.0", "bit_rate": "64000"},
            "streams": [
                {"codec_type": "audio", "codec_name": "aac", "channels": 2, "sample_rate": "44100"},
                {"codec_type": "video", "codec_name": "mjpeg"}
            ]
        }"#;
        let probed = parse_ffprobe_json(json).unwrap();
        assert!(probed.has_embedded_artwork);
    }
}
