use diesel::{Insertable, Queryable, QueryableByName};
use crate::domain::models::playlist::playlist_item::PlaylistItem;

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

impl Into<PlaylistItem> for PlaylistItemEntity {
    fn into(self) -> PlaylistItem {
        PlaylistItem {
            playlist_id: self.playlist_id,
            episode: self.episode,
            position: self.position,
        }
    }
}

impl From<PlaylistItem> for PlaylistItemEntity {
    fn from(value: PlaylistItem) -> Self {
        PlaylistItemEntity {
            playlist_id: value.playlist_id,
            episode: value.episode,
            position: value.position,
        }
    }
}