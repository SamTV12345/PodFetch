use diesel::sql_types::Text;
use diesel::{Queryable, QueryableByName};

#[derive(QueryableByName, Queryable)]
pub struct GPodderAvailablePodcastsEntity {
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String
}