use diesel::{BoolExpressionMethods, ExpressionMethods, Insertable, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, Queryable, QueryableByName, RunQueryDsl};
use diesel::dsl::max;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::playlist::playlist_item::PlaylistItemEntity;
use crate::adapters::persistence::model::podcast::episode::EpisodeEntity;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::PodcastEpisodeEntity;
use crate::domain::models::episode::episode::Episode;
use crate::domain::models::playlist::playlist_item::PlaylistItem;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::models::user::User;
use crate::utils::error::{map_db_error, CustomError};

pub struct PlaylistItemRepositoryImpl;



impl PlaylistItemRepositoryImpl {
    pub fn insert_playlist_item(playlist_item: PlaylistItem) ->
                                               Result<PlaylistItem, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        let res = playlist_items
            .filter(
                playlist_id
                    .eq(&playlist_item.playlist_id)
                    .and(episode.eq(playlist_item.episode)),
            )
            .first::<PlaylistItemEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res.into());
        }

        diesel::insert_into(playlist_items)
            .values((
                playlist_id.eq(&playlist_item.playlist_id),
                episode.eq(&playlist_item.episode),
                position.eq(&playlist_item.position),
            ))
            .get_result::<PlaylistItemEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|res| res.into())
    }

    pub fn delete_playlist_item(
        playlist_id_1: String,
        episode_1: i32,
        conn: &mut crate::adapters::persistence::dbconfig::DBType,
    ) -> Result<(), diesel::result::Error> {
        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(
            playlist_items.filter(playlist_id.eq(playlist_id_1).and(episode.eq(episode_1))),
        )
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_playlist_item_by_playlist_id(
        playlist_id_to_delete: String,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(playlist_id.eq(playlist_id_to_delete)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_playlist_items_by_playlist_id(
        playlist_id_to_load_from: String,
        user: User,
    ) -> Result<Vec<(PlaylistItem, PodcastEpisode, Option<Episode>)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes as episode_item;
        use crate::adapters::persistence::dbconfig::schema::episodes::episode as phistory_episode_id;
        use crate::adapters::persistence::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as phistory_username;
        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_id as epid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::id as eid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        let (ph1, ph2) = diesel::alias!(episode_item as ph1, episode_item as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(phistory_episode_id).eq(epid))
            .filter(ph2.field(phistory_username).eq(user.username))
            .group_by(ph2.field(phistory_episode_id));

        playlist_items
            .filter(playlist_id.eq(playlist_id_to_load_from))
            .inner_join(podcast_episodes.on(episode.eq(eid)))
            .left_join(ph1.on(ph1.field(phistory_episode_id).eq(epid)))
            .filter(ph1.field(phistory_date).nullable().eq_any(subquery))
            .order(position.asc())
            .load::<(PlaylistItemEntity, PodcastEpisodeEntity, Option<EpisodeEntity>)>(&mut
                get_connection())
            .map_err(map_db_error)
            .map(|res| {
                res.into_iter()
                    .map(|(item, podcast_episode, episode)| {
                        (
                            item.into(),
                            podcast_episode.into(),
                            episode.map(|e| e.into()),
                        )
                    })
                    .collect()
            })
    }

    pub fn delete_playlist_item_by_episode_id(
        episode_id_to_delete: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::playlist_items::dsl::*;

        diesel::delete(playlist_items.filter(episode.eq(episode_id_to_delete)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }
}