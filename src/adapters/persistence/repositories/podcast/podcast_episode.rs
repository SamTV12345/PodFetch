use actix::ActorStreamExt;
use chrono::{DateTime, Duration, Utc};
use diesel::{delete, insert_into, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, TextExpressionMethods};
use diesel::dsl::{max, sql, NotLike};
use diesel::query_source::Alias;
use rss::{Guid, Item};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::DBType;
use crate::adapters::persistence::dbconfig::schema::favorites::dsl::favorites;
use crate::adapters::persistence::dbconfig::schema::favorites::favored;
use crate::adapters::persistence::dbconfig::schema::podcast_episodes::date_of_recording;
use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
use crate::adapters::persistence::model::podcast::episode::EpisodeEntity;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::{PodcastEpisodeEntity};
use crate::constants::inner_constants::DEFAULT_IMAGE_URL;
use crate::domain::models::episode::episode::Episode;
use crate::domain::models::favorite::favorite::Favorite;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::time::opt_or_empty_string;

pub struct PodcastEpisodeRepositoryImpl;


impl PodcastEpisodeRepositoryImpl {
    pub fn get_podcast_episode_by_internal_id(
        podcast_episode_id_to_be_found: i32,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(id.eq(podcast_episode_id_to_be_found))
            .first::<PodcastEpisodeEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error).map(|e|e.map(|e|e.into()))?;

        Ok(found_podcast_episode)
    }

    pub fn get_position_of_episode(
        timestamp: &str,
        pid: i32,
    ) -> Result<i64, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

            podcast_episodes
                .filter(podcast_id.eq(pid))
                .filter(date_of_recording.le(timestamp))
                .count()
                .get_result::<i64>(&mut get_connection())
            .map_err(map_db_error)
    }


    pub fn get_podcast_episode_by_id(
        podcas_episode_id_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(episode_id.eq(podcas_episode_id_to_be_found))
            .first::<PodcastEpisodeEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        Ok(found_podcast_episode.into())
    }

    pub fn get_podcast_episode_by_url(
        podcas_episode_url_to_be_found: &str,
        i: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        let found_podcast_episode = if let Some(i_unwrapped) = i {
            podcast_episodes
                .filter(
                    url.eq(podcas_episode_url_to_be_found)
                        .and(podcast_id.eq(i_unwrapped)),
                )
                .first::<PodcastEpisodeEntity>(&mut get_connection())
                .optional()
                .map_err(map_db_error)?
        } else {
            podcast_episodes
                .filter(url.eq(podcas_episode_url_to_be_found))
                .first::<PodcastEpisodeEntity>(&mut get_connection())
                .optional()
                .map_err(map_db_error)?
        };

        Ok(found_podcast_episode.into())
    }

    pub fn query_podcast_episode_by_url(
        podcas_episode_url_to_be_found: &str,
    ) -> Result<Option<PodcastEpisode>, String> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let found_podcast_episode = podcast_episodes
            .filter(url.like("%".to_owned() + podcas_episode_url_to_be_found + "%"))
            .first::<PodcastEpisodeEntity>(&mut get_connection())
            .optional()
            .expect("Error loading podcast by id");

        Ok(found_podcast_episode.into())
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
            let parsed_date = DateTime::parse_from_rfc2822(date);

            match parsed_date {
                Ok(date) => {
                    let conv_date = date.with_timezone(&Utc);
                    inserted_date = conv_date.to_rfc3339()
                }
                Err(_) => {
                    // Sometimes it occurs that day of the week and date are wrong. This just
                    // takes the date and parses it
                    let date_without_weekday = date[5..].to_string();
                    DateTime::parse_from_str(
                        &date_without_weekday,
                        "%d %b %Y \
                    %H:%M:%S %z",
                    )
                        .map(|date| {
                            let conv_date = date.with_timezone(&Utc);
                            inserted_date = conv_date.to_rfc3339()
                        })
                        .expect("Error parsing date");
                }
            }
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
            .get_result::<PodcastEpisodeEntity>(&mut get_connection())
            .expect("Error inserting podcast episode");

        inserted_podcast.into()
    }


    pub fn get_timeline(
        username_to_search: &str,
        favored_only: bool,
        last_timestamp: Option<String>,
        not_listened: bool
    ) -> Result<(i32, (PodcastEpisode, Podcast, Option<Episode>, Option<Favorite>)), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::id as pid;

        use crate::adapters::persistence::dbconfig::schema::episodes as phi_struct;
        use crate::adapters::persistence::dbconfig::schema::episodes::episode as ehid;
        use crate::adapters::persistence::dbconfig::schema::episodes::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::timestamp as phistory_date;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as phi_username;
        use crate::adapters::persistence::dbconfig::schema::favorites::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::favorites::podcast_id as f_podcast_id;
        use crate::adapters::persistence::dbconfig::schema::favorites::username as f_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::guid as pguid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::podcast_id as e_podcast_id;

        let (ph1, ph2) = diesel::alias!(phi_struct as ph1, phi_struct as ph2);

        let subquery = ph2
            .select(max(ph2.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::timestamp)))
            .filter(ph2.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::username).eq(username_to_search.clone()))
            .group_by(ph2.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::episode));

        let part_query = podcast_episodes
            .inner_join(podcasts.on(crate::adapters::persistence::dbconfig::schema::podcast_episodes::columns::podcast_id.eq(crate::adapters::persistence::dbconfig::schema::podcasts::columns::id)))
            .left_join(ph1.on(ph1.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::guid).eq(crate::adapters::persistence::dbconfig::schema::podcast_episodes::columns::guid.nullable())))
            .filter(
                ph1.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::timestamp)
                    .nullable()
                    .eq_any(subquery.clone())
                    .or(ph1.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::timestamp).is_null()),
            )
            .left_join(
                favorites.on(crate::adapters::persistence::dbconfig::schema::favorites::columns::username
                    .eq(username_to_search.clone())
                    .and(crate::adapters::persistence::dbconfig::schema::favorites::columns::podcast_id.eq(crate::adapters::persistence::dbconfig::schema::podcasts::columns::id))),
            );

        let mut query = part_query
            .clone()
            .order(date_of_recording.desc())
            .limit(20)
            .into_boxed();

        let mut total_count = part_query.clone().count().into_boxed();

        match favored_only {
            true => {
                if let Some(last_id) = last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id.clone()));
                }

                query = query.filter(crate::adapters::persistence::dbconfig::schema::favorites::columns::username.eq(username_to_search.clone()));
                query = query.filter(favored.eq(true));
                total_count = total_count.filter(crate::adapters::persistence::dbconfig::schema::favorites::columns::username.eq(username_to_search.clone()));
            }
            false => {
                if let Some(last_id) = last_timestamp {
                    query = query.filter(date_of_recording.lt(last_id));
                }
            }
        }

        if not_listened {
            query = query.filter(ph1.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::timestamp).nullable().ne_all(subquery.clone()));
            total_count = total_count.filter(ph1.field(crate::adapters::persistence::dbconfig::schema::episodes::columns::timestamp).nullable().ne_all(subquery));
        }
        let results = total_count.get_result::<i64>(&mut get_connection()).map_err(map_db_error)?;
        query
            .load::<(PodcastEpisodeEntity, Podcast, Option<EpisodeEntity>, Option<Favorite>)>(&mut
                get_connection())
            .map_err(map_db_error)
    }

    pub fn query_for_podcast(
        query: &str,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use diesel::TextExpressionMethods;
        podcast_episodes
            .filter(
                name.like(format!("%{}%", query))
                    .or(description.like(format!("%{}%", query))),
            )
            .load::<PodcastEpisodeEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e|e.into_iter().map(|p|p.into()).collect())
    }

    pub fn get_episodes_by_podcast_id(
        id_to_search: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        podcast_episodes
            .filter(podcast_id.eq(id_to_search))
            .load::<PodcastEpisodeEntity>(&mut get_connection())
            .map(|e|e.into_iter().map(|p|p.into()).collect())
            .map_err(map_db_error)
    }

    pub fn delete_episodes_of_podcast(
        podcast_id: i32,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_id as podcast_id_column;

        delete(podcast_episodes)
            .filter(podcast_id_column.eq(podcast_id))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }


    pub fn update_podcast_episode_status(
        download_url_of_episode: &str,
        status_to_insert: &str,
    ) -> Result<PodcastEpisode, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.filter(url.eq(download_url_of_episode)))
            .set((
                status.eq(status_to_insert),
                download_time.eq(Utc::now().naive_utc()),
            ))
            .get_result::<PodcastEpisodeEntity>(&mut get_connection())
            .map(|e| e.into())
            .map_err(map_db_error)
    }

    pub fn get_episodes() -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::podcast_episodes as dsl_podcast_episodes;
        dsl_podcast_episodes
            .load::<PodcastEpisodeEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e| e.into_iter().map(|e| e.into()).collect())
    }

    pub fn get_podcast_episodes_older_than_days(
        days: i32,
        podcast_id_to_search: i32,
    ) -> Result<Vec<PodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        podcast_episodes
            .filter(download_time.lt(Utc::now().naive_utc() - Duration::days(days as i64)))
            .filter(podcast_id.eq(podcast_id_to_search))
            .load::<PodcastEpisodeEntity>(&mut get_connection())
            .map(|e| e.into_iter().map(|e| e.into()).collect())
            .map_err(map_db_error)
    }

    pub fn update_podcast_episode(podcast_episode: &PodcastEpisode) -> Result<PodcastEpisode,
        CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        diesel::update(podcast_episodes.filter(id.eq(podcast_episode.id)))
            .set(podcast_episode)
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(podcast_episode.clone())
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
        conn: &mut crate::adapters::persistence::dbconfig::DBType,
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

    /*
       Updates the deleted status. Deleted means that the user decided to forcefully remove the
       local episode. Thus, it should not be redownloaded with the scheduled download
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
            .load::<PodcastEpisodeEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|e| e.into_iter().map(|e| e.into()).collect())
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