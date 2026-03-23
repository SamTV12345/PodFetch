use podfetch_domain::episode::Episode;
use podfetch_web::history::{EpisodeAction, EpisodeDto};
use reqwest::Url;

pub fn map_episode_to_dto(episode: &Episode) -> EpisodeDto {
    EpisodeDto {
        podcast: episode.podcast.clone(),
        episode: episode.episode.clone(),
        timestamp: episode.timestamp,
        guid: episode.guid.clone(),
        action: EpisodeAction::from_string(&episode.action),
        started: episode.started,
        position: episode.position,
        total: episode.total,
        device: episode.device.clone(),
    }
}

pub fn map_episode_dto_to_episode(episode_dto: &EpisodeDto, username: String) -> Episode {
    // Remove query parameters
    let mut episode = Url::parse(&episode_dto.episode).unwrap();
    episode.set_query(None);

    Episode {
        id: 0,
        username,
        device: episode_dto.device.clone(),
        podcast: episode_dto.podcast.clone(),
        episode: episode_dto.episode.clone(),
        timestamp: episode_dto.timestamp,
        guid: episode_dto.guid.clone(),
        action: episode_dto.action.clone().to_string(),
        started: episode_dto.started,
        position: episode_dto.position,
        total: episode_dto.total,
    }
}
