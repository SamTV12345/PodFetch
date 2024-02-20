use crate::constants::inner_constants::DEFAULT_IMAGE_URL;
use crate::dbconfig::schema::*;
use crate::models::episode::Episode;
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

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    ToSchema,
    Serialize,
    Deserialize,
    Default,
    AsChangeset,
)]
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
    pub(crate) deleted: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_episode_path: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_image_path: Option<String>,
}

impl PodcastEpisode {
    pub fn is_downloaded(&self) -> bool {
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
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        let found_podcast_epsiode = if let Some(i_unwrapped) = i {
            podcast_episodes
                .filter(
                    url.eq(podcas_episode_url_to_be_found)
                        .and(podcast_id.eq(i_unwrapped)),
                )
                .first::<PodcastEpisode>(conn)
                .optional()
                .map_err(map_db_error)?
        } else {
            podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found))
                .first::<PodcastEpisode>(conn)
                .optional()
                .map_err(map_db_error)?
        };

        Ok(found_podcast_epsiode)
    }

    pub fn query_podcast_episode_by_url(
        conn: &mut DbConnection,
        podcas_episode_url_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.like("%".to_owned() + podcas_episode_url_to_be_found + "%"))
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
                let date = DateTime::parse_from_rfc2822(date).unwrap_or(DateTime::from(Utc::now()));
                inserted_date = date.to_rfc3339()
            }
            None => {}
        }

        let inserted_image_url: String = match optional_image {
            Some(c) => c,
            None => match podcast.image_url.is_empty() {
                true => DEFAULT_IMAGE_URL.to_string(),
                false => podcast.original_image_url,
            },
        };

        let guid_to_insert = Guid {
            value: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        };
        let inserted_podcast = insert_into(podcast_episodes)
            .values((
                total_time.eq(duration),
                podcast_id.eq(podcast.id),
                episode_id.eq(uuid_podcast.to_string()),
                name.eq(item.title.as_ref().unwrap_or(&"No title given".to_string())),
                url.eq(item.enclosure.unwrap().url),
                guid.eq(item.guid.unwrap_or(guid_to_insert).value),
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
        user: User,
    ) -> Result<Vec<(PodcastEpisode, Option<Episode>)>, CustomError> {
        use crate::dbconfig::schema::episodes as phistory;
        use crate::dbconfig::schema::episodes::guid as eguid;
        use crate::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::dbconfig::schema::episodes::username as phistory_username;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::dbconfig::schema::podcast_episodes::*;
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
                    .load::<(PodcastEpisode, Option<Episode>)>(conn)
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
                    .load::<(PodcastEpisode, Option<Episode>)>(conn)
                    .expect("Error loading podcasts");

                Ok(podcasts_found)
            }
        }
    }

    pub fn get_last_n_podcast_episodes(
        conn: &mut DbConnection,
        podcast_episode_id: i32,
        number_to_download: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(podcast_episode_id))
            .limit(number_to_download as i64)
            .order(date_of_recording.desc())
            .load::<PodcastEpisode>(conn)
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
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::file_episode_path as file_episode_path_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::file_image_path as file_image_path_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::local_image_url as local_image_url_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::local_url as local_url_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;

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

    pub fn delete_episodes_of_podcast(
        conn: &mut DbConnection,
        podcast_id: i32,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;

        Self::get_episodes_by_podcast_id(podcast_id, conn)
            .iter()
            .for_each(|episode| {
                PlaylistItem::delete_playlist_item_by_episode_id(episode.id, conn)
                    .expect("Error deleting episode");
            });

        delete(podcast_episodes)
            .filter(podcast_id_column.eq(podcast_id))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_podcast_image(
        id: &str,
        image_url: &str,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
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
        conn: &mut DbConnection,
    ) -> Result<bool, CustomError> {
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
    ) -> Result<PodcastEpisode, CustomError> {
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

    pub fn get_podcast_episodes_older_than_days(
        days: i32,
        conn: &mut DbConnection,
    ) -> Vec<PodcastEpisode> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        podcast_episodes
            .filter(download_time.lt(Utc::now().naive_utc() - Duration::days(days as i64)))
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn update_download_status_of_episode(id_to_find: i32, conn: &mut DbConnection) {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
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
                .get_result::<PodcastEpisode>(conn)
        })
        .expect("Error updating podcast episode");
    }

    pub fn get_episodes_by_podcast_id(
        id_to_search: i32,
        conn: &mut DbConnection,
    ) -> Vec<PodcastEpisode> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .load::<PodcastEpisode>(conn)
            .expect("Error loading podcast episode by id")
    }

    pub fn update_guid(
        conn: &mut DbConnection,
        guid_to_update: Guid,
        podcast_episode_id_to_update: &str,
    ) {
        use crate::dbconfig::schema::podcast_episodes::dsl::episode_id as podcast_episode_id;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(
            podcast_episodes.filter(podcast_episode_id.eq(podcast_episode_id_to_update)),
        )
        .set(guid.eq(guid_to_update.value))
        .execute(conn)
        .expect("Error updating guide");
    }

    pub fn update_podcast_episode(
        conn: &mut DbConnection,
        episode_to_update: PodcastEpisode,
    ) -> PodcastEpisode {
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
    pub fn update_deleted(
        conn: &mut DbConnection,
        episode_to_update: &str,
        deleted_status: bool,
    ) -> Result<usize, CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.filter(episode_id.eq(episode_to_update)))
            .set(deleted.eq(deleted_status))
            .execute(conn)
            .map_err(map_db_error)
    }

    pub fn get_podcast_episodes_by_podcast_to_k(
        conn: &mut DbConnection,
        top_k: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::podcast_episodes as p_episode;
        use crate::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::dbconfig::schema::podcasts::id as pod_id;
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
            .load::<PodcastEpisode>(conn)
            .map_err(map_db_error)
    }
}
