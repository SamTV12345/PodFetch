use podfetch_domain::tag::{Color, Tag, TagsPodcast};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TagCreate {
    pub name: String,
    pub description: Option<String>,
    pub color: Color,
}

pub trait TagsApplicationService {
    type Error;

    fn create_tag(
        &self,
        username: String,
        name: String,
        description: Option<String>,
        color: String,
    ) -> Result<Tag, Self::Error>;
    fn get_tags(&self, username: &str) -> Result<Vec<Tag>, Self::Error>;
    fn update_tag(
        &self,
        username: &str,
        tag_id: &str,
        update: TagCreate,
    ) -> Result<Tag, Self::Error>;
    fn delete_tag(&self, username: &str, tag_id: &str) -> Result<(), Self::Error>;
    fn add_podcast_to_tag(
        &self,
        username: &str,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<TagsPodcast, Self::Error>;
    fn delete_podcast_from_tag(
        &self,
        username: &str,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<(), Self::Error>;
}

pub fn create_tag<S>(service: &S, username: String, payload: TagCreate) -> Result<Tag, S::Error>
where
    S: TagsApplicationService,
{
    service.create_tag(
        username,
        payload.name,
        payload.description,
        payload.color.to_string(),
    )
}

pub fn get_tags<S>(service: &S, username: &str) -> Result<Vec<Tag>, S::Error>
where
    S: TagsApplicationService,
{
    service.get_tags(username)
}

pub fn update_tag<S>(
    service: &S,
    username: &str,
    tag_id: &str,
    payload: TagCreate,
) -> Result<Tag, S::Error>
where
    S: TagsApplicationService,
{
    service.update_tag(username, tag_id, payload)
}

pub fn delete_tag<S>(service: &S, username: &str, tag_id: &str) -> Result<(), S::Error>
where
    S: TagsApplicationService,
{
    service.delete_tag(username, tag_id)
}

pub fn add_podcast_to_tag<S>(
    service: &S,
    username: &str,
    tag_id: &str,
    podcast_id: i32,
) -> Result<TagsPodcast, S::Error>
where
    S: TagsApplicationService,
{
    service.add_podcast_to_tag(username, tag_id, podcast_id)
}

pub fn delete_podcast_from_tag<S>(
    service: &S,
    username: &str,
    tag_id: &str,
    podcast_id: i32,
) -> Result<(), S::Error>
where
    S: TagsApplicationService,
{
    service.delete_podcast_from_tag(username, tag_id, podcast_id)
}
