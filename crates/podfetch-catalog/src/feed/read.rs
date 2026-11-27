//! Reading what the catalog keeps from a podcast feed: dates, durations,
//! images, metadata and the `<podcast:transcript>` tags.

use crate::domain::podcast::PodcastMetadataUpdate;
use chrono::{DateTime, TimeZone, Utc};
use rss::{Channel, Item};
use uuid::Uuid;

/// A `<podcast:transcript>` tag of a feed item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedTranscript {
    pub url: String,
    pub mime_type: String,
    pub language: Option<String>,
}

/// Normalises an item's `pubDate` to RFC 3339; empty when it has none.
///
/// Feeds are sloppy, so besides RFC 2822 and RFC 3339 this accepts an
/// RFC 2822 date with a non-standard weekday ("Thurs, 2 Jan 2025 …").
/// Unparseable dates fall back to 2021-01-01 so the episode is still kept.
pub fn recording_date(item: &Item) -> String {
    let Some(date) = item.pub_date.as_deref() else {
        return String::new();
    };
    let without_weekday = |s: &str| {
        let (_, rest) = s.split_once(',')?;
        DateTime::parse_from_str(rest.trim(), "%d %b %Y %H:%M:%S %z").ok()
    };
    DateTime::parse_from_rfc2822(date)
        .or_else(|_| DateTime::parse_from_rfc3339(date))
        .ok()
        .or_else(|| without_weekday(date))
        .map_or_else(
            || {
                Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0)
                    .unwrap()
                    .to_rfc3339()
            },
            |parsed| parsed.with_timezone(&Utc).to_rfc3339(),
        )
}

/// An `itunes:duration` in seconds: `s`, `m:s`, `h:m:s` or `d:h:m:s`;
/// 0 if it can't be read.
pub fn duration(duration_str: &str) -> u32 {
    let parts: Vec<&str> = duration_str.split(':').collect();
    match parts.len() {
        1 => parts[0].parse::<u32>().unwrap_or(0),
        2 => {
            let minutes = parts[0].parse::<u32>().unwrap_or(0);
            let seconds = parts[1].parse::<u32>().unwrap_or(0);
            minutes * 60 + seconds
        }
        3 => {
            let hours = parts[0].parse::<u32>().unwrap_or(0);
            let minutes = parts[1].parse::<u32>().unwrap_or(0);
            let seconds = parts[2].parse::<u32>().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        4 => {
            let days = parts[0].parse::<u32>().unwrap_or(0);
            let hours = parts[1].parse::<u32>().unwrap_or(0);
            let minutes = parts[2].parse::<u32>().unwrap_or(0);
            let seconds = parts[3].parse::<u32>().unwrap_or(0);
            days * 86400 + hours * 3600 + minutes * 60 + seconds
        }
        _ => 0,
    }
}

/// Reads `<podcast:transcript>` tags out of a feed item's extensions.
///
/// The `rss` crate keys parsed extensions by their local name, i.e. without
/// the namespace prefix, so a `<podcast:transcript>` element normally lands
/// under `extensions()["podcast"]["transcript"]`. Some feeds/tooling can
/// still surface the qualified name as the map key instead, so both
/// `"transcript"` and `"podcast:transcript"` are checked defensively. A tag
/// without a `url` attribute is skipped — there is nothing to store or later
/// download (Task 8) for it.
pub fn transcripts(item: &Item) -> Vec<FeedTranscript> {
    let Some(podcast_ns) = item.extensions().get("podcast") else {
        return Vec::new();
    };

    ["transcript", "podcast:transcript"]
        .into_iter()
        .filter_map(|key| podcast_ns.get(key))
        .flatten()
        .filter_map(|ext| {
            let url = ext.attrs().get("url")?.clone();
            let mime_type = ext
                .attrs()
                .get("type")
                .cloned()
                .unwrap_or_else(|| "text/plain".to_string());
            let language = ext.attrs().get("language").cloned();
            Some(FeedTranscript {
                url,
                mime_type,
                language,
            })
        })
        .collect()
}

/// The podcast's main image from an RSS channel: `<image><url>`, falling
/// back to `<itunes:image href>` - many feeds (e.g. Substack) only have the
/// latter.
pub fn channel_image_url(channel: &rss::Channel) -> Option<String> {
    let non_empty = |url: &str| {
        let url = url.trim();
        (!url.is_empty()).then(|| url.to_string())
    };
    channel
        .image()
        .and_then(|image| non_empty(&image.url))
        .or_else(|| {
            channel
                .itunes_ext()
                .and_then(|ext| ext.image())
                .and_then(non_empty)
        })
}

/// The podcast fields a feed carries. Feeds without the iTunes extension
/// leave the podcast as it is. Empty values are stored as none; the
/// explicit flag isn't read from the feed and stays "false".
pub fn metadata(channel: &Channel, podcast_id: Uuid) -> Option<PodcastMetadataUpdate> {
    let itunes = channel.itunes_ext.as_ref()?;
    let non_empty = |value: Option<String>| value.filter(|v| !v.is_empty());
    let guid = channel
        .extensions()
        .get("podcast")
        .and_then(|m| m.get("guid"))
        .and_then(|v| v.first())
        .and_then(|v| v.value.clone());
    let keywords = itunes
        .categories
        .iter()
        .map(|c| c.text.clone())
        .collect::<Vec<String>>()
        .join(",");
    Some(PodcastMetadataUpdate {
        id: podcast_id,
        author: non_empty(itunes.author.clone()),
        keywords: non_empty(Some(keywords)),
        explicit: Some(false.to_string()),
        language: non_empty(channel.language.clone()),
        description: non_empty(Some(channel.description.clone())),
        last_build_date: non_empty(channel.last_build_date.clone()),
        guid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item_with_date(date: &str) -> Item {
        Item {
            pub_date: Some(date.to_owned()),
            ..Default::default()
        }
    }

    #[test]
    fn recording_date_accepts_common_formats() {
        let parse = |d| recording_date(&item_with_date(d));
        assert_eq!(
            parse("Thu, 02 Jan 2025 10:00:00 +0000"),
            "2025-01-02T10:00:00+00:00"
        );
        assert_eq!(
            parse("2025-01-02T10:00:00+01:00"),
            "2025-01-02T09:00:00+00:00"
        );
        assert_eq!(
            parse("Thurs, 02 Jan 2025 10:00:00 +0000"),
            "2025-01-02T10:00:00+00:00"
        );
        assert_eq!(recording_date(&Item::default()), "");
    }

    #[test]
    fn recording_date_never_panics_on_garbage() {
        // Used to slice `&date[5..]`, which panicked on short or non-ASCII input.
        for garbage in ["", "abc", "äöüäöü", "Mo., 1 Jän 2025"] {
            assert_eq!(
                recording_date(&item_with_date(garbage)),
                "2021-01-01T00:00:00+00:00"
            );
        }
    }

    use rss::extension::{Extension, ExtensionBuilder};
    use std::collections::BTreeMap;

    fn transcript_extension(
        url: Option<&str>,
        mime_type: Option<&str>,
        language: Option<&str>,
    ) -> Extension {
        let mut builder = ExtensionBuilder::default();
        builder.name("podcast:transcript");

        let mut attrs = BTreeMap::new();
        if let Some(url) = url {
            attrs.insert("url".to_string(), url.to_string());
        }
        if let Some(mime_type) = mime_type {
            attrs.insert("type".to_string(), mime_type.to_string());
        }
        if let Some(language) = language {
            attrs.insert("language".to_string(), language.to_string());
        }
        builder.attrs(attrs);

        builder.build()
    }

    fn item_with_transcripts(transcripts: Vec<Extension>) -> Item {
        let mut podcast_ns = BTreeMap::new();
        podcast_ns.insert("transcript".to_string(), transcripts);

        let mut extensions = rss::extension::ExtensionMap::new();
        extensions.insert("podcast".to_string(), podcast_ns);

        Item {
            extensions,
            ..Default::default()
        }
    }

    #[test]
    fn extract_transcript_tags_reads_url_type_and_language() {
        let item = item_with_transcripts(vec![
            transcript_extension(
                Some("https://example.com/ep1.vtt"),
                Some("text/vtt"),
                Some("en"),
            ),
            transcript_extension(
                Some("https://example.com/ep1.json"),
                Some("application/json"),
                Some("en"),
            ),
        ]);

        let tags = transcripts(&item);

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
        assert_eq!(tags[0].mime_type, "text/vtt");
        assert_eq!(tags[0].language.as_deref(), Some("en"));
        assert_eq!(tags[1].url, "https://example.com/ep1.json");
        assert_eq!(tags[1].mime_type, "application/json");
        assert_eq!(tags[1].language.as_deref(), Some("en"));
    }

    #[test]
    fn extract_transcript_tags_defaults_missing_type_to_text_plain() {
        let item = item_with_transcripts(vec![transcript_extension(
            Some("https://example.com/ep1.txt"),
            None,
            None,
        )]);

        let tags = transcripts(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].mime_type, "text/plain");
        assert_eq!(tags[0].language, None);
    }

    #[test]
    fn extract_transcript_tags_skips_tag_without_url() {
        let item = item_with_transcripts(vec![
            transcript_extension(None, Some("text/vtt"), Some("en")),
            transcript_extension(
                Some("https://example.com/ep1.vtt"),
                Some("text/vtt"),
                Some("en"),
            ),
        ]);

        let tags = transcripts(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
    }

    #[test]
    fn extract_transcript_tags_checks_qualified_name_key_too() {
        let mut podcast_ns = BTreeMap::new();
        podcast_ns.insert(
            "podcast:transcript".to_string(),
            vec![transcript_extension(
                Some("https://example.com/ep1.vtt"),
                Some("text/vtt"),
                None,
            )],
        );
        let mut extensions = rss::extension::ExtensionMap::new();
        extensions.insert("podcast".to_string(), podcast_ns);
        let item = Item {
            extensions,
            ..Default::default()
        };

        let tags = transcripts(&item);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].url, "https://example.com/ep1.vtt");
    }

    #[test]
    fn extract_transcript_tags_returns_empty_when_no_podcast_namespace() {
        let item = Item::default();

        let tags = transcripts(&item);

        assert!(tags.is_empty());
    }

    fn channel(xml_channel_body: &str) -> rss::Channel {
        let xml = format!(
            r#"<?xml version="1.0"?><rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd"><channel><title>t</title><link>https://x</link><description>d</description>{xml_channel_body}</channel></rss>"#
        );
        rss::Channel::read_from(xml.as_bytes()).unwrap()
    }

    #[test]
    fn prefers_rss_image() {
        let c = channel(
            r#"<image><url>https://x/rss.jpg</url><title>t</title><link>https://x</link></image><itunes:image href="https://x/itunes.jpg"/>"#,
        );
        assert_eq!(channel_image_url(&c).as_deref(), Some("https://x/rss.jpg"));
    }

    #[test]
    fn falls_back_to_itunes_image() {
        let c = channel(r#"<itunes:image href="https://x/itunes.jpg"/>"#);
        assert_eq!(
            channel_image_url(&c).as_deref(),
            Some("https://x/itunes.jpg")
        );
    }

    #[test]
    fn none_without_any_image() {
        assert_eq!(channel_image_url(&channel("")), None);
    }

    #[test]
    fn durations_in_all_forms() {
        assert_eq!(duration("95"), 95);
        assert_eq!(duration("1:35"), 95);
        assert_eq!(duration("1:00:05"), 3605);
        assert_eq!(duration("1:00:00:01"), 86401);
        assert_eq!(duration(""), 0);
        assert_eq!(duration("1:2:3:4:5"), 0);
        assert_eq!(duration("x:30"), 30);
    }

    #[test]
    fn metadata_needs_the_itunes_extension() {
        let id = Uuid::nil();
        assert!(metadata(&channel(""), id).is_none());

        let update = metadata(
            &channel(
                r#"<language>de</language><itunes:author>Author</itunes:author><itunes:category text="Tech"/><itunes:category text="News"/>"#,
            ),
            id,
        )
        .unwrap();
        assert_eq!(update.author.as_deref(), Some("Author"));
        assert_eq!(update.keywords.as_deref(), Some("Tech,News"));
        assert_eq!(update.language.as_deref(), Some("de"));
        assert_eq!(update.description.as_deref(), Some("d"));
        assert_eq!(update.explicit.as_deref(), Some("false"));
        assert_eq!(update.last_build_date, None);
    }
}
