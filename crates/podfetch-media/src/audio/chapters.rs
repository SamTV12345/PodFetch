//! Chapters embedded in an episode's audio file (ID3 `CHAP` frames, MP4
//! chapter lists).

use chrono::Duration;
use common_infrastructure::error::CustomError;

pub type Image = url::Url;

/// Represents a web link for a [`Chapter`].
#[derive(Debug, PartialEq, Clone)]
pub struct Link {
    /// The URL of the link.
    pub url: url::Url,
    /// The title of the link.
    pub title: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub struct Chapter {
    /// The starting time of the chapter.
    pub start: Duration,
    /// The end time of the chapter.
    pub end: Option<Duration>,
    /// The title of this chapter.
    pub title: Option<String>,
    /// The image to use as chapter art.
    pub image: Option<Image>,
    /// Web page or supporting document related to the topic of this chapter.
    pub link: Option<Link>,
    /// If this property is set to true, this chapter should not display visibly to the user in either the table of contents or as a jump-to point in the user interface. In the original spec, the inverse of this is called `toc`.
    pub hidden: bool,
}

/// The chapters of an ID3-tagged file, ordered by start. A file without a
/// readable tag has none.
pub fn read_from_mp3(path: &str) -> Result<Vec<Chapter>, CustomError> {
    let tag = id3::Tag::read_from_path(path)
        .map_err(|e| format!("Error reading ID3 tag from `{path}`: {e}"));

    let tag = match tag {
        Ok(tag) => tag,
        Err(err) => {
            tracing::error!("Error reading ID3 tag: {}", err);
            return Ok(Vec::new());
        }
    };

    let mut chapters = Vec::new();

    for id3_chapter in tag.chapters() {
        let start = Duration::milliseconds(id3_chapter.start_time as i64);

        let temp_end = Duration::milliseconds(id3_chapter.end_time as i64);
        // Some programs might encode chapters as instants, i.e., with the start and end time being the same.
        let end = if temp_end == start {
            None
        } else {
            Some(temp_end)
        };

        let mut title = None;
        let mut link = None;

        for subframe in &id3_chapter.frames {
            match subframe.content() {
                id3::Content::Text(text) => {
                    title = Some(text.clone());
                }
                id3::Content::Link(url) => {
                    link = Some(Link {
                        url: url::Url::parse(url)
                            .map_err(<url::ParseError as Into<CustomError>>::into)?,
                        title: None,
                    });
                }
                id3::Content::ExtendedLink(extended_link) => {
                    link = Some(Link {
                        url: url::Url::parse(&extended_link.link)
                            .map_err(<url::ParseError as Into<CustomError>>::into)?,
                        title: match extended_link.description.trim() {
                            "" => None,
                            description => Some(description.to_string()),
                        },
                    });
                }
                _ => {}
            }
        }

        chapters.push(Chapter {
            start,
            end,
            title,
            link,
            ..Default::default()
        });
    }

    // Order chapters by start time.
    chapters.sort_by_key(|a| a.start);

    Ok(chapters)
}

/// The chapters of an MP4 file; each one ends where the next starts.
pub fn read_from_mp4(path: &str) -> Vec<Chapter> {
    let tag = mp4ameta::Tag::read_from_path(path);
    let tag = match tag {
        Ok(tag) => tag,
        Err(err) => {
            tracing::error!("Error reading MP4 tag: {}", err);
            return Vec::new();
        }
    };
    let chapters_list: Vec<_> = tag.chapter_list().iter().collect();
    let mut chapters = Vec::new();
    for (index, id3_chapter) in chapters_list.iter().enumerate() {
        let start = Duration::milliseconds(id3_chapter.start.as_millis() as i64);

        let end = if index + 1 < chapters_list.len() {
            Some(Duration::milliseconds(
                chapters_list[index + 1].start.as_millis() as i64,
            ))
        } else {
            None
        };

        let title = if id3_chapter.title.is_empty() {
            None
        } else {
            Some(id3_chapter.title.clone())
        };

        chapters.push(Chapter {
            title,
            link: None,
            start,
            end,
            ..Default::default()
        });
    }
    chapters
}
