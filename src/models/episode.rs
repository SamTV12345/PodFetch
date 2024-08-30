use crate::constants::inner_constants::DEFAULT_DEVICE;
use crate::dbconfig::schema::episodes;
use crate::dbconfig::schema::episodes::dsl::episodes as episodes_dsl;
use crate::DBType as DbConnection;
use chrono::{NaiveDateTime, Utc};
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::{ExpressionMethods, JoinOnDsl};
use diesel::{
    BoolExpressionMethods, Insertable, NullableExpressionMethods, OptionalExtension, QueryDsl,
    QueryId, Queryable, QueryableByName, RunQueryDsl, Selectable,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Error;
use diesel::query_dsl::methods::DistinctDsl;
use utoipa::ToSchema;
use crate::models::gpodder_available_podcasts::GPodderAvailablePodcasts;
use crate::models::misc_models::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::utils::error::{map_db_error, CustomError};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Queryable,
    QueryableByName,
    Insertable,
    Clone,
    Selectable,
    ToSchema,
    QueryId,
)]
pub struct Episode {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
    #[diesel(sql_type = Text)]
    pub episode: String,
    #[diesel(sql_type = Timestamp)]
    pub timestamp: NaiveDateTime,
    #[diesel(sql_type = Nullable<Text>)]
    pub guid: Option<String>,
    #[diesel(sql_type = Text)]
    pub action: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub started: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub position: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub total: Option<i32>,
}

impl Episode {
    pub fn insert_episode(
        &self,
        conn: &mut DbConnection,
    ) -> Result<Episode, diesel::result::Error> {
        use crate::dbconfig::schema::episodes::dsl::*;

        let res = episodes
            .filter(
                timestamp
                    .eq(self.clone().timestamp)
                    .and(device.eq(&self.clone().device))
                    .and(podcast.eq(&self.clone().podcast))
                    .and(episode.eq(&self.clone().episode))
                    .and(timestamp.eq(self.clone().timestamp)),
            )
            .first::<Episode>(conn)
            .optional()
            .expect("");

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res);
        }

        let mut cleaned_url_parsed = Url::parse(&self.episode).unwrap();
        cleaned_url_parsed.set_query(None);
        diesel::insert_into(episodes)
            .values((
                username.eq(&self.username),
                device.eq(&self.device),
                podcast.eq(&self.podcast),
                episode.eq(&self.episode),
                timestamp.eq(&self.timestamp),
                guid.eq(&self.guid),
                action.eq(&self.action),
                started.eq(&self.started),
                position.eq(&self.position),
                total.eq(&self.total),
            ))
            .get_result(conn)
    }

    pub fn convert_to_episode_dto(&self) -> EpisodeDto {
        EpisodeDto {
            podcast: self.podcast.clone(),
            episode: self.episode.clone(),
            timestamp: self.timestamp,
            guid: self.guid.clone(),
            action: EpisodeAction::from_string(&self.action),
            started: self.started,
            position: self.position,
            total: self.total,
            device: self.clone().device.clone(),
        }
    }

    pub fn convert_to_episode(episode_dto: &EpisodeDto, username: String) -> Episode {
        // Remove query parameters
        let mut episode = Url::parse(&episode_dto.episode).unwrap();
        episode.set_query(None);

        Episode {
            id: 0,
            username,
            device: episode_dto.device.clone(),
            podcast: episode_dto.podcast.clone(),
            episode: episode_dto.episode.clone(),
            timestamp: episode_dto.timestamp,
            guid: episode_dto.guid.clone(),
            action: episode_dto.action.clone().to_string(),
            started: episode_dto.started,
            position: episode_dto.position,
            total: episode_dto.total,
        }
    }
    pub async fn get_actions_by_username(
        username1: String,
        conn: &mut DbConnection,
        since_date: Option<NaiveDateTime>,
        opt_device: Option<String>,
        _opt_aggregate: Option<String>,
        opt_podcast: Option<String>,
    ) -> Vec<Episode> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::dsl::timestamp;
        use crate::dbconfig::schema::episodes::table as ep_table;
        use crate::dbconfig::schema::episodes::username;

        let mut query = ep_table.filter(username.eq(username1)).into_boxed();

        if let Some(since_date) = since_date {
            query = query.filter(timestamp.gt(since_date));
        }

        if let Some(device) = opt_device {
            // Always sync the webview
            query = query.filter(
                ep_dsl::device
                    .eq(device)
                    .or(ep_dsl::device.eq(DEFAULT_DEVICE)),
            );
        }

        if let Some(podcast) = opt_podcast {
            query = query.filter(ep_dsl::podcast.eq(podcast));
        }

        query
            .load::<Episode>(conn)
            .expect("Error querying episodes")
    }

    pub fn get_watch_log_by_username_and_episode(
        username1: String,
        conn: &mut DbConnection,
        episode_1: String,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::dbconfig::schema::episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcasts::table as podcast_episode_table;
        use diesel::JoinOnDsl;
        use diesel::Table;

        episodes
            .inner_join(podcast_episode_table.on(podcast.eq(rssfeed)))
            .filter(username.eq(username1))
            .filter(episode.eq(episode_1))
            .order_by(timestamp.desc())
            .select(episodes::all_columns())
            .first::<Episode>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn get_last_watched_episodes(
        username_to_find: String,
        conn: &mut DbConnection,
    ) -> Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>, CustomError> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::dsl::guid as eguid;
        use crate::dbconfig::schema::episodes::username as e_username;
        use crate::dbconfig::schema::podcast_episodes::dsl::guid as pguid;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts as podcast_table;
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
            .load::<(PodcastEpisode, Episode, Podcast)>(conn)
            .map_err(map_db_error)?;

        let mapped_watched_episodes = query
            .iter()
            .map(|e| PodcastWatchedEpisodeModelWithPodcastEpisode {
                id: e.clone().1.id,
                podcast_id: e.clone().2.id,
                episode_id: e.0.episode_id.clone(),
                url: e.0.url.clone(),
                name: e.0.name.clone(),
                image_url: e.0.image_url.clone(),
                watched_time: e.clone().1.position.unwrap(),
                date: e.clone().1.timestamp,
                total_time: e.clone().0.total_time,
                podcast_episode: e.0.clone(),
                podcast: e.2.clone(),
            })
            .collect();
        Ok(mapped_watched_episodes)
    }

    pub fn delete_by_username_and_episode(
        username1: &str,
        conn: &mut DbConnection,
    ) -> Result<(), Error> {
        use crate::dbconfig::schema::episodes::dsl::episodes;
        use crate::dbconfig::schema::episodes::username;
        diesel::delete(episodes.filter(username.eq(username1)))
            .execute(conn)
            .expect("");
        Ok(())
    }


    pub fn find_episodes_not_in_webview(conn: &mut DbConnection) -> Result<Vec<GPodderAvailablePodcasts>, CustomError> {
        use crate::dbconfig::schema::episodes::dsl::episodes;
        use crate::dbconfig::schema::episodes::device;
        use crate::dbconfig::schema::episodes::podcast;
        use crate::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::dbconfig::schema::podcasts::dsl::rssfeed;

        let result = DistinctDsl::distinct(episodes
            .left_join(podcasts.on(podcast.eq(rssfeed)))
            .select((device, podcast))
            .filter(rssfeed.is_null()))
             .filter(device.ne("webview"))
            .load::<GPodderAvailablePodcasts>(conn)
            .map_err(map_db_error)?;


        Ok(result)
    }

    pub fn delete_watchtime(conn: &mut DbConnection, podcast_id: i32) -> Result<(), CustomError> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::table as ep_table;
        use crate::dbconfig::schema::podcasts::dsl as podcast_dsl;
        use crate::dbconfig::schema::podcasts::table as podcast_table;

        let found_podcast: Option<Podcast> = podcast_table
            .filter(podcast_dsl::id.eq(podcast_id))
            .first(conn)
            .optional()
            .map_err(map_db_error)?;

        diesel::delete(ep_table.filter(ep_dsl::podcast.eq(found_podcast.unwrap().rssfeed)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }

    pub fn get_watchtime(
        conn: &mut DbConnection,
        episode_id: String,
        username: String,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::table as ep_table;
        use crate::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
        use crate::dbconfig::schema::podcast_episodes::table as pe_table;

        let podcast_episode = pe_table
            .filter(pe_dsl::episode_id.eq(episode_id))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        let episode = ep_table
            .filter(ep_dsl::username.eq(username))
            .filter(ep_dsl::guid.eq(podcast_episode.unwrap().guid))
            .first::<Episode>(conn)
            .optional()
            .map_err(map_db_error)?;

        Ok(episode)
    }

    pub fn log_watchtime(
        conn: &mut DbConnection,
        pod_watch_model: PodcastWatchedPostModel,
        username: String,
    ) -> Result<(), CustomError> {
        use crate::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
        use crate::dbconfig::schema::podcast_episodes::table as pe_table;
        use crate::dbconfig::schema::podcasts::dsl as p_dsl;
        use crate::dbconfig::schema::podcasts::table as p_table;

        let found_episode = pe_table
            .filter(pe_dsl::episode_id.eq(pod_watch_model.podcast_episode_id.clone()))
            .first::<PodcastEpisode>(conn)
            .optional()
            .map_err(map_db_error)?;

        if found_episode.clone().is_none() {
            return Err(CustomError::NotFound);
        }
        let found_episode = found_episode.unwrap();

        let podcast = p_table
            .filter(p_dsl::id.eq(found_episode.podcast_id))
            .first::<Podcast>(conn)
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
            conn,
        ) {
            Ok(Some(mut episode)) => {
                episode.position = Some(pod_watch_model.time);
                diesel::update(episodes_dsl.filter(episodes::id.eq(episode.id)))
                    .set((
                        episodes::position.eq(pod_watch_model.time),
                        episodes::timestamp.eq(Utc::now().naive_utc()),
                    ))
                    .execute(conn)
                    .map_err(map_db_error)?;
                return Ok(());
            }
            Ok(None) => {
                let episode = Episode {
                    id,
                    username,
                    device: DEFAULT_DEVICE.to_string(),
                    podcast: podcast.unwrap().rssfeed,
                    episode: found_episode.url.clone(),
                    timestamp: Default::default(),
                    guid: Some(found_episode.guid.clone()),
                    action: "play".to_string(),
                    started: None,
                    position: Some(pod_watch_model.time),
                    total: Some(found_episode.total_time),
                };
                episode.insert_episode(conn).map_err(map_db_error)?;
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }

    pub fn get_watchlog_by_device_and_episode(
        episode_guid: String,
        device_id: String,
        conn: &mut DbConnection,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::table as ep_table;

        ep_table
            .filter(ep_dsl::device.eq(device_id))
            .filter(ep_dsl::guid.eq(episode_guid))
            .first::<Episode>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn delete_by_username(conn: &mut DbConnection, username: &str) -> Result<(), CustomError> {
        use crate::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::dbconfig::schema::episodes::table as ep_table;

        diesel::delete(ep_table.filter(ep_dsl::username.eq(username)))
            .execute(conn)
            .map_err(map_db_error)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EpisodeDto {
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: EpisodeAction,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
    pub device: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq, Clone)]
pub enum EpisodeAction {
    New,
    Download,
    Play,
    Delete,
}

impl EpisodeAction {
    pub fn from_string(s: &str) -> Self {
        match s {
            "new" => EpisodeAction::New,
            "download" => EpisodeAction::Download,
            "play" => EpisodeAction::Play,
            "delete" => EpisodeAction::Delete,
            _ => panic!("Unknown episode action: {}", s),
        }
    }
}

impl fmt::Display for EpisodeAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EpisodeAction::New => write!(f, "new"),
            EpisodeAction::Download => write!(f, "download"),
            EpisodeAction::Play => write!(f, "play"),
            EpisodeAction::Delete => write!(f, "delete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeActionRaw {
    New,
    Download,
    Play,
    Delete,
}
