//! `GET /api/search/podcast?term=<term>&country=<cc>` — audiobookshelf-compat
//! iTunes-backed podcast search. Mobile apps consume this to populate the
//! "Add Podcast" search results.
//!
//! Maps PodFetch's existing `PodcastService::find_podcast()` (which already
//! calls iTunes Search) onto the byte-shape upstream returns from
//! `server/providers/iTunes.js::cleanPodcast`.

use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::podcast::ItunesModel;
use crate::services::podcast::service::PodcastService;
use axum::Json;
use axum::extract::{Query, State};
use common_infrastructure::error::CustomError;
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, utoipa::IntoParams)]
pub struct PodcastSearchQuery {
    pub term: Option<String>,
    #[allow(dead_code)]
    pub country: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/search/podcast",
    params(PodcastSearchQuery),
    responses((status = 200, description = "iTunes-backed podcast search results")),
    tag = "audiobookshelf"
)]
pub async fn search_podcast(
    State(_state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Query(query): Query<PodcastSearchQuery>,
) -> Result<Json<Vec<Value>>, CustomError> {
    let term = query.term.as_deref().unwrap_or("").trim();
    if term.is_empty() {
        return Ok(Json(Vec::new()));
    }
    let wrapper = PodcastService::find_podcast(term).await;
    let results: Vec<Value> = wrapper
        .results()
        .iter()
        .filter(|r| r.feed_url.as_deref().is_some_and(|s| !s.is_empty()))
        .map(map_itunes_result)
        .collect();
    Ok(Json(results))
}

/// Mirrors upstream `iTunes.cleanPodcast()`. Field names are camelCase
/// because the mobile-app Jackson deserialiser is configured with the
/// default name strategy and pulls fields by exact key.
fn map_itunes_result(r: &ItunesModel) -> Value {
    let cover = r
        .artwork_url600
        .clone()
        .or_else(|| r.artwork_url_100.clone())
        .or_else(|| r.artwork_url60.clone())
        .or_else(|| r.artwork_url30.clone());
    let explicit = r
        .track_explicitness
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("explicit"))
        .unwrap_or(false);
    json!({
        "id": r.collection_id,
        "artistId": r.artist_id,
        "title": r.collection_name.clone().unwrap_or_default(),
        "artistName": r.artist_name.clone().unwrap_or_default(),
        "description": r.description.clone().unwrap_or_default(),
        "descriptionPlain": r.description.clone().unwrap_or_default(),
        "releaseDate": r.release_date.clone().unwrap_or_default(),
        "genres": r.genres,
        "cover": cover.unwrap_or_default(),
        "trackCount": r.track_count.unwrap_or(0),
        "feedUrl": r.feed_url.clone().unwrap_or_default(),
        "pageUrl": r.collection_view_url.clone().unwrap_or_default(),
        "explicit": explicit,
    })
}

pub fn get_search_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(search_podcast))
}

#[cfg(test)]
pub fn map_itunes_result_for_test(r: &ItunesModel) -> Value {
    map_itunes_result(r)
}
