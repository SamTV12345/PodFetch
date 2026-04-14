use crate::tags::{Tag, TagCreate, TagsApplicationService, TagsPodcast};
use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::tag::{TagRepository, TagUpdate};
use podfetch_persistence::adapters::TagRepositoryImpl;
use podfetch_persistence::db::database;
use std::sync::Arc;

pub struct TagService {
    repository: Arc<dyn TagRepository<Error = CustomError>>,
}

impl TagService {
    pub fn new(repository: Arc<dyn TagRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(TagRepositoryImpl::new(database())))
    }

    pub fn create_tag(
        &self,
        user_id: i32,
        name: String,
        description: Option<String>,
        color: String,
    ) -> Result<Tag, CustomError> {
        let tag = podfetch_domain::tag::Tag::new(name, description, color, user_id);
        self.repository.create(tag).map(Into::into)
    }

    pub fn get_tags(&self, user_id: i32) -> Result<Vec<Tag>, CustomError> {
        self.repository
            .get_tags(user_id)
            .map(|tags| tags.into_iter().map(Into::into).collect())
    }

    pub fn get_tags_of_podcast(
        &self,
        podcast_id: i32,
        user_id: i32,
    ) -> Result<Vec<Tag>, CustomError> {
        self.repository
            .get_tags_of_podcast(podcast_id, user_id)
            .map(|tags| tags.into_iter().map(Into::into).collect())
    }

    pub fn update_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        update: TagUpdate,
    ) -> Result<Tag, CustomError> {
        self.get_tag_for_user(tag_id, user_id)?;
        self.repository.update(tag_id, update).map(Into::into)
    }

    pub fn delete_tag(&self, user_id: i32, tag_id: &str) -> Result<(), CustomError> {
        self.get_tag_for_user(tag_id, user_id)?;
        self.repository.delete_tag_podcasts(tag_id)?;
        self.repository.delete(tag_id)
    }

    pub fn add_podcast_to_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<TagsPodcast, CustomError> {
        self.get_tag_for_user(tag_id, user_id)?;
        self.repository
            .add_podcast_to_tag(tag_id.to_string(), podcast_id)
            .map(Into::into)
    }

    pub fn delete_podcast_from_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<(), CustomError> {
        self.get_tag_for_user(tag_id, user_id)?;
        self.repository
            .delete_tag_podcasts_by_podcast_id_tag_id(podcast_id, tag_id)
    }

    pub fn delete_podcast_tags(&self, podcast_id: i32) -> Result<(), CustomError> {
        self.repository
            .delete_tag_podcasts_by_podcast_id(podcast_id)
    }

    fn get_tag_for_user(
        &self,
        tag_id: &str,
        user_id: i32,
    ) -> Result<podfetch_domain::tag::Tag, CustomError> {
        self.repository
            .get_tag_by_id_and_user_id(tag_id, user_id)?
            .ok_or_else(|| CustomErrorInner::NotFound(Debug).into())
    }
}

impl TagsApplicationService for TagService {
    type Error = CustomError;

    fn create_tag(
        &self,
        user_id: i32,
        name: String,
        description: Option<String>,
        color: String,
    ) -> Result<Tag, Self::Error> {
        self.create_tag(user_id, name, description, color)
    }

    fn get_tags(&self, user_id: i32) -> Result<Vec<Tag>, Self::Error> {
        self.get_tags(user_id)
    }

    fn update_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        update: TagCreate,
    ) -> Result<Tag, Self::Error> {
        self.update_tag(
            user_id,
            tag_id,
            TagUpdate {
                name: update.name,
                description: update.description,
                color: update.color.to_string(),
            },
        )
    }

    fn delete_tag(&self, user_id: i32, tag_id: &str) -> Result<(), Self::Error> {
        self.delete_tag(user_id, tag_id)
    }

    fn add_podcast_to_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<TagsPodcast, Self::Error> {
        self.add_podcast_to_tag(user_id, tag_id, podcast_id)
    }

    fn delete_podcast_from_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<(), Self::Error> {
        self.delete_podcast_from_tag(user_id, tag_id, podcast_id)
    }
}
