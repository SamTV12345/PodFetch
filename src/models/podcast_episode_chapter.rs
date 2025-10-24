use diesel::{AsChangeset, Identifiable, Queryable, QueryableByName, Selectable};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, BigInt, Text};

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    ToSchema,
    Serialize,
    Deserialize,
    Default,
    AsChangeset,
)]
pub struct PodcastEpisodeChapter {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Integer)]
    pub episode_id: i32,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = BigInt)]
    pub start_time: i64,
    #[diesel(sql_type = BigInt)]
    pub end_time: i64,
    #[diesel(sql_type = Text)]
    pub href: Option<String>,
    #[diesel(sql_type = Text)]
    pub image: Option<String>,
}
