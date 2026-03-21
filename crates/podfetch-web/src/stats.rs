use chrono::{DateTime, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WeekdayStats {
    pub day_index: i32,
    pub weekday: String,
    pub listened_seconds: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopPodcastStats {
    pub podcast_id: i32,
    pub podcast_name: String,
    pub image_url: String,
    pub listened_seconds: i64,
    pub listened_episodes: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatsOverview {
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
    pub listened_podcasts: i64,
    pub listened_episodes: i64,
    pub total_listened_seconds: i64,
    pub top_podcasts: Vec<TopPodcastStats>,
    pub active_weekdays: Vec<WeekdayStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct StatsOverviewQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub top_limit: Option<usize>,
}

pub trait StatsApplicationService {
    type Error;
    type StatsOverview;

    fn get_stats_overview(
        &self,
        username: &str,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        top_limit: usize,
    ) -> Result<Self::StatsOverview, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum StatsControllerError<E: Display> {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0}")]
    Service(E),
}

pub fn get_stats_overview<S>(
    service: &S,
    username: &str,
    params: StatsOverviewQueryParams,
) -> Result<S::StatsOverview, StatsControllerError<S::Error>>
where
    S: StatsApplicationService,
    S::Error: Display,
{
    let from = params
        .from
        .as_deref()
        .map(|from| parse_datetime(from, false))
        .transpose()
        .map_err(|error| match error {
            StatsControllerError::BadRequest(message) => StatsControllerError::BadRequest(message),
            StatsControllerError::Service(impossible) => match impossible {},
        })?;
    let to = params
        .to
        .as_deref()
        .map(|to| parse_datetime(to, true))
        .transpose()
        .map_err(|error| match error {
            StatsControllerError::BadRequest(message) => StatsControllerError::BadRequest(message),
            StatsControllerError::Service(impossible) => match impossible {},
        })?;
    if let (Some(from), Some(to)) = (from, to)
        && from > to
    {
        return Err(StatsControllerError::BadRequest(
            "'from' must be less than or equal to 'to'".to_string(),
        ));
    }

    let top_limit = params.top_limit.unwrap_or(5).clamp(1, 20);
    service
        .get_stats_overview(username, from, to, top_limit)
        .map_err(StatsControllerError::Service)
}

fn parse_datetime(
    input: &str,
    end_of_day: bool,
) -> Result<NaiveDateTime, StatsControllerError<std::convert::Infallible>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(input) {
        return Ok(parsed.naive_utc());
    }
    if let Ok(parsed_date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return if end_of_day {
            Ok(parsed_date.and_hms_opt(23, 59, 59).unwrap())
        } else {
            Ok(parsed_date.and_hms_opt(0, 0, 0).unwrap())
        };
    }

    Err(StatsControllerError::BadRequest(format!(
        "Invalid datetime format: {input}. Use RFC3339 or YYYY-MM-DD."
    )))
}
