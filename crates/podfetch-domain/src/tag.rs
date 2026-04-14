use chrono::{NaiveDateTime, Utc};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Color {
    Red,
    Green,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Red => write!(f, "Red"),
            Color::Green => write!(f, "Green"),
            Color::Blue => write!(f, "Blue"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub user_id: i32,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub color: String,
}

impl Tag {
    pub fn new(name: String, description: Option<String>, color: String, user_id: i32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            user_id,
            description,
            created_at: Utc::now().naive_utc(),
            color,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagsPodcast {
    pub tag_id: String,
    pub podcast_id: i32,
}

#[derive(Debug, Clone)]
pub struct TagUpdate {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
}

pub trait TagRepository: Send + Sync {
    type Error;

    fn create(&self, tag: Tag) -> Result<Tag, Self::Error>;
    fn get_tags(&self, user_id: i32) -> Result<Vec<Tag>, Self::Error>;
    fn get_tags_of_podcast(&self, podcast_id: i32, user_id: i32)
    -> Result<Vec<Tag>, Self::Error>;
    fn get_tag_by_id_and_user_id(
        &self,
        tag_id: &str,
        user_id: i32,
    ) -> Result<Option<Tag>, Self::Error>;
    fn update(&self, tag_id: &str, update: TagUpdate) -> Result<Tag, Self::Error>;
    fn delete(&self, tag_id: &str) -> Result<(), Self::Error>;
    fn add_podcast_to_tag(
        &self,
        tag_id_to_insert: String,
        podcast_id_to_insert: i32,
    ) -> Result<TagsPodcast, Self::Error>;
    fn delete_tag_podcasts(&self, tag_id: &str) -> Result<(), Self::Error>;
    fn delete_tag_podcasts_by_podcast_id_tag_id(
        &self,
        podcast_id: i32,
        tag_id: &str,
    ) -> Result<(), Self::Error>;
    fn delete_tag_podcasts_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error>;
}
