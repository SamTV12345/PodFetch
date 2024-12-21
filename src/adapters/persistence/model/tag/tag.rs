use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, QueryableByName};
use utoipa::ToSchema;
use crate::domain::models::tag::tag::Tag;

#[derive(
    Debug,
    QueryableByName,
    Queryable,
    AsChangeset,
    Insertable,
    Clone,
)]
#[diesel(treat_none_as_null = true)]
pub struct TagEntity {
    #[diesel(sql_type = Text)]
    pub(crate) id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub created_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    pub color: String,
}

impl From<Tag> for TagEntity {
    fn from(tag: Tag) -> Self {
        TagEntity {
            id: tag.id,
            name: tag.name,
            username: tag.username,
            description: tag.description,
            created_at: tag.created_at,
            color: tag.color,
        }
    }
}

impl Into<Tag> for TagEntity {
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


