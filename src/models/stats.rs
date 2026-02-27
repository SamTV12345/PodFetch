use crate::models::listening_event::ListeningEvent;
use crate::models::podcasts::Podcast;
use crate::utils::error::CustomError;
use chrono::Datelike;
use chrono::NaiveDateTime;
use std::cmp::Reverse;
use std::collections::HashMap;
use utoipa::ToSchema;

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

impl StatsOverview {
    pub fn calculate_for_user(
        username: &str,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
        top_limit: usize,
    ) -> Result<Self, CustomError> {
        let events = ListeningEvent::get_by_user_and_range(username, from, to)?;
        let total_listened_seconds = events
            .iter()
            .map(|event| i64::from(event.delta_seconds))
            .sum::<i64>();

        let mut podcast_aggregation: HashMap<i32, (i64, std::collections::HashSet<i32>)> =
            HashMap::new();
        let mut weekday_seconds = [0i64; 7];

        for event in events {
            let entry = podcast_aggregation
                .entry(event.podcast_id)
                .or_insert((0, std::collections::HashSet::new()));
            entry.0 += i64::from(event.delta_seconds);
            entry.1.insert(event.podcast_episode_db_id);

            let weekday_index = event.listened_at.weekday().num_days_from_monday() as usize;
            weekday_seconds[weekday_index] += i64::from(event.delta_seconds);
        }

        let listened_podcasts = podcast_aggregation.len() as i64;
        let listened_episodes = podcast_aggregation
            .values()
            .map(|(_, episodes)| episodes.len() as i64)
            .sum::<i64>();

        let podcasts = Podcast::get_all_podcasts()?;
        let podcast_index = podcasts
            .into_iter()
            .map(|podcast| (podcast.id, podcast))
            .collect::<HashMap<_, _>>();

        let mut top_podcasts = podcast_aggregation
            .into_iter()
            .map(|(podcast_id, (seconds, episodes))| {
                let podcast = podcast_index.get(&podcast_id);
                TopPodcastStats {
                    podcast_id,
                    podcast_name: podcast
                        .map(|podcast| podcast.name.clone())
                        .unwrap_or_else(|| "Unknown Podcast".to_string()),
                    image_url: podcast
                        .map(|podcast| podcast.image_url.clone())
                        .unwrap_or_default(),
                    listened_seconds: seconds,
                    listened_episodes: episodes.len() as i64,
                }
            })
            .collect::<Vec<_>>();

        top_podcasts.sort_by_key(|podcast| Reverse(podcast.listened_seconds));
        top_podcasts.truncate(top_limit);

        let active_weekdays = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ]
        .iter()
        .enumerate()
        .map(|(idx, weekday)| WeekdayStats {
            day_index: (idx + 1) as i32,
            weekday: (*weekday).to_string(),
            listened_seconds: weekday_seconds[idx],
        })
        .collect::<Vec<_>>();

        Ok(Self {
            from,
            to,
            listened_podcasts,
            listened_episodes,
            total_listened_seconds,
            top_podcasts,
            active_weekdays,
        })
    }
}
