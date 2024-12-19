use crate::constants::inner_constants::DEFAULT_IMAGE_URL;
use crate::adapters::persistence::dbconfig::schema::*;
use crate::adapters::persistence::dbconfig::DBType;
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::time::opt_or_empty_string;
use crate::DBType as DbConnection;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::dsl::{max, sql};
use diesel::prelude::{Identifiable, Queryable, QueryableByName, Selectable};
use diesel::query_source::Alias;
use diesel::sql_types::{Bool, Integer, Nullable, Text, Timestamp};
use diesel::AsChangeset;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{
    delete, insert_into, BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, RunQueryDsl, TextExpressionMethods,
};
use rss::{Guid, Item};
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::domain::models::episode::episode::Episode;

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    Default,
    AsChangeset,
)]
pub struct PodcastEpisodeEntity {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Integer)]
    pub(crate) podcast_id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) episode_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub(crate) url: String,
    #[diesel(sql_type = Text)]
    pub(crate) date_of_recording: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Integer)]
    pub total_time: i32,
    #[diesel(sql_type = Text)]
    pub(crate) local_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) local_image_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) description: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub(crate) download_time: Option<NaiveDateTime>,
    #[diesel(sql_type = Text)]
    pub(crate) guid: String,
    #[diesel(sql_type = Bool)]
    pub(crate) deleted: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_episode_path: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_image_path: Option<String>,
    #[diesel(sql_type = Bool)]
    pub (crate) episode_numbering_processed : bool,
}

impl PodcastEpisodeEntity {
    pub fn is_downloaded(&self) -> bool {
        self.status == "D"
    }


    pub fn get_podcast_episodes_of_podcast(
        podcast_id_to_be_searched: i32,
        last_id: Option<String>,
        user: User,
    ) -> Result<Vec<(PodcastEpisode, Option<Episode>)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes as phistory;
        use crate::adapters::persistence::dbconfig::schema::episodes::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as phistory_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::*;
        let (ph1, ph2) = diesel::alias!(phistory as ph1, phistory as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(phistory_username).eq(user.username))
            .filter(ph2.field(eguid).eq(ph1.field(eguid)))
            .group_by(ph2.field(eguid));

        match last_id {
            Some(last_id) => {
                let podcasts_found = podcast_episodes
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .left_join(ph1.on(ph1.field(eguid).eq(guid.nullable())))
                    .filter(
                        ph1.field(phistory_date)
                            .nullable()
                            .eq_any(subquery)
                            .or(ph1.field(phistory_date).is_null()),
                    )
                    .filter(date_of_recording.lt(last_id))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<(PodcastEpisode, Option<Episode>)>(&mut get_connection())
                    .map_err(map_db_error)?;
                Ok(podcasts_found)
            }
            None => {
                let podcasts_found = podcast_episodes
                    .left_join(ph1.on(ph1.field(eguid).eq(guid.nullable())))
                    .filter(
                        ph1.field(phistory_date)
                            .nullable()
                            .eq_any(subquery)
                            .or(ph1.field(phistory_date).is_null()),
                    )
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<(PodcastEpisode, Option<Episode>)>(&mut get_connection())
                    .expect("Error loading podcasts");

                Ok(podcasts_found)
            }
        }
    }

    pub fn get_last_n_podcast_episodes(
        podcast_episode_id: i32,
        number_to_download: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(podcast_episode_id))
            .limit(number_to_download as i64)
            .order(date_of_recording.desc())
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn update_local_paths(
        episode_id: &str,
        image_url: &str,
        local_download_url: &str,
        file_image_path: &str,
        file_episode_path: &str,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::file_episode_path as file_episode_path_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::file_image_path as file_image_path_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::local_image_url as local_image_url_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;

        let result = podcast_episodes
            .filter(episode_id_column.eq(episode_id))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        if result.is_some() {
            diesel::update(podcast_episodes)
                .filter(episode_id_column.eq(episode_id))
                .set((
                    local_image_url_column.eq(image_url),
                    local_url_column.eq(local_download_url),
                    file_episode_path_column.eq(file_episode_path),
                    file_image_path_column.eq(file_image_path),
                ))
                .execute(conn)
                .map_err(map_db_error)?;
        }
        Ok(())
    }

    pub fn update_podcast_image(
        id: &str,
        image_url: &str,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::image_url as image_url_column;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result = dsl_podcast
            .filter(directory_id.eq(id))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(..) => {
                diesel::update(dsl_podcast.filter(directory_id.eq(id)))
                    .set(image_url_column.eq(image_url))
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn check_if_downloaded(
        download_episode_url: &str,
    ) -> Result<bool, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::url as podcast_episode_url;
        let result = dsl_podcast_episodes
            .filter(local_url_column.is_not_null())
            .filter(podcast_episode_url.eq(download_episode_url))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(podcast_episode) => {
                match podcast_episode.status.as_str() {
                    "N" => Ok(false),
                    "D" => Ok(true),
                    "P" => Ok(false),
                    _ => Ok(false),
                }
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }




    pub fn update_download_status_of_episode(id_to_find: i32) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        do_retry(|| {
            diesel::update(podcast_episodes.filter(id.eq(id_to_find)))
                .set((
                    status.eq("N"),
                    download_time.eq(sql("NULL")),
                    local_url.eq(""),
                    local_image_url.eq(""),
                    file_episode_path.eq(sql("NULL")),
                    file_image_path.eq(sql("NULL")),
                ))
                .get_result::<PodcastEpisode>(&mut get_connection())
        })
        .expect("Error updating podcast episode");
    }

    pub fn get_episodes_by_podcast_id(
        id_to_search: i32,
    ) -> Vec<PodcastEpisode> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .load::<PodcastEpisode>(&mut get_connection())
            .expect("Error loading podcast episode by id")
    }

    pub fn update_guid(
        guid_to_update: Guid,
        podcast_episode_id_to_update: &str,
    ) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_id as podcast_episode_id;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(
            podcast_episodes.filter(podcast_episode_id.eq(podcast_episode_id_to_update)),
        )
        .set(guid.eq(guid_to_update.value))
        .execute(&mut get_connection())
        .expect("Error updating guide");
    }

    pub fn update_podcast_episode(
        episode_to_update: PodcastEpisode,
    ) -> PodcastEpisode {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.find(episode_to_update.id))
            .set(episode_to_update)
            .get_result::<PodcastEpisode>(&mut get_connection())
            .expect("Error updating podcast episode")
    }

    /*
       Updates the deleted status. Deleted means that the user decided to forcefully remove the
       local episode. Thus it should not be redownloaded with the scheduled download
    */
    pub fn update_deleted(
        episode_to_update: &str,
        deleted_status: bool,
    ) -> Result<usize, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.filter(episode_id.eq(episode_to_update)))
            .set(deleted.eq(deleted_status))
            .execute(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn get_podcast_episodes_by_podcast_to_k(
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes as p_episode;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as pod_id;
        let (podcast_episode1, podcast_episode2) = diesel::alias!(p_episode as p1, p_episode as p2);

        podcast_episode1
            .select(Alias::fields(&podcast_episode1, p_episode::all_columns))
            .inner_join(podcasts.on(pod_id.eq(podcast_episode1.field(podcast_id_column))))
            .filter(
                podcast_episode1.field(date_of_recording).eq_any(
                    podcast_episode2
                        .select(podcast_episode2.field(date_of_recording))
                        .filter(podcast_episode2.field(podcast_id_column).eq(pod_id))
                        .order(podcast_episode2.field(date_of_recording).desc())
                        .limit(top_k.into()),
                ),
            )
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn update_episode_numbering_processed(conn: &mut DBType, processed: bool,
                                              episode_id_to_update: &str) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_numbering_processed as episode_numbering_processed_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        diesel::update(dsl_podcast_episodes)
            .set(episode_numbering_processed_column.eq(processed))
            .filter(episode_id.eq(episode_id_to_update))
            .execute(conn)
            .expect("Error updating episode numbering processed");
    }
}
