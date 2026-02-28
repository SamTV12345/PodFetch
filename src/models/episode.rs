use crate::DBType as DbConnection;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::episodes;
use crate::adapters::persistence::dbconfig::schema::episodes::dsl::episodes as episodes_dsl;
use crate::constants::inner_constants::DEFAULT_DEVICE;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::gpodder_available_podcasts::GPodderAvailablePodcasts;
use crate::models::listening_event::{ListeningEvent, NewListeningEvent};
use crate::models::misc_models::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::error::ErrorSeverity::{Critical, Warning};
use crate::utils::error::{CustomError, CustomErrorInner, map_db_error};
use chrono::{NaiveDateTime, Utc};
use diesel::query_dsl::methods::DistinctDsl;
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::{
    BoolExpressionMethods, Insertable, NullableExpressionMethods, OptionalExtension, QueryDsl,
    QueryId, Queryable, QueryableByName, RunQueryDsl, Selectable,
};
use diesel::{ExpressionMethods, JoinOnDsl};
use rand::RngExt;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Error;
use utoipa::ToSchema;

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

const LISTENING_DELTA_GRACE_SECONDS: i64 = 15;

impl Episode {
    pub fn insert_episode(&self) -> Result<Episode, diesel::result::Error> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::*;

        let res = episodes
            .filter(
                timestamp
                    .eq(self.clone().timestamp)
                    .and(device.eq(&self.clone().device))
                    .and(podcast.eq(&self.clone().podcast))
                    .and(episode.eq(&self.clone().episode))
                    .and(timestamp.eq(self.clone().timestamp)),
            )
            .first::<Episode>(&mut get_connection())
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
            .get_result(&mut get_connection())
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
        username1: &str,
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
        }

        if let Some(podcast) = opt_podcast {
            query = query.filter(ep_dsl::podcast.eq(podcast));
        }

        query
            .order_by(timestamp.desc())
            .load::<Episode>(&mut get_connection())
            .expect("Error querying episodes")
    }

    pub fn get_watch_log_by_username_and_episode(
        username1: String,
        conn: &mut DbConnection,
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
            .first::<Episode>(conn)
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    pub fn get_last_watched_episodes(
        user: &User,
    ) -> Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::guid as eguid;
        use crate::adapters::persistence::dbconfig::schema::episodes::username as e_username;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::guid as pguid;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl::*;

        use crate::adapters::persistence::dbconfig::schema::podcasts as podcast_table;
        use diesel::JoinOnDsl;

        let (episodes1, episodes2) = diesel::alias!(episodes as p1, episodes as p2);

        let username_to_find = user.username.clone();
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
            .order_by(episodes1.field(ep_dsl::timestamp).desc())
            .load::<(PodcastEpisode, Episode, Podcast)>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        let mapped_watched_episodes = query
            .iter()
            .map(|e| PodcastWatchedEpisodeModelWithPodcastEpisode {
                episode: e.1.clone().convert_to_episode_dto(),
                podcast_episode: (
                    e.0.clone(),
                    Some(user).cloned(),
                    None::<FavoritePodcastEpisode>,
                )
                    .into(),
                podcast: e.2.clone().into(),
            })
            .collect();
        Ok(mapped_watched_episodes)
    }

    pub fn delete_by_username_and_episode(
        username1: &str,
        conn: &mut DbConnection,
    ) -> Result<(), Error> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl::episodes;
        use crate::adapters::persistence::dbconfig::schema::episodes::username;
        use crate::adapters::persistence::dbconfig::schema::listening_events::dsl as le_dsl;
        use crate::adapters::persistence::dbconfig::schema::listening_events::table as le_table;

        diesel::delete(le_table.filter(le_dsl::username.eq(username1)))
            .execute(conn)
            .expect("");
        diesel::delete(episodes.filter(username.eq(username1)))
            .execute(conn)
            .expect("");
        Ok(())
    }

    pub fn find_episodes_not_in_webview() -> Result<Vec<GPodderAvailablePodcasts>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::podcasts;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl::rssfeed;
        use crate::adapters::persistence::dbconfig::schema::subscriptions::device;
        use crate::adapters::persistence::dbconfig::schema::subscriptions::dsl::subscriptions;
        use crate::adapters::persistence::dbconfig::schema::subscriptions::podcast;

        let result = DistinctDsl::distinct(
            subscriptions
                .left_join(podcasts.on(podcast.eq(rssfeed)))
                .select((device, podcast))
                .filter(rssfeed.is_null()),
        )
        .filter(device.ne("webview"))
        .load::<GPodderAvailablePodcasts>(&mut get_connection())
        .map_err(|e| map_db_error(e, Critical))?;

        Ok(result)
    }

    pub fn delete_watchtime(podcast_id: i32) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;
        use crate::adapters::persistence::dbconfig::schema::listening_events::dsl as le_dsl;
        use crate::adapters::persistence::dbconfig::schema::listening_events::table as le_table;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl as podcast_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcasts::table as podcast_table;

        let found_podcast: Option<Podcast> = podcast_table
            .filter(podcast_dsl::id.eq(podcast_id))
            .first(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        diesel::delete(le_table.filter(le_dsl::podcast_id.eq(podcast_id)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        diesel::delete(ep_table.filter(ep_dsl::podcast.eq(found_podcast.unwrap().rssfeed)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }

    pub fn get_watchtime(episode_id: &str, username: &str) -> Result<Option<Episode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::table as pe_table;

        let podcast_episode = pe_table
            .filter(pe_dsl::episode_id.eq(episode_id))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if podcast_episode.is_none() {
            return Ok(None);
        }

        let episode = ep_table
            .filter(ep_dsl::username.eq(username))
            .filter(ep_dsl::guid.eq(podcast_episode.unwrap().guid))
            .order_by(ep_dsl::timestamp.desc())
            .first::<Episode>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        Ok(episode)
    }

    pub fn log_watchtime(
        pod_watch_model: PodcastWatchedPostModel,
        username: String,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcast_episodes::table as pe_table;
        use crate::adapters::persistence::dbconfig::schema::podcasts::dsl as p_dsl;
        use crate::adapters::persistence::dbconfig::schema::podcasts::table as p_table;

        let found_episode = pe_table
            .filter(pe_dsl::episode_id.eq(pod_watch_model.podcast_episode_id.clone()))
            .first::<PodcastEpisode>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if found_episode.clone().is_none() {
            return Err(CustomErrorInner::NotFound(Warning).into());
        }
        let found_episode = found_episode.unwrap();

        let podcast = p_table
            .filter(p_dsl::id.eq(found_episode.podcast_id))
            .first::<Podcast>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        if podcast.is_none() {
            return Err(CustomErrorInner::NotFound(Warning).into());
        }

        let now = Utc::now().naive_utc();

        match Self::get_watchlog_by_device_and_episode(
            &username,
            &found_episode.guid,
            DEFAULT_DEVICE,
        ) {
            Ok(Some(mut episode)) => {
                let listened_delta_seconds = Self::calculate_listened_delta(
                    episode.position,
                    Some(episode.timestamp),
                    pod_watch_model.time,
                    now,
                );
                if listened_delta_seconds > 0 {
                    ListeningEvent::insert_event(NewListeningEvent {
                        username: username.clone(),
                        device: DEFAULT_DEVICE.to_string(),
                        podcast_episode_id: found_episode.episode_id.clone(),
                        podcast_id: found_episode.podcast_id,
                        podcast_episode_db_id: found_episode.id,
                        delta_seconds: listened_delta_seconds,
                        start_position: pod_watch_model.time.saturating_sub(listened_delta_seconds),
                        end_position: pod_watch_model.time,
                        listened_at: now,
                    })?;
                }

                episode.position = Some(pod_watch_model.time);
                diesel::update(episodes_dsl.filter(episodes::id.eq(episode.id)))
                    .set((
                        episodes::started.eq(pod_watch_model.time),
                        episodes::position.eq(pod_watch_model.time),
                        episodes::timestamp.eq(now),
                    ))
                    .execute(&mut get_connection())
                    .map_err(|e| map_db_error(e, Critical))?;
                return Ok(());
            }
            Ok(None) => {
                let mut rng = rand::rng();
                let id = rng.random_range(0..1000000);
                let episode = Episode {
                    id,
                    username,
                    device: DEFAULT_DEVICE.to_string(),
                    podcast: podcast.unwrap().rssfeed,
                    episode: found_episode.url.clone(),
                    timestamp: now,
                    guid: Some(found_episode.guid.clone()),
                    action: "play".to_string(),
                    started: Some(pod_watch_model.time),
                    position: Some(pod_watch_model.time),
                    total: Some(found_episode.total_time),
                };
                episode
                    .insert_episode()
                    .map_err(|e| map_db_error(e, Critical))?;
            }
            Err(e) => {
                return Err(e);
            }
        }

        Ok(())
    }

    pub fn get_watchlog_by_device_and_episode(
        username_to_find: &str,
        episode_guid: &str,
        device_id: &str,
    ) -> Result<Option<Episode>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;

        ep_table
            .filter(ep_dsl::username.eq(username_to_find))
            .filter(ep_dsl::device.eq(device_id))
            .filter(ep_dsl::guid.eq(episode_guid))
            .order_by(ep_dsl::timestamp.desc())
            .first::<Episode>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))
    }

    fn calculate_listened_delta(
        previous_position: Option<i32>,
        previous_timestamp: Option<NaiveDateTime>,
        current_position: i32,
        now: NaiveDateTime,
    ) -> i32 {
        let Some(previous_position) = previous_position else {
            return 0;
        };

        if current_position <= previous_position {
            return 0;
        }

        let raw_delta = current_position.saturating_sub(previous_position);
        let Some(previous_timestamp) = previous_timestamp else {
            return raw_delta;
        };

        let elapsed = now
            .signed_duration_since(previous_timestamp)
            .num_seconds()
            .max(0);
        let max_allowed = elapsed.saturating_add(LISTENING_DELTA_GRACE_SECONDS);
        raw_delta.min(max_allowed.min(i64::from(i32::MAX)) as i32)
    }

    pub fn delete_by_username(username: &str) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::episodes::dsl as ep_dsl;
        use crate::adapters::persistence::dbconfig::schema::episodes::table as ep_table;
        use crate::adapters::persistence::dbconfig::schema::listening_events::dsl as le_dsl;
        use crate::adapters::persistence::dbconfig::schema::listening_events::table as le_table;

        diesel::delete(le_table.filter(le_dsl::username.eq(username)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        diesel::delete(ep_table.filter(ep_dsl::username.eq(username)))
            .execute(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
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

#[derive(Serialize, Deserialize, Debug, ToSchema)]
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
            _ => panic!("Unknown episode action: {s}"),
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

#[cfg(test)]
mod tests {
    use super::Episode;
    use chrono::{NaiveDate, NaiveDateTime};

    fn dt(hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, 27)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    #[test]
    fn calculate_listened_delta_returns_zero_without_previous_position() {
        let delta = Episode::calculate_listened_delta(None, None, 10, dt(10, 0, 0));
        assert_eq!(delta, 0);
    }

    #[test]
    fn calculate_listened_delta_returns_zero_for_seek_back() {
        let delta =
            Episode::calculate_listened_delta(Some(30), Some(dt(10, 0, 0)), 20, dt(10, 0, 4));
        assert_eq!(delta, 0);
    }

    #[test]
    fn calculate_listened_delta_caps_large_forward_seek() {
        let delta =
            Episode::calculate_listened_delta(Some(10), Some(dt(10, 0, 0)), 400, dt(10, 0, 5));
        assert_eq!(delta, 20);
    }

    #[test]
    fn calculate_listened_delta_keeps_regular_progress() {
        let delta =
            Episode::calculate_listened_delta(Some(100), Some(dt(10, 0, 0)), 108, dt(10, 0, 3));
        assert_eq!(delta, 8);
    }
}
