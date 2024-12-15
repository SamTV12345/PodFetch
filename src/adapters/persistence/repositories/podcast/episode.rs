use std::io::Error;
use chrono::{NaiveDateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl};
use diesel::query_dsl::methods::DistinctDsl;
use reqwest::Url;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::episodes;
use crate::adapters::persistence::model::podcast::episode::EpisodeEntity;
use crate::adapters::persistence::model::podcast::podcast::PodcastEntity;
use crate::adapters::persistence::model::podcast_episode::podcast_episode::PodcastEpisodeEntity;
use crate::constants::inner_constants::DEFAULT_DEVICE;
use crate::domain::models::episode::episode::Episode;
use crate::models::gpodder_available_podcasts::GPodderAvailablePodcasts;
use crate::utils::error::{map_db_error, CustomError};

pub struct EpisodeRepositoryImpl;



impl EpisodeRepositoryImpl {
    pub fn insert_episode(
        episode_to_insert: Episode
    ) -> Result<Episode, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::*;
        let episode_entity = EpisodeEntity::from(episode_to_insert);
        let res = episodes
            .filter(
                timestamp
                    .eq(episode_entity.timestamp)
                    .and(device.eq(&episode_entity.device))
                    .and(podcast.eq(&episode_entity.podcast))
                    .and(episode.eq(&episode_entity.episode))
                    .and(timestamp.eq(episode_entity.timestamp)),
            )
            .first::<EpisodeEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res.into());
        }

        let mut cleaned_url_parsed = Url::parse(&episode_entity.episode).unwrap();
        cleaned_url_parsed.set_query(None);
        diesel::insert_into(episodes)
            .values((
                username.eq(&episode_entity.username),
                device.eq(&episode_entity.device),
                podcast.eq(&episode_entity.podcast),
                episode.eq(&episode_entity.episode),
                timestamp.eq(&episode_entity.timestamp),
                guid.eq(&episode_entity.guid),
                action.eq(&episode_entity.action),
                started.eq(&episode_entity.started),
                position.eq(&episode_entity.position),
                total.eq(&episode_entity.total),
            ))
            .get_result(&mut get_connection())
            .map_err(map_db_error)
    }

    pub fn delete_by_username(username: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;

        diesel::delete(ep_table.filter(ep_dsl::username.eq(username)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub async fn get_actions_by_username(
        username1: String,
        since_date: Option<NaiveDateTime>,
        opt_device: Option<String>,
        _opt_aggregate: Option<String>,
        opt_podcast: Option<String>,
    ) -> Vec<Episode> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::timestamp;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;
        use crate::adapters::persistence::dbconfig::schema::episodes::username;

        let mut query = ep_table.filter(username.eq(username1)).into_boxed();

        println!("Since {}", since_date.unwrap());
        if let Some(since_date) = since_date {
            query = query.filter(timestamp.ge(since_date));
        }

        if let Some(device) = opt_device {
            // Always sync the webview
            query = query.filter(
                ep_dsl::device
                    .eq(device)
                    .or(ep_dsl::device.eq(DEFAULT_DEVICE)),
            );
        } else {
            query = query.filter(ep_dsl::device.eq(DEFAULT_DEVICE))
        }

        if let Some(podcast) = opt_podcast {
            query = query.filter(ep_dsl::podcast.eq(podcast));
        }


        query
            .load::<Episode>(&mut get_connection())
            .expect("Error querying episodes")
    }

    pub fn delete_by_username_and_episode(
        username1: &str,
    ) -> Result<(), Error> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::episodes;
        use crate::adapters::persistence::dbconfig::schema::episodes::username;
        diesel::delete(episodes.filter(username.eq(username1)))
            .execute(&mut get_connection())
            .expect("");
        Ok(())
    }

    pub fn get_watchlog_by_device_and_episode(
        episode_guid: String,
        device_id: String,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;

        ep_table
            .filter(ep_dsl::device.eq(device_id))
            .filter(ep_dsl::guid.eq(episode_guid))
            .first::<Episode>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
    }


    pub fn log_watchtime(
        podcast_episode_id: &str,
        time: i32,
        username: &str,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::table as pe_table;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl as p_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcasts::table as p_table;

        let found_episode = pe_table
            .filter(pe_dsl::episode_id.eq(podcast_episode_id))
            .first::<PodcastEpisodeEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if found_episode.clone().is_none() {
            return Err(CustomError::NotFound);
        }
        let found_episode = found_episode.unwrap();

        let podcast = p_table
            .filter(p_dsl::id.eq(found_episode.podcast_id))
            .first::<PodcastEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        if podcast.is_none() {
            return Err(CustomError::NotFound);
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();

        let id = rng.gen_range(0..1000000);

        match Self::get_watchlog_by_device_and_episode(
            found_episode.guid.clone(),
            DEFAULT_DEVICE.to_string(),
        ) {
            Ok(Some(mut episode)) => {
                episode.position = Some(time);
                diesel::update(episodes::table.filter(episodes::id.eq(episode.id)))
                    .set((
                        episodes::position.eq(time),
                        episodes::timestamp.eq(Utc::now().naive_utc()),
                    ))
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
                Ok(())
            }
            Ok(None) => {
                let naive_date = Utc::now().naive_utc();
                let episode = Episode {
                    id,
                    username: username.to_string(),
                    device: DEFAULT_DEVICE.to_string(),
                    podcast: podcast.unwrap().rssfeed,
                    episode: found_episode.url.clone(),
                    timestamp: naive_date,
                    guid: Some(found_episode.guid.clone()),
                    action: "play".to_string(),
                    started: None,
                    position: Some(time),
                    total: Some(found_episode.total_time),
                };
                Self::insert_episode(episode).map(|_| ())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub fn find_episodes_not_in_webview() -> Result<Vec<GPodderAvailablePodcasts>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::episodes;
        use crate::adapters::persistence::dbconfig::schema::episodes::device;
        use crate::adapters::persistence::dbconfig::schema::episodes::podcast;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::rssfeed;

        let result = DistinctDsl::distinct(episodes
            .left_join(podcasts.on(podcast.eq(rssfeed)))
            .select((device, podcast))
            .filter(rssfeed.is_null()))
            .filter(device.ne("webview"))
            .load::<GPodderAvailablePodcasts>(&mut get_connection())
            .map_err(map_db_error)?;

        Ok(result)
    }

    pub fn delete_watchtime(podcast_id: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl as podcast_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcasts::table as podcast_table;

        let found_podcast: Option<Podcast> = podcast_table
            .filter(podcast_dsl::id.eq(podcast_id))
            .first(&mut get_connection())
            .optional()
            .map_err(map_db_error)?;

        diesel::delete(ep_table.filter(ep_dsl::podcast.eq(found_podcast.unwrap().rssfeed)))
            .execute(&mut get_connection())
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_watch_log_by_username_and_episode(
        username1: String,
        conn: &mut crate::adapters::persistence::dbconfig::DBType,
        episode_1: String,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts::table as podcast_episode_table;
        use diesel::JoinOnDsl;
        use diesel::Table;

        episodes
            .inner_join(podcast_episode_table.on(podcast.eq(rssfeed)))
            .filter(username.eq(username1))
            .filter(episode.eq(episode_1))
            .order_by(timestamp.desc())
            .select(episodes::all_columns())
            .first::<EpisodeEntity>(conn)
            .optional()
            .map_err(map_db_error)
            .map(|e| e.map(|e| e.into()))
    }

    pub fn get_last_watched_episodes(
        username_to_find: String,
    ) -> Result<Vec<(PodcastEpisode, Episode, Podcast)>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as e_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::guid as pguid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::adapters::persistence::dbconfig::schema::podcasts as podcast_table;
        use diesel::JoinOnDsl;

        let (episodes1, episodes2) = diesel::alias!(episodes as p1, episodes as p2);

        // Always get the latest available
        let subquery = episodes2
            .select(diesel::dsl::max(episodes2.field(ep_dsl::timestamp)))
            .filter(
                episodes2
                    .field(ep_dsl::episode)
                    .eq(episodes2.field(ep_dsl::episode)),
            )
            .filter(
                episodes2
                    .field(ep_dsl::username)
                    .eq(username_to_find.clone()),
            )
            .group_by(episodes2.field(ep_dsl::episode));

        let query = podcast_episodes
            .inner_join(episodes1.on(pguid.nullable().eq(episodes1.field(eguid))))
            .inner_join(podcast_table::table.on(podcast_table::id.eq(podcast_id)))
            .filter(episodes1.field(e_username).eq(username_to_find.clone()))
            .filter(
                episodes1
                    .field(ep_dsl::timestamp)
                    .nullable()
                    .eq_any(subquery),
            )
            .filter(episodes1.field(ep_dsl::action).eq("play"))
            .load::<(PodcastEpisode, EpisodeEntity, Podcast)>(&mut get_connection())
            .map_err(map_db_error)?;

        let mapped_watched_episodes = query
            .iter()
            .map(|e| (e.0.clone(), e.1.clone().into(), e.2.clone()))
            .collect();
        Ok(mapped_watched_episodes)
    }
}