use common_infrastructure::error::CustomError;
use podfetch_domain::tag::{Tag, TagRepository, TagUpdate, TagsPodcast};
use podfetch_persistence::db::Database;
use podfetch_persistence::tag::DieselTagRepository;

pub struct TagRepositoryImpl {
    inner: DieselTagRepository,
}

impl TagRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselTagRepository::new(database),
        }
    }
}

impl TagRepository for TagRepositoryImpl {
    type Error = CustomError;

    fn create(&self, tag: Tag) -> Result<Tag, Self::Error> {
        self.inner.create(tag).map_err(Into::into)
    }

    fn get_tags(&self, username: &str) -> Result<Vec<Tag>, Self::Error> {
        self.inner.get_tags(username).map_err(Into::into)
    }

    fn get_tags_of_podcast(
        &self,
        podcast_id: i32,
        username: &str,
    ) -> Result<Vec<Tag>, Self::Error> {
        self.inner
            .get_tags_of_podcast(podcast_id, username)
            .map_err(Into::into)
    }

    fn get_tag_by_id_and_username(
        &self,
        tag_id: &str,
        username: &str,
    ) -> Result<Option<Tag>, Self::Error> {
        self.inner
            .get_tag_by_id_and_username(tag_id, username)
            .map_err(Into::into)
    }

    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, Self::Error> {
        self.inner.update(tag_id, update).map_err(Into::into)
    }

    fn delete(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete(tag_id).map_err(Into::into)
    }

    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, Self::Error> {
        self.inner
            .add_podcast_to_tag(tag_id_to_insert, podcast_id_to_insert)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), Self::Error> {
        self.inner.delete_tag_podcasts(tag_id).map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id_tag_id(podcast_id, tag_id)
            .map_err(Into::into)
    }

    fn delete_tag_podcasts_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error> {
        self.inner
            .delete_tag_podcasts_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }
}

