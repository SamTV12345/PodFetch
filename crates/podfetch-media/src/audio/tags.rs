//! Writing an episode's metadata into its audio file (ID3v2.4 for MP3,
//! iTunes atoms for MP4).

use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use id3::{ErrorKind, Tag, TagLike};
use std::fs::File;
use std::io::Read;

/// What the media library writes into an episode's file, taken from the
/// catalog at the boundary.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpisodeTags {
    /// The title to set and whether it carries the episode number; `None`
    /// leaves the file's title alone (see [`crate::rules::resolve_episode_title`]).
    pub title: Option<(String, bool)>,
    /// The podcast's author.
    pub artist: Option<String>,
    /// The podcast's name.
    pub album: String,
    /// The podcast's keywords.
    pub genre: Option<String>,
    /// The podcast's language, used for the ID3 comment.
    pub language: Option<String>,
    /// The episode's description.
    pub comment: String,
    /// The episode's recording date as stored (RFC 3339).
    pub date_of_recording: String,
    /// The episode's position in the podcast.
    pub track_number: Option<i64>,
}

/// The tag layout of an audio file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagFormat {
    Id3,
    Mp4,
}

impl TagFormat {
    /// Detects the file's format. Formats PodFetch can't tag are an error.
    pub fn of_file(path: &str) -> Result<Self, CustomError> {
        use file_format::FileFormat;
        let detected = FileFormat::from_file(path).map_err(|e| {
            CustomErrorInner::Conflict(
                format!("Failed to detect file format for {path}: {e}"),
                ErrorSeverity::Warning,
            )
        })?;
        match detected {
            FileFormat::Mpeg12AudioLayer3
            | FileFormat::Mpeg12AudioLayer2
            | FileFormat::Id3v2
            | FileFormat::WaveformAudio => Ok(Self::Id3),
            FileFormat::Mpeg4Part14
            | FileFormat::Mpeg4Part14Audio
            | FileFormat::AppleItunesAudio => Ok(Self::Mp4),
            _ => {
                tracing::error!("File format not supported: {detected:?}");
                Err(CustomErrorInner::Conflict(
                    "File format not supported".to_string(),
                    ErrorSeverity::Error,
                )
                .into())
            }
        }
    }

    /// The chapters embedded in the file.
    pub fn read_chapters(self, path: &str) -> Result<Vec<super::chapters::Chapter>, CustomError> {
        match self {
            Self::Id3 => super::chapters::read_from_mp3(path),
            Self::Mp4 => Ok(super::chapters::read_from_mp4(path)),
        }
    }

    /// Writes `tags` into the file at `path`. `title_set` is called with the
    /// "carries the episode number" flag when a title is set, before the
    /// file is written; an error from it aborts the write.
    pub fn write(
        self,
        path: &str,
        image_path: &str,
        tags: &EpisodeTags,
        title_set: &mut dyn FnMut(bool) -> Result<(), CustomError>,
    ) -> Result<(), CustomError> {
        match self {
            Self::Id3 => write_id3(path, image_path, tags, title_set),
            Self::Mp4 => write_mp4(path, tags, title_set),
        }
    }
}

fn write_id3(
    path: &str,
    image_path: &str,
    tags: &EpisodeTags,
    title_set: &mut dyn FnMut(bool) -> Result<(), CustomError>,
) -> Result<(), CustomError> {
    let mut tag = match Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(id3::Error {
            kind: ErrorKind::NoTag,
            ..
        }) => Tag::new(),
        Err(err) => {
            return Err(CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Error).into());
        }
    };

    if let 0 = tag.pictures().count()
        && let Ok(mut image_file) = File::open(image_path)
    {
        let mut image_data = Vec::new();
        let _ = image_file.read_to_end(&mut image_data);
        tag.add_frame(id3::frame::Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: id3::frame::PictureType::CoverFront,
            description: "Cover".to_string(),
            data: image_data,
        });
    }

    if let Some((title, numbered)) = &tags.title {
        tag.set_title(title);
        title_set(*numbered)?;
    }

    if let Some(author) = &tags.artist {
        tag.set_artist(author);
    }

    tag.set_album(&tags.album);

    if let Some(recorded) = id3_timestamp(&tags.date_of_recording) {
        tag.set_date_recorded(recorded);
    }

    if tag.genres().is_none()
        && let Some(keywords) = &tags.genre
    {
        tag.set_genre(keywords);
    }

    if tag.clone().comments().next().is_none() {
        tag.add_frame(id3::frame::Comment {
            lang: tags.language.clone().unwrap_or("eng".to_string()),
            description: "Comment".to_string(),
            text: tags.comment.clone(),
        });
    }

    if tag.track().is_none()
        && let Some(track_number) = tags.track_number
    {
        tag.set_track(track_number as u32);
    }

    // Always write ID3v2.4 because otherwise there are compatibility issues with
    // embedded tags. A failed tag write must not fail the download.
    if let Err(e) = tag.write_to_path(path, id3::Version::Id3v24) {
        tracing::error!("Error writing metadata to {path}: {e}");
    }
    Ok(())
}

fn write_mp4(
    path: &str,
    tags: &EpisodeTags,
    title_set: &mut dyn FnMut(bool) -> Result<(), CustomError>,
) -> Result<(), CustomError> {
    let mut tag = match mp4ameta::Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(e) => {
            // Many M4A files (especially from Anchor/Spotify) have non-standard
            // atom layouts that mp4ameta can't parse. This is not fatal - the
            // audio file itself is fine, we just skip metadata writing.
            tracing::warn!("Could not read MP4 metadata of {path}, skipping metadata update: {e}");
            return Ok(());
        }
    };

    if let Some((title, numbered)) = &tags.title {
        tag.set_title(title);
        title_set(*numbered)?;
    }
    tag.set_artist(tags.artist.clone().unwrap_or("Unknown".to_string()));
    tag.set_album(&tags.album);
    tag.set_genre(tags.genre.clone().unwrap_or("Unknown".to_string()));
    tag.set_comment(&tags.comment);
    if let Some(track_number) = tags.track_number {
        tag.set_track_number(track_number as u16);
    }

    if let Err(e) = tag.write_to_path(path) {
        tracing::error!(
            "Error writing MP4 metadata to {path}, file may use an unsupported atom layout: {e}"
        );
    }
    Ok(())
}

/// Converts a stored RFC 3339 recording date into an ID3 timestamp.
///
/// ID3 timestamps carry no time zone, so the date is taken in UTC. Episodes
/// without a (parseable) date get no recording date tag.
fn id3_timestamp(date_of_recording: &str) -> Option<id3::Timestamp> {
    let date = chrono::DateTime::parse_from_rfc3339(date_of_recording)
        .ok()?
        .with_timezone(&chrono::Utc);
    date.format("%Y-%m-%dT%H:%M:%S").to_string().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id3_timestamp_from_stored_dates() {
        let ts = id3_timestamp("2025-01-02T10:30:00+01:00").unwrap();
        assert_eq!(ts.to_string(), "2025-01-02T09:30:00");
        // Episodes without a pubDate store an empty string.
        assert!(id3_timestamp("").is_none());
        assert!(id3_timestamp("garbage").is_none());
    }

    /// A file with an ID3 tag and nothing else; enough for tagging.
    fn tagged_file(name: &str, prepare: impl FnOnce(&mut Tag)) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("podfetch-media-tags-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("episode.mp3");
        std::fs::write(&path, [0xFF, 0xFB, 0x90, 0x00]).unwrap();
        let mut tag = Tag::new();
        prepare(&mut tag);
        tag.write_to_path(&path, id3::Version::Id3v24).unwrap();
        path
    }

    fn tags() -> EpisodeTags {
        EpisodeTags {
            title: Some(("3 - Episode".to_string(), true)),
            artist: Some("Author".to_string()),
            album: "Podcast".to_string(),
            genre: Some("Tech".to_string()),
            language: Some("deu".to_string()),
            comment: "About the episode".to_string(),
            date_of_recording: "2025-01-02T10:30:00+01:00".to_string(),
            track_number: Some(3),
        }
    }

    #[test]
    fn writes_the_episode_into_an_id3_tag() {
        let path = tagged_file("all", |_| {});
        let path_str = path.to_str().unwrap();
        let mut numbered = None;

        TagFormat::Id3
            .write(path_str, "/nonexistent.jpg", &tags(), &mut |n| {
                numbered = Some(n);
                Ok(())
            })
            .unwrap();

        let tag = Tag::read_from_path(&path).unwrap();
        assert_eq!(tag.title(), Some("3 - Episode"));
        assert_eq!(tag.artist(), Some("Author"));
        assert_eq!(tag.album(), Some("Podcast"));
        assert_eq!(tag.genre(), Some("Tech"));
        assert_eq!(tag.track(), Some(3));
        assert_eq!(
            tag.date_recorded().map(|d| d.to_string()),
            Some("2025-01-02T09:30:00".to_string())
        );
        let comment = tag.comments().next().unwrap();
        assert_eq!(comment.lang, "deu");
        assert_eq!(comment.text, "About the episode");
        assert_eq!(numbered, Some(true));
    }

    #[test]
    fn keeps_what_the_file_already_has() {
        let path = tagged_file("keep", |tag| {
            tag.set_title("Original");
            tag.set_genre("Comedy");
            tag.set_track(9);
            tag.add_frame(id3::frame::Comment {
                lang: "eng".to_string(),
                description: "Comment".to_string(),
                text: "From the publisher".to_string(),
            });
        });
        let untitled = EpisodeTags {
            title: None,
            ..tags()
        };
        let mut called = false;

        TagFormat::Id3
            .write(
                path.to_str().unwrap(),
                "/nonexistent.jpg",
                &untitled,
                &mut |_| {
                    called = true;
                    Ok(())
                },
            )
            .unwrap();

        let tag = Tag::read_from_path(&path).unwrap();
        assert_eq!(tag.title(), Some("Original"));
        assert_eq!(tag.genre(), Some("Comedy"));
        assert_eq!(tag.track(), Some(9));
        assert_eq!(tag.comments().count(), 1);
        assert!(!called, "no title set, nothing to record");
    }

    #[test]
    fn embeds_the_cover_once() {
        let path = tagged_file("cover", |_| {});
        let cover = path.with_file_name("image.jpg");
        std::fs::write(&cover, b"jpeg bytes").unwrap();

        for _ in 0..2 {
            TagFormat::Id3
                .write(
                    path.to_str().unwrap(),
                    cover.to_str().unwrap(),
                    &tags(),
                    &mut |_| Ok(()),
                )
                .unwrap();
        }

        let tag = Tag::read_from_path(&path).unwrap();
        let pictures: Vec<_> = tag.pictures().collect();
        assert_eq!(pictures.len(), 1);
        assert_eq!(pictures[0].data, b"jpeg bytes");
    }

    #[test]
    fn a_failing_title_callback_leaves_the_file_unchanged() {
        let path = tagged_file("abort", |tag| tag.set_title("Original"));

        let result = TagFormat::Id3.write(
            path.to_str().unwrap(),
            "/nonexistent.jpg",
            &tags(),
            &mut |_| Err(CustomErrorInner::Unknown(ErrorSeverity::Error).into()),
        );

        assert!(result.is_err());
        assert_eq!(
            Tag::read_from_path(&path).unwrap().title(),
            Some("Original")
        );
    }

    #[test]
    fn an_unreadable_mp4_is_skipped() {
        let path = tagged_file("mp4", |_| {});
        let mut called = false;

        TagFormat::Mp4
            .write(path.to_str().unwrap(), "", &tags(), &mut |_| {
                called = true;
                Ok(())
            })
            .unwrap();

        assert!(!called);
    }

    #[test]
    fn a_missing_file_has_no_format() {
        assert!(TagFormat::of_file("/nonexistent/podfetch/episode.mp3").is_err());
    }
}
