use diesel::sql_types::Text;
use diesel::{Queryable, QueryableByName};

#[derive(Serialize, QueryableByName, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct GPodderAvailablePodcasts {
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
}
