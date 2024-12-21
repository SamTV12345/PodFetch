use chrono::NaiveDateTime;
use utoipa::ToSchema;
use crate::adapters::api::models::podcast::podcast_dto::PodcastDto;
use crate::adapters::api::models::podcast_episode::podcast_episode::PodcastEpisodeDto;
use crate::adapters::persistence::model::podcast::episode::EpisodeEntity;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i32,
    pub date: NaiveDateTime,
    pub total_time: i32,
    pub podcast_episode: PodcastEpisodeDto,
    pub podcast: PodcastDto,
}

impl From<(PodcastEpisode, EpisodeEntity, Podcast)> for
PodcastWatchedEpisodeModelWithPodcastEpisode {
    fn from(value: (PodcastEpisode, EpisodeEntity, Podcast)) -> Self {
        PodcastWatchedEpisodeModelWithPodcastEpisode {
            id: value.clone().1.id,
            podcast_id: value.clone().2.id,
            episode_id: value.0.episode_id.clone(),
            url: value.0.url.clone(),
            name: value.0.name.clone(),
            image_url: value.0.image_url.clone(),
            watched_time: value.clone().1.position.unwrap(),
            date: value.clone().1.timestamp,
            total_time: value.clone().0.total_time,
            podcast_episode: value.0.into(),
            podcast: value.2.into(),
        }
    }
}