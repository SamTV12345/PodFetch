use actix::ActorStreamExt;
use chrono::{DateTime, Utc};
use diesel::{delete, insert_into, BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, TextExpressionMethods};
use diesel::dsl::{max, NotLike};
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
    ) -> Result<usize, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        let result = diesel::QueryDsl::order(
            podcast_episodes
                .filter(podcast_id.eq(pid))
                .filter(date_of_recording.le(timestamp)),
            date_of_recording.desc(),
        )
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(result)
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
}