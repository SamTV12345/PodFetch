use diesel::{Identifiable, Queryable, QueryableByName, Selectable};
use utoipa::ToSchema;
use diesel::sql_types::{Text, Integer, Nullable, Bool};
use crate::domain::models::podcast::podcast::Podcast;

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[table_name = "podcast"]
pub struct PodcastEntity {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub directory_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) rssfeed: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub summary: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub language: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub explicit: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub keywords: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_build_date: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub author: Option<String>,
    #[diesel(sql_type = Bool)]
    pub active: bool,
    #[diesel(sql_type = Text)]
    pub original_image_url: String,
    #[diesel(sql_type = Text)]
    pub directory_name: String,
}

impl From<Podcast> for PodcastEntity {
    fn from(value: Podcast) -> Self {
        Self {
            id: value.id,
            name: value.name,
            directory_id: value.directory_id,
            rssfeed: value.rssfeed,
            image_url: value.image_url,
            summary: value.summary,
            author: value.author,
            keywords: value.keywords,
            active: value.active,
            language: value.language,
            directory_name: value.directory_name,
            explicit: value.explicit,
            last_build_date: value.last_build_date,
            original_image_url: value.original_image_url,
        }
    }
}