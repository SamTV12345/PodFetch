use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::dsl::favorite_podcast_episodes;
use crate::adapters::persistence::dbconfig::schema::*;
use crate::adapters::persistence::dbconfig::DBType;
use crate::constants::inner_constants::{PodcastEpisodeWithFavorited, DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
use crate::models::episode::Episode;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::time::opt_or_empty_string;
use crate::DBType as DbConnection;
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, ParseResult, Utc};
use diesel::dsl::{max, sql, IsNotNull};
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
use crate::adapters::file::file_handler::FileHandlerType;

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
    pub(crate) description: String,
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
    pub(crate) episode_numbering_processed: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) download_location: Option<String>,
}

impl PodcastEpisode {
    pub(crate) fn is_downloaded(&self) -> bool {
        self.download_location.is_some()
    }
}

impl PodcastEpisode {

    pub fn get_podcast_episode_by_internal_id(
        conn: &mut DbConnection,
        podcast_episode_id_to_be_found: i32,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(id.eq(podcast_episode_id_to_be_found))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        Ok(found_podcast_episode)
    }

    pub fn get_position_of_episode(
        timestamp: &str,
        pid: i32,
        conn: &mut DBType,
    ) -> Result<usize, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let result = diesel::QueryDsl::order(
            podcast_episodes
                .filter(podcast_id.eq(pid))
                .filter(date_of_recording.le(timestamp)),
            date_of_recording.desc(),
        )
        .execute(conn)
        .map_err(map_db_error)?;
        Ok(result)
    }

    pub fn get_podcast_episode_by_id(
        podcas_episode_id_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(episode_id.eq(podcas_episode_id_to_be_found))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        Ok(found_podcast_episode)
    }

    pub fn get_podcast_episode_by_url(
        podcas_episode_url_to_be_found: &str,
        i: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        let found_podcast_epsiode = if let Some(i_unwrapped) = i {
            podcast_episodes
                .filter(
                    url.eq(podcas_episode_url_to_be_found)
                        .and(podcast_id.eq(i_unwrapped)),
                )
                .first::<PodcastEpisode>(&mut get_connection())
                .optional()
                .map_err(map_db_error)?
        } else {
            podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found))
                .first::<PodcastEpisode>(&mut get_connection())
                .optional()
                .map_err(map_db_error)?
        };

        Ok(found_podcast_epsiode)
    }

    pub fn query_podcast_episode_by_url(
        podcas_episode_url_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.like("%".to_owned() + podcas_episode_url_to_be_found + "%"))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode)
    }

    pub fn insert_podcast_episodes(
        podcast: Podcast,
        item: Item,
        optional_image: Option<String>,
        duration: i32,
    ) -> PodcastEpisode {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        let uuid_podcast = uuid::Uuid::new_v4();

        let mut inserted_date = "".to_string();

        if let Some(date) = &item.pub_date {
            fn parse_naive(timestring: &str) -> ParseResult<DateTime<FixedOffset>> {
                let date_without_weekday = &timestring[5..];
                DateTime::parse_from_str(
                    date_without_weekday,
                    "%d %b %Y \
                    %H:%M:%S %z",
                )
            }

            let parsed_date = DateTime::parse_from_rfc2822(date).unwrap_or(
                DateTime::parse_from_rfc3339(date).unwrap_or(
                    parse_naive(date)
                        .unwrap_or(DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z").unwrap()),
                ),
            );
            inserted_date = parsed_date.with_timezone(&Utc).to_rfc3339();
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
            .get_result::<PodcastEpisode>(&mut get_connection())
            .expect("Error inserting podcast episode");

        inserted_podcast
    }

    pub fn get_podcast_episodes_of_podcast(
        podcast_id_to_be_searched: i32,
        last_id: Option<String>,
        only_unlistened: Option<bool>,
        user: &User,
    ) -> PodcastEpisodeWithFavorited {
        use crate::adapters::persistence::dbconfig::schema::episodes as phistory;
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::episode_id as e_fav_episodes;
        use crate::adapters::persistence::dbconfig::schema::favorite_podcast_episodes::username as u_fav_episodes;

        use crate::adapters::persistence::dbconfig::schema::episodes::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::position as phistory_position;
        use crate::adapters::persistence::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::adapters::persistence::dbconfig::schema::episodes::total as phistory_total;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as phistory_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::*;
        let (ph1, ph2) = diesel::alias!(phistory as ph1, phistory as ph2);

        let subquery = ph2
            .select(max(ph2.field(phistory_date)))
            .filter(ph2.field(phistory_username).eq(&user.username))
            .filter(ph2.field(eguid).eq(ph1.field(eguid)))
            .group_by(ph2.field(eguid));

        let mut podcast_query = podcast_episodes
            .filter(podcast_id.eq(podcast_id_to_be_searched))
            .left_join(ph1.on(ph1.field(eguid).eq(guid.nullable())))
            .left_join(
                favorite_podcast_episodes
                    .on(e_fav_episodes.eq(id).and(u_fav_episodes.eq(&user.username))),
            )
            .filter(
                ph1.field(phistory_date)
                    .nullable()
                    .eq_any(subquery)
                    .or(ph1.field(phistory_date).is_null()),
            )
            .order(date_of_recording.desc())
            .limit(75)
            .into_boxed();

        if let Some(last_id) = &last_id {
            podcast_query = podcast_query.filter(date_of_recording.lt(last_id));
        }

        if let Some(only_unlistened) = &only_unlistened {
            if *only_unlistened {
                podcast_query = podcast_query.filter(
                    ph1.field(phistory_position)
                        .is_null()
                        .or(ph1.field(phistory_total).ne(ph1.field(phistory_position))),
                );
            }
        }

        podcast_query
            .load::<(
                PodcastEpisode,
                Option<Episode>,
                Option<FavoritePodcastEpisode>,
            )>(&mut get_connection())
            .map_err(map_db_error)
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
        file_image_path: &str,
        file_episode_path: &str,
        conn: &mut DbConnection,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_id as episode_id_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::file_episode_path as file_episode_path_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::file_image_path as file_image_path_column;
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
                    file_episode_path_column.eq(file_episode_path),
                    file_image_path_column.eq(file_image_path),
                ))
                .execute(conn)
                .map_err(map_db_error)?;
        }
        Ok(())
    }

    pub fn delete_episodes_of_podcast(podcast_id: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;

        Self::get_episodes_by_podcast_id(podcast_id)?
            .iter()
            .for_each(|episode| {
                PlaylistItem::delete_playlist_item_by_episode_id(episode.id, &mut get_connection())
                    .expect("Error deleting episode");
            });

        delete(podcast_episodes)
            .filter(podcast_id_column.eq(podcast_id))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn update_podcast_image(id: &str, image_url: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::directory_id;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::image_url as image_url_column;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts as dsl_podcast;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::download_location as
        podcast_download_location;

        let result = dsl_podcast
            .filter(directory_id.eq(id))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .expect("Error loading podcast episode by id");
        match result {
            Some(..) => {
                diesel::update(dsl_podcast.filter(directory_id.eq(id)))
                    .set((image_url_column.eq(image_url),
                          podcast_download_location.eq(ENVIRONMENT_SERVICE.default_file_handler.get_type()
                            .to_string())
                    ))
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
                Ok(())
            }
            None => {
                panic!("Podcast episode not found");
            }
        }
    }


    pub fn is_downloaded_eq() ->
                              IsNotNull<crate::adapters::persistence::dbconfig::schema::podcast_episodes::download_location> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::download_location;

        download_location.is_not_null()
    }

    pub fn check_if_downloaded(download_episode_url: &str) -> Result<bool, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;

        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::url as podcast_episode_url;

        let result = dsl_podcast_episodes
            .filter(Self::is_downloaded_eq())
            .filter(podcast_episode_url.eq(download_episode_url))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
        .map_err(map_db_error)?;
        Ok(result.is_some())
    }

    pub fn update_podcast_episode_status(
        download_url_of_episode: &str,
        download_location_to_set: Option<FileHandlerType>
    ) -> Result<PodcastEpisode, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let updated_podcast =
            diesel::update(podcast_episodes.filter(url.eq(download_url_of_episode)))
                .set((
                    download_location.eq::<Option<String>>(download_location_to_set.map
                    (|d|d.to_string())),
                    download_time.eq(Utc::now().naive_utc()),
                ))
                .get_result::<PodcastEpisode>(&mut get_connection())
                .expect("Error updating podcast episode");

        Ok(updated_podcast)
    }

    pub fn get_episodes() -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        dsl_podcast_episodes
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn get_podcast_episodes_older_than_days(
        days: i32,
        podcast_id_to_search: i32,
    ) -> Vec<PodcastEpisode> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        podcast_episodes
            .filter(download_time.lt(Utc::now().naive_utc() - Duration::days(days as i64)))
            .filter(podcast_id.eq(podcast_id_to_search))
            .load::<PodcastEpisode>(&mut get_connection())
            .expect("Error loading podcast episode by id")
    }

    pub fn remove_download_status_of_episode(id_to_find: i32) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        do_retry(|| {
            diesel::update(podcast_episodes.filter(id.eq(id_to_find)))
                .set((
                    download_location.eq(sql("NULL")),
                    download_time.eq(sql("NULL")),
                    file_episode_path.eq(sql("NULL")),
                    file_image_path.eq(sql("NULL")),
                ))
                .get_result::<PodcastEpisode>(&mut get_connection())
        })
        .expect("Error updating podcast episode");
    }

    pub fn get_episodes_by_podcast_id(
        id_to_search: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .load::<PodcastEpisode>(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn update_guid(guid_to_update: Guid, podcast_episode_id_to_update: &str) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_id as podcast_episode_id;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(
            podcast_episodes.filter(podcast_episode_id.eq(podcast_episode_id_to_update)),
        )
        .set(guid.eq(guid_to_update.value))
        .execute(&mut get_connection())
        .expect("Error updating guide");
    }

    pub fn update_podcast_episode(episode_to_update: PodcastEpisode) -> PodcastEpisode {
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

    pub fn update_episode_numbering_processed(
        conn: &mut DBType,
        processed: bool,
        episode_id_to_update: &str,
    ) {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::episode_numbering_processed as episode_numbering_processed_column;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        diesel::update(dsl_podcast_episodes)
            .set(episode_numbering_processed_column.eq(processed))
            .filter(episode_id.eq(episode_id_to_update))
            .execute(conn)
            .expect("Error updating episode numbering processed");
    }
}
