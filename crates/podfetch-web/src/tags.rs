use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "Red"),
            Color::Green => write!(f, "Green"),
            Color::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub user_id: i32,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub color: String,
}

impl From<podfetch_domain::tag::Tag> for Tag {
    fn from(value: podfetch_domain::tag::Tag) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
            description: value.description,
            created_at: value.created_at,
            color: value.color,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub struct TagsPodcast {
    pub tag_id: String,
    pub podcast_id: i32,
}

impl From<podfetch_domain::tag::TagsPodcast> for TagsPodcast {
    fn from(value: podfetch_domain::tag::TagsPodcast) -> Self {
        Self {
            tag_id: value.tag_id,
            podcast_id: value.podcast_id,
        }
    }
}

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
        user_id: i32,
        name: String,
        description: Option<String>,
        color: String,
    ) -> Result<Tag, Self::Error>;
    fn get_tags(&self, user_id: i32) -> Result<Vec<Tag>, Self::Error>;
    fn update_tag(&self, user_id: i32, tag_id: &str, update: TagCreate)
    -> Result<Tag, Self::Error>;
    fn delete_tag(&self, user_id: i32, tag_id: &str) -> Result<(), Self::Error>;
    fn add_podcast_to_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<TagsPodcast, Self::Error>;
    fn delete_podcast_from_tag(
        &self,
        user_id: i32,
        tag_id: &str,
        podcast_id: i32,
    ) -> Result<(), Self::Error>;
}

pub fn create_tag<S>(service: &S, user_id: i32, payload: TagCreate) -> Result<Tag, S::Error>
where
    S: TagsApplicationService,
{
    service.create_tag(
        user_id,
        payload.name,
        payload.description,
        payload.color.to_string(),
    )
}

pub fn get_tags<S>(service: &S, user_id: i32) -> Result<Vec<Tag>, S::Error>
where
    S: TagsApplicationService,
{
    service.get_tags(user_id)
}

pub fn update_tag<S>(
    service: &S,
    user_id: i32,
    tag_id: &str,
    payload: TagCreate,
) -> Result<Tag, S::Error>
where
    S: TagsApplicationService,
{
    service.update_tag(user_id, tag_id, payload)
}

pub fn delete_tag<S>(service: &S, user_id: i32, tag_id: &str) -> Result<(), S::Error>
where
    S: TagsApplicationService,
{
    service.delete_tag(user_id, tag_id)
}

pub fn add_podcast_to_tag<S>(
    service: &S,
    user_id: i32,
    tag_id: &str,
    podcast_id: i32,
) -> Result<TagsPodcast, S::Error>
where
    S: TagsApplicationService,
{
    service.add_podcast_to_tag(user_id, tag_id, podcast_id)
}

pub fn delete_podcast_from_tag<S>(
    service: &S,
    user_id: i32,
    tag_id: &str,
    podcast_id: i32,
) -> Result<(), S::Error>
where
    S: TagsApplicationService,
{
    service.delete_podcast_from_tag(user_id, tag_id, podcast_id)
}
