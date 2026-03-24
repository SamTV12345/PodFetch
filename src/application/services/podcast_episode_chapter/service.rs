use crate::adapters::persistence::dbconfig::db::database;
use crate::adapters::persistence::repositories::podcast_episode_chapter_repository::PodcastEpisodeChapterRepositoryImpl;
use crate::application::services::download::chapter::Chapter;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use common_infrastructure::error::CustomError;
use podfetch_domain::podcast_episode_chapter::PodcastEpisodeChapterRepository;
use podfetch_web::settings::UpsertPodcastEpisodeChapter;
use std::sync::Arc;

#[derive(Clone)]
pub struct PodcastEpisodeChapterService {
    repository: Arc<dyn PodcastEpisodeChapterRepository<Error = CustomError>>,
}

impl PodcastEpisodeChapterService {
    pub fn new(repository: Arc<dyn PodcastEpisodeChapterRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(PodcastEpisodeChapterRepositoryImpl::new(
            database(),
        )))
    }

    pub fn save_chapter(
        &self,
        chapter_to_save: &Chapter,
        podcast_episode: &PodcastEpisode,
    ) -> Result<(), CustomError> {
        self.repository.upsert(UpsertPodcastEpisodeChapter {
            episode_id: podcast_episode.id,
            title: chapter_to_save.title.clone().unwrap_or_default(),
            start_time: chapter_to_save.start.num_seconds() as i32,
            end_time: chapter_to_save.end.unwrap_or_default().num_seconds() as i32,
            href: chapter_to_save
                .link
                .clone()
                .map(|link| link.url.to_string()),
            image: chapter_to_save.image.clone().map(|img| img.to_string()),
        }
        .into())
    }

    pub fn get_chapters_by_episode_id(
        &self,
        episode_id: i32,
    ) -> Result<Vec<podfetch_domain::podcast_episode_chapter::PodcastEpisodeChapter>, CustomError> {
        self.repository.get_by_episode_id(episode_id)
    }
}


