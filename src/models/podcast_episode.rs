use crate::dbconfig::schema::*;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::prelude::{Queryable, Identifiable, Selectable, QueryableByName};
use diesel::{BoolExpressionMethods, delete, insert_into, OptionalExtension, RunQueryDsl, TextExpressionMethods};
use diesel::dsl::sql;
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp, Bool};
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use rss::{Guid, Item};
use crate::DbConnection;
use crate::models::podcasts::Podcast;
use crate::utils::do_retry::do_retry;
use crate::utils::time::opt_or_empty_string;
use diesel::AsChangeset;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Queryable, Identifiable,QueryableByName, Selectable, Debug, PartialEq, Clone, ToSchema,
Serialize, Deserialize, Default, AsChangeset)]
pub struct PodcastEpisode {
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
    pub (crate) deleted: bool,
}

impl PodcastEpisode{
    pub fn is_downloaded(&self) -> bool{
        self.status == "D"
    }

    pub fn get_podcast_episode_by_internal_id(
        conn: &mut DbConnection,
        podcast_episode_id_to_be_found: i32,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(id.eq(podcast_episode_id_to_be_found))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_episode_by_id(
        conn: &mut DbConnection,
        podcas_episode_id_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(episode_id.eq(podcas_episode_id_to_be_found))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_episode_by_url(
        conn: &mut DbConnection,
        podcas_episode_url_to_be_found: &str,
        i: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
       let found_podcast_epsiode = if let Some(i_unwrapped) = i{
             podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found)
                    .and(podcast_id.eq(i_unwrapped)))
                .first::<PodcastEpisode>(conn)
                .optional()
                .expect("Error loading podcast by id")
        }
        else{
            podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found))
                .first::<PodcastEpisode>(conn)
                .optional()
                .expect("Error loading podcast by id")
        };


        Ok(found_podcast_epsiode)
    }

    pub fn query_podcast_episode_by_url(
        conn: &mut DbConnection,
        podcas_episode_url_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.like("%".to_owned()+podcas_episode_url_to_be_found+"%"))
            .first::<PodcastEpisode>(conn)
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }

    pub fn insert_podcast_episodes(
        conn: &mut DbConnection,
        podcast: Podcast,
        item: Item,
        optional_image: Option<String>,
        duration: i32,
    ) -> PodcastEpisode {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        let uuid_podcast = uuid::Uuid::new_v4();

        let mut inserted_date = "".to_string();

        match &item.pub_date {
            Some(date) => {
                let date = DateTime::parse_from_rfc2822(date).expect("Error parsing date");
                inserted_date = date.to_rfc3339()
            }
            None => {}
        }

        let inserted_image_url = match optional_image {
            Some(image_url_podcast_episode) => {
                image_url_podcast_episode
            }
            None => {
                 podcast.original_image_url
            }
        };

        let inserted_podcast = insert_into(podcast_episodes)
            .values((
                total_time.eq(duration),
                podcast_id.eq(podcast.id),
                episode_id.eq(uuid_podcast.to_string()),
                name.eq(item.title.as_ref().unwrap().clone()),
                url.eq(item.enclosure.unwrap().url),
                date_of_recording.eq(inserted_date),
                image_url.eq(inserted_image_url),
                description.eq(opt_or_empty_string(item.description)),
            ))
            .get_result::<PodcastEpisode>(conn)
            .expect("Error inserting podcast episode");


        inserted_podcast
    }


    pub fn get_podcast_episodes_of_podcast(
        conn: &mut DbConnection,
        podcast_id_to_be_searched: i32,
        last_id: Option<String>,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        match last_id {
            Some(last_id) => {
                let podcasts_found = podcast_episodes
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .filter(date_of_recording.lt(last_id))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .map_err(map_db_error)?;
                Ok(podcasts_found)
            }
            None => {
                let podcasts_found = podcast_episodes
                    .filter(podcast_id.eq(podcast_id_to_be_searched))
                    .order(date_of_recording.desc())
                    .limit(75)
                    .load::<PodcastEpisode>(conn)
                    .expect("Error loading podcasts");

                Ok(podcasts_found)
            }
        }
    }

    pub fn get_last_n_podcast_episodes(
        conn: &mut DbConnection,
        podcast_episode_id: i32,
        number_to_download: i32
    ) -> Result<Vec<PodcastEpisode>, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        let podcasts = podcast_episodes
            .filter(podcast_id.eq(podcast_episode_id))
            .limit(number_to_download as i64)
            .order(date_of_recording.desc())
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcasts");
        Ok(podcasts)
    }

    pub fn update_total_podcast_time_and_image(
        episode_id: &str,
        image_url: &str,
        local_download_url: &str,
        conn: &mut DbConnection,
    ) -> Result<(), String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::local_image_url as local_image_url_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        let result = podcast_episodes
            .filter(episode_id_column.eq(episode_id))
            .first::<PodcastEpisode>(conn)
            .optional()
            .expect("Error loading podcast episode by id");

        match result {
            Some(..) => {
                diesel::update(podcast_episodes)
                    .filter(episode_id_column.eq(episode_id))
                    .set((
                        local_image_url_column.eq(image_url),
                        local_url_column.eq(local_download_url),
                    ))
                    .execute(conn)
                    .expect("Error updating local image url");
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn delete_episodes_of_podcast(conn: &mut DbConnection, podcast_id: i32) -> Result<(), String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;


        delete(podcast_episodes)
            .filter(podcast_id_column.eq(podcast_id))
            .execute(conn)
            .expect("Error deleting podcast episodes");
        Ok(())
    }

    pub fn update_podcast_image(id: &str, image_url: &str, conn: &mut DbConnection) -> Result<(), String> {
        use crate::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::dbconfig::schema::podcasts::dsl::image_url as image_url_column;
        use crate::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;

        let result = dsl_podcast
            .filter(directory_id.eq(id))
            .first::<Podcast>(conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(..) => {
                diesel::update(dsl_podcast.filter(directory_id.eq(id)))
                    .set(image_url_column.eq(image_url))
                    .execute(conn)
                    .expect("Error updating podcast episode");
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }

    pub fn check_if_downloaded(download_episode_url: &str, conn: &mut DbConnection) -> Result<bool, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        use crate::dbconfig::schema::podcast_episodes::url as podcast_episode_url;
        let result = dsl_podcast_episodes
            .filter(local_url_column.is_not_null())
            .filter(podcast_episode_url.eq(download_episode_url))
            .first::<PodcastEpisode>(conn)
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(podcast_episode) => {
                return match podcast_episode.status.as_str() {
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

    pub fn update_podcast_episode_status(
        download_url_of_episode: &str,
        status_to_insert: &str,
        conn: &mut DbConnection,
    ) -> Result<PodcastEpisode, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        let updated_podcast =
            diesel::update(podcast_episodes.filter(url.eq(download_url_of_episode)))
                .set((
                    status.eq(status_to_insert),
                    download_time.eq(Utc::now().naive_utc()),
                ))
                .get_result::<PodcastEpisode>(conn)
                .expect("Error updating podcast episode");

        Ok(updated_podcast)
    }


    pub fn get_episodes(conn: &mut DbConnection) -> Vec<PodcastEpisode> {
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        dsl_podcast_episodes
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn get_podcast_episodes_older_than_days(days: i32,conn: &mut DbConnection) ->
    Vec<PodcastEpisode> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        podcast_episodes
            .filter(download_time.lt(Utc::now().naive_utc() - Duration::days(days as i64)))
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcast episode by id")
    }


    pub fn update_download_status_of_episode(id_to_find: i32, conn: &mut DbConnection) {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        do_retry(||{diesel::update(podcast_episodes.filter(id.eq(id_to_find)))
            .set((status.eq("N"), download_time.eq(sql("NULL"))))
            .get_result::<PodcastEpisode>(conn)}
        ).expect("Error updating podcast episode");
    }

    pub fn get_episodes_by_podcast_id(
        id_to_search: i32,
        conn: &mut DbConnection
    ) -> Vec<PodcastEpisode> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn update_guid(conn: &mut DbConnection, guid_to_update:Guid, podcast_episode_id_to_update:
    &str) {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as podcast_episode_id;

        diesel::update(podcast_episodes.filter(podcast_episode_id.eq(podcast_episode_id_to_update)))
            .set(guid.eq(guid_to_update.value))
            .execute(conn)
            .expect("Error updating guide");
    }

    pub fn update_podcast_episode(conn: &mut DbConnection, episode_to_update:PodcastEpisode) ->
    PodcastEpisode {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.find(episode_to_update.id))
            .set(episode_to_update)
            .get_result::<PodcastEpisode>(conn)
            .expect("Error updating podcast episode")
    }

    /*
        Updates the deleted status. Deleted means that the user decided to forcefully remove the
        local episode. Thus it should not be redownloaded with the scheduled download
     */
    pub fn update_deleted(conn: &mut DbConnection, episode_to_update:&str, deleted_status:bool)
        ->Result<usize, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.filter(episode_id.eq(episode_to_update)))
            .set(deleted.eq(deleted_status))
            .execute(conn)
            .map_err(map_db_error)
    }
}