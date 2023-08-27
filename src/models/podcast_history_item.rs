use chrono::{NaiveDateTime, Utc};
use diesel::{BoolExpressionMethods, delete, ExpressionMethods, insert_into, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl};
use crate::DbConnection;
use diesel::Queryable;
use diesel::QueryId;
use diesel::QueryableByName;
use crate::models::misc_models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel};
use utoipa::ToSchema;
use diesel::Selectable;
use crate::models::episode::{Episode, EpisodeAction};
use diesel::sql_types::*;
use crate::constants::inner_constants::STANDARD_USER;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::mapping_service::MappingService;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Serialize, Deserialize, Queryable, QueryableByName, Clone, ToSchema, QueryId,Selectable,
Debug)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name=crate::dbconfig::schema::podcast_history_items)]
pub struct PodcastHistoryItem {
    #[diesel(sql_type = Integer, column_name=id)]
    pub id: i32,
    #[diesel(sql_type = Integer, column_name=podcast_id)]
    pub podcast_id: i32,
    #[diesel(sql_type = Text,column_name=episode_id)]
    pub episode_id: String,
    #[diesel(sql_type = Integer, column_name=watched_time)]
    pub watched_time: i32,
    #[diesel(sql_type = Timestamp,column_name=date)]
    pub date: NaiveDateTime,
    #[diesel(sql_type = Text,column_name=username)]
    pub username: String
}

impl PodcastHistoryItem{
    pub fn delete_by_username(username1: String, conn: &mut DbConnection) -> Result<(),
        diesel::result::Error>{
        use crate::dbconfig::schema::podcast_history_items::dsl::*;
        diesel::delete(podcast_history_items.filter(username.eq(username1)))
            .execute(conn)?;
        Ok(())
    }

    pub fn log_watchtime(conn: &mut DbConnection, watch_model: PodcastWatchedPostModel,
                         designated_username: String) -> Result<(), CustomError> {
        let result = PodcastEpisode::get_podcast_episode_by_id(conn, &watch_model.podcast_episode_id)
            .unwrap();

        use crate::dbconfig::schema::podcast_history_items::dsl::*;
        match result {
            Some(result) => {
                let naive_date_time = Utc::now().naive_utc();

                insert_into(podcast_history_items)
                    .values((
                        podcast_id.eq(result.podcast_id),
                        episode_id.eq(result.episode_id),
                        watched_time.eq(watch_model.time),
                        date.eq(naive_date_time),
                        username.eq(designated_username),
                    ))
                    .execute(conn)
                    .map_err(map_db_error)?;
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn delete_watchtime(conn: &mut DbConnection, podcast_id_to_delete: i32) -> Result<(),
        CustomError> {
        use crate::dbconfig::schema::podcast_history_items::dsl::*;

        delete(podcast_history_items)
            .filter(podcast_id.eq(podcast_id_to_delete))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_watchtime(
        conn: &mut DbConnection,
        podcast_id_tos_search: &str,
        username_to_find: String
    ) -> Result<PodcastHistoryItem, CustomError> {
        let result = PodcastEpisode::get_podcast_episode_by_id(conn, podcast_id_tos_search)?;
        use crate::dbconfig::schema::podcast_history_items::dsl::*;

        match result {
            Some(found_podcast) => {
                let history_item = podcast_history_items
                    .filter(episode_id.eq(podcast_id_tos_search).and(username.eq(username_to_find.clone())))
                    .order(date.desc())
                    .first::<PodcastHistoryItem>(conn)
                    .optional()
                    .map_err(map_db_error)?;

                match history_item {
                    Some(found_history_item) => {
                        let option_episode = Episode::get_watch_log_by_username_and_episode
                            (username_to_find.clone(), conn, found_podcast.clone().url)?;
                        if let Some(episode) = option_episode {
                            if episode.action == EpisodeAction::Play.to_string() && episode
                                .position.unwrap()>found_history_item.watched_time && episode.timestamp>found_history_item.date{

                                let found_podcast_item = Podcast::get_podcast(conn,
                                                                             found_history_item
                                    .podcast_id)?;
                                return Ok(Episode::convert_to_podcast_history_item(&episode,
                                                                                   found_podcast_item,
                                                                                   found_podcast));
                            }
                        }
                        Ok(found_history_item)
                    },
                    None => Ok(PodcastHistoryItem {
                        id: 0,
                        podcast_id: found_podcast.podcast_id,
                        episode_id: found_podcast.episode_id,
                        watched_time: 0,
                        username: STANDARD_USER.to_string(),
                        date: Utc::now().naive_utc()
                    }),
                }
            }
            None => {
                Err(CustomError::NotFound)
            }
        }
    }


    pub fn get_last_watched_podcasts(
        conn: &mut DbConnection,
        designated_username: String, mapping_service: MappingService) ->
                                                                          Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_history_items;
        
        use crate::dbconfig::schema::podcast_history_items::dsl::episode_id as ehid;
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as eid;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use diesel::NullableExpressionMethods;

        let (history_item1, history_item2) = diesel::alias!(podcast_history_items as p1, podcast_history_items
            as p2);

        let subquery = history_item1
            .select(diesel::dsl::max(history_item1.field(podcast_history_items::date)))
            .filter(history_item1.field(podcast_history_items::episode_id).eq(history_item1.field(ehid)))
            .group_by(history_item1.field(ehid));

        let result = history_item2
            .inner_join(podcast_episodes.on(history_item2.field(ehid).eq(eid)))
            .filter(history_item2.field(podcast_history_items::username).eq(designated_username))
            .filter(history_item2.field(podcast_history_items::date).nullable().eq_any( subquery))
            .load::<(PodcastHistoryItem, PodcastEpisode)>(conn)
            .map_err(map_db_error)?;


        let podcast_watch_episode = result.iter().map(|(podcast_history_item, podcast_episode)| {
            let podcast_dto = mapping_service
                .map_podcastepisode_to_dto(podcast_episode);
            let podcast = Podcast::get_podcast(conn, podcast_episode.podcast_id).unwrap();
            mapping_service
                .map_podcast_history_item_to_with_podcast_episode(
                    &podcast_history_item.clone(),
                    podcast_dto,
                    podcast,
                )
        }).collect::<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>();
        Ok(podcast_watch_episode)
    }

    pub fn get_watch_logs_by_username(username_to_search: String, conn: &mut DbConnection,
                                      since: NaiveDateTime)
                                      ->
                                      Result<Vec<(PodcastHistoryItem, PodcastEpisode, Podcast)>,
                                          CustomError> {
        use crate::dbconfig::schema::podcast_history_items;
        use crate::dbconfig::schema::podcasts;
        use crate::dbconfig::schema::podcast_episodes;

        podcast_history_items::table
            .inner_join(podcast_episodes::table.on(podcast_history_items::episode_id.eq(podcast_episodes::episode_id)))
            .inner_join(podcasts::table)
            .filter(podcast_history_items::episode_id.eq(podcast_episodes::episode_id))
            .filter(podcast_history_items::podcast_id.eq(podcasts::id))
            .filter(podcast_history_items::username.eq(username_to_search))
            .filter(podcast_history_items::date.ge(since))
            .load::<(PodcastHistoryItem, PodcastEpisode, Podcast)>(conn)
            .map_err(map_db_error)
    }
}