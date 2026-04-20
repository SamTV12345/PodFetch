use crate::app_state::AppState;
use crate::podcast::{Feed, PodindexResponse};
use crate::services::discover::itunes_charts::ItunesChartEntry;
use crate::services::discover::service::{CategoryDto, DiscoverService};
use axum::extract::Query;
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use podfetch_domain::user::User;
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TrendingQuery {
    /// Comma-separated list of Podcastindex category IDs to narrow the
    /// trending list. Omit to get globally trending podcasts.
    pub cat: Option<String>,
    /// Optional ISO language code ("de", "en"). Filter trending by language.
    pub lang: Option<String>,
    /// How many results to return, capped at 50.
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ChartsQuery {
    /// ISO-3166-1 alpha-2 country code, e.g. "de", "us".
    pub country: String,
    /// iTunes genre ID. Omit for the overall country top-chart.
    pub genre: Option<u32>,
    /// How many results to return (capped at 200 by iTunes).
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct ForYouQuery {
    pub limit: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/discover/trending",
    params(TrendingQuery),
    responses((status = 200, description = "Globally or filtered trending podcasts", body = PodindexResponse)),
    tag = "discover"
)]
pub async fn get_trending(
    Query(query): Query<TrendingQuery>,
) -> Result<Json<PodindexResponse>, CustomError> {
    let limit = query.limit.unwrap_or(50);
    let cats = parse_category_ids(query.cat.as_deref());
    let lang = query.lang.as_deref().filter(|s| !s.is_empty());
    let response = DiscoverService::trending(&cats, lang, limit).await?;
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/discover/charts",
    params(ChartsQuery),
    responses((status = 200, description = "iTunes country top-podcasts chart", body = Vec<ItunesChartEntry>)),
    tag = "discover"
)]
pub async fn get_charts(
    Query(query): Query<ChartsQuery>,
) -> Result<Json<Vec<ItunesChartEntry>>, CustomError> {
    let limit = query.limit.unwrap_or(100);
    let entries = DiscoverService::charts(&query.country, query.genre, limit).await?;
    Ok(Json(entries))
}

#[utoipa::path(
    get,
    path = "/discover/categories",
    responses((status = 200, description = "Known podcast categories and their iTunes/Podcastindex IDs", body = Vec<CategoryDto>)),
    tag = "discover"
)]
pub async fn get_categories() -> Json<Vec<CategoryDto>> {
    Json(DiscoverService::categories())
}

#[utoipa::path(
    get,
    path = "/discover/for-you",
    params(ForYouQuery),
    responses((status = 200, description = "Trending feeds matching the user's subscribed categories", body = Vec<Feed>)),
    tag = "discover"
)]
pub async fn get_for_you(
    Extension(requester): Extension<User>,
    Query(query): Query<ForYouQuery>,
) -> Result<Json<Vec<Feed>>, CustomError> {
    let limit = query.limit.unwrap_or(50);
    let feeds = DiscoverService::for_you(&requester, limit).await?;
    Ok(Json(feeds))
}

fn parse_category_ids(input: Option<&str>) -> Vec<u32> {
    input
        .map(|s| {
            s.split(',')
                .filter_map(|part| part.trim().parse::<u32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

pub fn get_discover_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_trending))
        .routes(routes!(get_charts))
        .routes(routes!(get_categories))
        .routes(routes!(get_for_you))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_separated_category_ids() {
        assert_eq!(parse_category_ids(Some("55,9, 102")), vec![55, 9, 102]);
        assert_eq!(parse_category_ids(Some("not,a,number")), Vec::<u32>::new());
        assert_eq!(parse_category_ids(None), Vec::<u32>::new());
    }
}
