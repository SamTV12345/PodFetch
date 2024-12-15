use diesel::{Insertable, Queryable, QueryableByName};

#[derive(
    Debug, Queryable, QueryableByName, Insertable, Clone,
)]
pub struct PlaylistItemEntity {
    #[diesel(sql_type = Text)]
    pub playlist_id: String,
    #[diesel(sql_type = Integer)]
    pub episode: i32,
    #[diesel(sql_type = Integer)]
    pub position: i32,
}