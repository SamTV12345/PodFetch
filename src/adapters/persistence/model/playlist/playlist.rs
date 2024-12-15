use diesel::{Insertable, Queryable, QueryableByName};
use crate::domain::models::playlist::playlist::Playlist;
use diesel::sql_types::Text;

#[derive(Queryable, Insertable, QueryableByName, Clone)]
#[table_name = "playlists"]
pub struct PlaylistEntity {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Integer)]
    pub user_id: i32,
}

impl From<Playlist> for PlaylistEntity {
    fn from(value: Playlist) -> Self {
        PlaylistEntity {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
        }
    }
}


impl Into<Playlist> for PlaylistEntity {
    fn into(self) -> Playlist {
        Playlist {
            id: self.id,
            name: self.name,
            user_id: self.user_id,
        }
    }
}