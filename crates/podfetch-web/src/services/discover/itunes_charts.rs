//! Thin wrapper around Apple's free country-specific podcast charts RSS feed.
//!
//! URL pattern: `https://itunes.apple.com/{country}/rss/toppodcasts/limit={N}/genre={id}/json`.
//! `country` is an ISO-3166-1 alpha-2 lowercase code (de, us, gb, ...). Genre
//! can be omitted to get the overall top chart. No authentication required.

use common_infrastructure::error::ErrorSeverity::Error as ErrorSev;
use common_infrastructure::error::{CustomError, CustomErrorInner, map_reqwest_error};
use common_infrastructure::http::get_http_client;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItunesChartEntry {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub image: Option<String>,
    pub feed_url: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<String>,
}

pub struct ItunesChartsService;

impl ItunesChartsService {
    pub async fn top_podcasts(
        country: &str,
        genre_id: Option<u32>,
        limit: u32,
    ) -> Result<Vec<ItunesChartEntry>, CustomError> {
        let limit = limit.clamp(1, 200);
        let country = sanitize_country(country);
        let url = match genre_id {
            Some(id) => format!(
                "https://itunes.apple.com/{country}/rss/toppodcasts/limit={limit}/genre={id}/json"
            ),
            None => {
                format!("https://itunes.apple.com/{country}/rss/toppodcasts/limit={limit}/json")
            }
        };

        let result = get_http_client(&ENVIRONMENT_SERVICE)
            .get(&url)
            .send()
            .await
            .map_err(map_reqwest_error)?;
        let status = result.status();
        let body = result.text().await.map_err(map_reqwest_error)?;

        if !status.is_success() {
            log::error!("iTunes charts error {status}: {body}");
            return Err(CustomErrorInner::BadRequest(body, ErrorSev).into());
        }

        let parsed: RawFeed = serde_json::from_str(&body).map_err(|e| {
            log::error!("Could not parse iTunes charts response: {e}");
            CustomError::from(CustomErrorInner::BadRequest(e.to_string(), ErrorSev))
        })?;

        // The iTunes chart lookup gives us a track id but not the RSS feed
        // URL. We have to resolve it via /lookup on-demand. Keep it simple:
        // do NOT resolve here (it would N+1 the requests); the frontend's
        // "Subscribe" button can call the existing iTunes lookup flow with
        // the collection id. We still expose the id so the UI can link out.
        Ok(parsed
            .feed
            .entry
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect())
    }
}

fn sanitize_country(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphabetic())
        .take(2)
        .collect::<String>()
        .to_ascii_lowercase()
}

#[derive(Debug, Deserialize)]
struct RawFeed {
    feed: RawFeedBody,
}

#[derive(Debug, Deserialize)]
struct RawFeedBody {
    entry: Option<Vec<RawEntry>>,
}

#[derive(Debug, Deserialize)]
struct RawEntry {
    id: RawIdField,
    #[serde(rename = "im:name")]
    name: RawLabelField,
    #[serde(rename = "im:artist")]
    artist: Option<RawLabelField>,
    #[serde(rename = "im:image")]
    image: Option<Vec<RawLabelField>>,
    #[serde(rename = "im:releaseDate")]
    release_date: Option<RawLabelField>,
    category: Option<RawCategory>,
}

#[derive(Debug, Deserialize)]
struct RawIdField {
    label: String,
    attributes: Option<RawIdAttributes>,
}

#[derive(Debug, Deserialize)]
struct RawIdAttributes {
    #[serde(rename = "im:id")]
    id: String,
}

#[derive(Debug, Deserialize)]
struct RawLabelField {
    label: String,
}

#[derive(Debug, Deserialize)]
struct RawCategory {
    attributes: Option<RawCategoryAttributes>,
}

#[derive(Debug, Deserialize)]
struct RawCategoryAttributes {
    label: String,
}

impl From<RawEntry> for ItunesChartEntry {
    fn from(value: RawEntry) -> Self {
        let id = value
            .id
            .attributes
            .map(|a| a.id)
            .unwrap_or_else(|| value.id.label.clone());
        let image = value
            .image
            .and_then(|imgs| imgs.into_iter().last().map(|l| l.label));
        ItunesChartEntry {
            id,
            name: value.name.label,
            artist: value.artist.map(|l| l.label),
            image,
            feed_url: None,
            genre: value.category.and_then(|c| c.attributes.map(|a| a.label)),
            release_date: value.release_date.map(|l| l.label),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_country_code() {
        assert_eq!(sanitize_country("DE"), "de");
        assert_eq!(sanitize_country("de-DE"), "de");
        assert_eq!(sanitize_country("  us"), "us");
        assert_eq!(sanitize_country("xxx"), "xx");
    }

    #[test]
    fn parses_minimal_feed() {
        let body = r#"{
            "feed": {
                "entry": [
                    {
                        "id": {
                            "label": "https://itunes.apple.com/de/podcast/x/id123",
                            "attributes": {"im:id": "123"}
                        },
                        "im:name": {"label": "Test"},
                        "im:artist": {"label": "Host"},
                        "im:image": [{"label": "small"}, {"label": "big"}],
                        "category": {"attributes": {"label": "News"}}
                    }
                ]
            }
        }"#;
        let parsed: RawFeed = serde_json::from_str(body).unwrap();
        let entries: Vec<ItunesChartEntry> = parsed
            .feed
            .entry
            .unwrap()
            .into_iter()
            .map(Into::into)
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "123");
        assert_eq!(entries[0].name, "Test");
        assert_eq!(entries[0].image.as_deref(), Some("big"));
        assert_eq!(entries[0].genre.as_deref(), Some("News"));
    }
}
