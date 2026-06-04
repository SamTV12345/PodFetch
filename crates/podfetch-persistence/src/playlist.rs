use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::playlist::{Playlist, PlaylistItem, PlaylistRepository};
use uuid::Uuid;

diesel::table! {
    playlists (id) {
        id -> Text,
        name -> Text,
        user_id -> Text,
    }
}

diesel::table! {
    playlist_items (playlist_id, episode) {
        playlist_id -> Text,
        episode -> Text,
        position -> Integer,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = playlists)]
struct PlaylistEntity {
    id: String,
    name: String,
    user_id: String,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = playlists)]
struct PlaylistInsertEntity {
    id: String,
    name: String,
    user_id: String,
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = playlist_items)]
struct PlaylistItemEntity {
    playlist_id: String,
    episode: String,
    position: i32,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = playlist_items)]
struct PlaylistItemInsertEntity {
    playlist_id: String,
    episode: String,
    position: i32,
}

impl From<PlaylistEntity> for Playlist {
    fn from(value: PlaylistEntity) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: Uuid::parse_str(&value.user_id).expect("valid uuid in db"),
        }
    }
}

impl From<Playlist> for PlaylistInsertEntity {
    fn from(value: Playlist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id.to_string(),
        }
    }
}

impl From<PlaylistItemEntity> for PlaylistItem {
    fn from(value: PlaylistItemEntity) -> Self {
        Self {
            playlist_id: value.playlist_id,
            episode: Uuid::parse_str(&value.episode).expect("valid uuid in db"),
            position: value.position,
        }
    }
}

impl From<PlaylistItem> for PlaylistItemInsertEntity {
    fn from(value: PlaylistItem) -> Self {
        Self {
            playlist_id: value.playlist_id,
            episode: value.episode.to_string(),
            position: value.position,
        }
    }
}

pub struct DieselPlaylistRepository {
    database: Database,
}

impl DieselPlaylistRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PlaylistRepository for DieselPlaylistRepository {
    type Error = PersistenceError;

    fn find_by_name(&self, playlist_name: &str) -> Result<Option<Playlist>, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        p_table
            .filter(p_dsl::name.eq(playlist_name))
            .first::<PlaylistEntity>(&mut self.database.connection()?)
            .optional()
            .map(|playlist| playlist.map(Into::into))
            .map_err(Into::into)
    }

    fn insert_playlist(&self, playlist: Playlist) -> Result<Playlist, Self::Error> {
        use self::playlists::table as p_table;

        diesel::insert_into(p_table)
            .values(PlaylistInsertEntity::from(playlist))
            .get_result::<PlaylistEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_id(&self, playlist_id_to_search: &str) -> Result<Option<Playlist>, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        p_table
            .filter(p_dsl::id.eq(playlist_id_to_search))
            .first::<PlaylistEntity>(&mut self.database.connection()?)
            .optional()
            .map(|playlist| playlist.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_user_and_id(
        &self,
        playlist_id_to_search: &str,
        playlist_user_id: Uuid,
    ) -> Result<Option<Playlist>, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        p_table
            .filter(p_dsl::id.eq(playlist_id_to_search))
            .filter(p_dsl::user_id.eq(playlist_user_id.to_string()))
            .first::<PlaylistEntity>(&mut self.database.connection()?)
            .optional()
            .map(|playlist| playlist.map(Into::into))
            .map_err(Into::into)
    }

    fn list_by_user(&self, playlist_user_id: Uuid) -> Result<Vec<Playlist>, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        p_table
            .filter(p_dsl::user_id.eq(playlist_user_id.to_string()))
            .load::<PlaylistEntity>(&mut self.database.connection()?)
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn update_playlist_name(
        &self,
        playlist_id_to_update: &str,
        playlist_user_id: Uuid,
        new_name: &str,
    ) -> Result<usize, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        diesel::update(
            p_table
                .filter(p_dsl::id.eq(playlist_id_to_update))
                .filter(p_dsl::user_id.eq(playlist_user_id.to_string())),
        )
        .set(p_dsl::name.eq(new_name))
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn delete_playlist(
        &self,
        playlist_id_to_delete: &str,
        playlist_user_id: Uuid,
    ) -> Result<usize, Self::Error> {
        use self::playlists::dsl as p_dsl;
        use self::playlists::table as p_table;

        diesel::delete(
            p_table
                .filter(p_dsl::id.eq(playlist_id_to_delete))
                .filter(p_dsl::user_id.eq(playlist_user_id.to_string())),
        )
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn insert_playlist_item(&self, item: PlaylistItem) -> Result<PlaylistItem, Self::Error> {
        use self::playlist_items::dsl as pi_dsl;
        use self::playlist_items::table as pi_table;

        let existing = pi_table
            .filter(pi_dsl::playlist_id.eq(&item.playlist_id))
            .filter(pi_dsl::episode.eq(item.episode.to_string()))
            .first::<PlaylistItemEntity>(&mut self.database.connection()?)
            .optional()?;

        if let Some(existing) = existing {
            return Ok(existing.into());
        }

        diesel::insert_into(pi_table)
            .values(PlaylistItemInsertEntity::from(item))
            .get_result::<PlaylistItemEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn list_items_by_playlist_id(
        &self,
        playlist_id_to_search: &str,
    ) -> Result<Vec<PlaylistItem>, Self::Error> {
        use self::playlist_items::dsl as pi_dsl;
        use self::playlist_items::table as pi_table;

        pi_table
            .filter(pi_dsl::playlist_id.eq(playlist_id_to_search))
            .order(pi_dsl::position.asc())
            .load::<PlaylistItemEntity>(&mut self.database.connection()?)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn delete_items_by_playlist_id(
        &self,
        playlist_id_to_delete: &str,
    ) -> Result<usize, Self::Error> {
        use self::playlist_items::dsl as pi_dsl;
        use self::playlist_items::table as pi_table;

        diesel::delete(pi_table.filter(pi_dsl::playlist_id.eq(playlist_id_to_delete)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn delete_playlist_item(
        &self,
        playlist_id_to_delete: &str,
        episode_id_to_delete: Uuid,
    ) -> Result<usize, Self::Error> {
        use self::playlist_items::dsl as pi_dsl;
        use self::playlist_items::table as pi_table;

        diesel::delete(
            pi_table
                .filter(pi_dsl::playlist_id.eq(playlist_id_to_delete))
                .filter(pi_dsl::episode.eq(episode_id_to_delete.to_string())),
        )
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn delete_items_by_episode_id(&self, episode_id_to_delete: Uuid) -> Result<usize, Self::Error> {
        use self::playlist_items::dsl as pi_dsl;
        use self::playlist_items::table as pi_table;

        diesel::delete(pi_table.filter(pi_dsl::episode.eq(episode_id_to_delete.to_string())))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}
