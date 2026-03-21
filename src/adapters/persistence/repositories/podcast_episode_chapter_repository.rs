use crate::utils::error::CustomError;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapter, PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};
use podfetch_persistence::db::Database;
use podfetch_persistence::podcast_episode_chapter::DieselPodcastEpisodeChapterRepository;

pub struct PodcastEpisodeChapterRepositoryImpl {
    inner: DieselPodcastEpisodeChapterRepository,
}

impl PodcastEpisodeChapterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeChapterRepository::new(database),
        }
    }
}

impl PodcastEpisodeChapterRepository for PodcastEpisodeChapterRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        self.inner.upsert(chapter).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: i32,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }
}
