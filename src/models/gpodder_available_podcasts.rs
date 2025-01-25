use diesel::sql_types::Text;
use diesel::{Queryable, QueryableByName};
use utoipa::ToSchema;

#[derive(Serialize, QueryableByName, Queryable, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderAvailablePodcasts {
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
}
