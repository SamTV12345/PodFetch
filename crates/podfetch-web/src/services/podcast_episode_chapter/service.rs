use crate::settings::UpsertPodcastEpisodeChapter;
use common_infrastructure::error::CustomError;
use podfetch_catalog::domain::chapter::PodcastEpisodeChapterRepository;
use podfetch_catalog::persistence::chapter::PodcastEpisodeChapterRepositoryImpl;
use podfetch_media::audio::Chapter;
use podfetch_persistence::db::database;
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
        episode_id: &str,
    ) -> Result<(), CustomError> {
        self.repository.upsert(
            UpsertPodcastEpisodeChapter {
                episode_id: uuid::Uuid::parse_str(episode_id).map_err(|_| {
                    CustomError::from(common_infrastructure::error::CustomErrorInner::BadRequest(
                        "invalid episode id".to_string(),
                        common_infrastructure::error::ErrorSeverity::Warning,
                    ))
                })?,
                title: chapter_to_save.title.clone().unwrap_or_default(),
                start_time: chapter_to_save.start.num_seconds() as i32,
                end_time: chapter_to_save.end.unwrap_or_default().num_seconds() as i32,
                href: chapter_to_save
                    .link
                    .clone()
                    .map(|link| link.url.to_string()),
                image: chapter_to_save.image.clone().map(|img| img.to_string()),
            }
            .into(),
        )
    }

    pub fn get_chapters_by_episode_id(
        &self,
        episode_id: uuid::Uuid,
    ) -> Result<Vec<podfetch_catalog::domain::chapter::PodcastEpisodeChapter>, CustomError> {
        self.repository.get_by_episode_id(episode_id)
    }
}
