use chrono::NaiveDateTime;
use utoipa::ToSchema;
use crate::domain::models::tag::tag::Tag;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TagDto {
    pub(crate) id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub color: String,
}

impl From<Tag> for TagDto {
    fn from(value: Tag) -> Self {
        Self {
            id: value.id,
            name: value.name,
            username: value.username,
            description: value.description,
            created_at: value.created_at,
            color: value.color,
        }
    }
}

impl Into<Tag> for TagDto {
    fn into(self) -> Tag {
        Tag {
            id: self.id,
            name: self.name,
            username: self.username,
            description: self.description,
            created_at: self.created_at,
            color: self.color,
        }
    }
}