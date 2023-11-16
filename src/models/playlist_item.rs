use crate::dbconfig::schema::playlist_items;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};
use crate::DBType as DbConnection;
use diesel::dsl::max;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::ExpressionMethods;
use diesel::{Queryable, QueryableByName};
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, Debug, Queryable, QueryableByName, Insertable, Clone, ToSchema,
)]
pub struct PlaylistItem {
    #[diesel(sql_type = Text)]
    pub playlist_id: String,
    #[diesel(sql_type = Integer)]
    pub episode: i32,
    #[diesel(sql_type = Integer)]
    pub position: i32,
}

impl PlaylistItem {
    pub fn insert_playlist_item(
        &self,
        conn: &mut DbConnection,
    ) -> Result<PlaylistItem, CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        let res = playlist_items
            .filter(
                playlist_id
                    .eq(self.playlist_id.clone())
                    .and(episode.eq(self.episode)),
            )
            .first::<PlaylistItem>(conn)
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        diesel::insert_into(playlist_items)
            .values((
                playlist_id.eq(&self.playlist_id),
                episode.eq(&self.episode),
                position.eq(&self.position),
            ))
            .get_result::<PlaylistItem>(conn)
            .map_err(map_db_error)
    }

    pub fn delete_playlist_item(
        playlist_id_1: String,
        episode_1: i32,
        conn: &mut DbConnection,
    ) -> Result<(), diesel::result::Error> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(
            playlist_items.filter(playlist_id.eq(playlist_id_1).and(episode.eq(episode_1))),
        )
        .execute(conn)?;
        Ok(())
    }

    pub fn delete_playlist_item_by_playlist_id(
        playlist_id_1: String,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(playlist_id.eq(playlist_id_1)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_playlist_items_by_playlist_id(
        playlist_id_1: String,
        conn: &mut DbConnection,
        user: User,
    ) -> Result<Vec<(PlaylistItem, PodcastEpisode, Option<PodcastHistoryItem>)>, CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as epid;
        use crate::dbconfig::schema::podcast_episodes::dsl::id as eid;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::dbconfig::schema::podcast_history_items as phistory;
        use crate::dbconfig::schema::podcast_history_items::date as phistory_date;
        use crate::dbconfig::schema::podcast_history_items::episode_id as phistory_episode_id;
        use crate::dbconfig::schema::podcast_history_items::username as phistory_username;
        let (ph1, ph2) = diesel::alias!(phistory as ph1, phistory as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(phistory_episode_id).eq(epid))
            .filter(ph2.field(phistory_username).eq(user.username))
            .group_by(ph2.field(phistory_episode_id));

        playlist_items
            .filter(playlist_id.eq(playlist_id_1))
            .inner_join(podcast_episodes.on(episode.eq(eid)))
            .left_join(ph1.on(ph1.field(phistory_episode_id).eq(epid)))
            .filter(ph1.field(phistory_date).nullable().eq_any(subquery))
            .order(position.asc())
            .load::<(PlaylistItem, PodcastEpisode, Option<PodcastHistoryItem>)>(conn)
            .map_err(map_db_error)
    }

    pub fn delete_playlist_item_by_episode_id(
        episode_id_1: i32,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(episode.eq(episode_id_1)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }
}
