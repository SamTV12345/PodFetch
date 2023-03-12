use crate::constants::constants::{ITUNES_URL};
use reqwest::ClientBuilder as AsyncClientBuilder;
use serde_json::Value;
use crate::models::itunes_models::Podcast;
use crate::service::podcast_episode_service::PodcastEpisodeService;

pub async fn find_podcast(podcast: &str)-> Value {
    let client = AsyncClientBuilder::new().build().unwrap();
    let result = client.get(ITUNES_URL.to_owned()+podcast).send().await.unwrap();
    log::info!("Found podcast: {}", result.url());
    return result.json().await.unwrap();
}



pub fn schedule_episode_download(podcast: Podcast){
    let mut podcast_service = PodcastEpisodeService::new();
    let result = podcast_service.get_last_5_podcast_episodes(podcast.clone());
    for podcast_episode in result {
        podcast_service.download_podcast_episode_if_not_locally_available(podcast_episode,
                                                                                 podcast.clone());
    }
}

pub fn refresh_podcast(podcast:Podcast){
    log::info!("Refreshing podcast: {}", podcast.name);
    PodcastEpisodeService::insert_podcast_episodes(podcast.clone());
    schedule_episode_download(podcast);
}
