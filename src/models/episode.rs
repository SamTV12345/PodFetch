use std::fmt;
use std::io::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, QueryableByName, Insertable, RunQueryDsl, QueryDsl, BoolExpressionMethods, OptionalExtension, TextExpressionMethods};

use crate::dbconfig::schema::episodes;
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use diesel::ExpressionMethods;
use reqwest::Url;

use crate::{DbConnection};
use crate::dbconfig::schema::episodes::dsl::episodes as episodes_dsl;

use crate::models::misc_models::PodcastWatchedEpisodeModelWithPodcastEpisode;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_history_item::PodcastHistoryItem;
use crate::models::podcasts::Podcast;
use crate::utils::error::{CustomError, map_db_error};

#[derive(Serialize, Deserialize, Debug,Queryable, QueryableByName,Insertable, Clone, ToSchema)]
pub struct Episode{
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
    pub started:Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub position:Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub total:Option<i32>,
    #[diesel(sql_type = Text)]
    pub cleaned_url: String
}


impl Episode{
    pub fn insert_episode(&self, conn: &mut DbConnection) -> Result<Episode, diesel::result::Error> {
        use crate::dbconfig::schema::episodes::dsl::*;

        let res = episodes.filter(timestamp.eq(self.clone().timestamp)
            .and(device.eq(&self.clone().device))
            .and(podcast.eq(&self.clone().podcast))
            .and(episode.eq(&self.clone().episode))
            .and(timestamp.eq(self.clone().timestamp)))
            .first::<Episode>(conn)
            .optional()
            .expect("");

        if let Some(unwrapped_res) = res {
            return Ok(unwrapped_res)
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
                cleaned_url.eq(&cleaned_url_parsed.to_string()),
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

    pub fn convert_to_episode(episode_dto: &EpisodeDto, username: String)->Episode{
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
            cleaned_url: episode.to_string(),
        }
    }
    pub async fn get_actions_by_username(username1: String, conn: &mut DbConnection, since_date: Option<NaiveDateTime>) ->Vec<Episode>{
        use crate::dbconfig::schema::episodes::username;
        use crate::dbconfig::schema::episodes::dsl::episodes;
        use crate::dbconfig::schema::episodes::dsl::timestamp;
        match since_date {
            Some(e)=>{
                episodes
                    .filter(username.eq(username1))
                    .filter(timestamp.gt(e))
                    .load::<Episode>(conn)
                    .expect("")
            },
            None=>{
                episodes
                    .filter(username.eq(username1))
                    .load::<Episode>(conn)
                    .expect("")
            }
        }
    }

    pub fn get_watch_log_by_username_and_episode(username1: String, conn: &mut DbConnection,
                                                 episode_1: String) ->Result<Option<Episode>,
        CustomError> {

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

    pub fn convert_to_podcast_history_item(&self, podcast_1: Podcast,pod_episode: PodcastEpisode)
        ->
                                                                              PodcastHistoryItem {
        PodcastHistoryItem {
            id: self.id,
            podcast_id: podcast_1.id,
            episode_id: pod_episode.episode_id,
            watched_time: self.position.unwrap(),
            date: self.timestamp,
            username: self.username.clone(),
        }
    }

    pub fn get_last_watched_episodes(username_to_find: String, conn: &mut DbConnection)
                                     ->Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>,
                                         CustomError>{
        use crate::dbconfig::schema::episodes::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use crate::dbconfig::schema::podcasts as podcast_table;
        use diesel::JoinOnDsl;

        let query = podcast_episodes
            .inner_join(episodes.on(url.like(cleaned_url.concat("%"))))
            .inner_join(podcast_table::table.on(podcast_table::id.eq(podcast_id)))
            .filter(username.eq(username_to_find.clone()))
            .load::<(PodcastEpisode,Episode, Podcast)>(conn)
            .map_err(map_db_error)?;

        let _query_1 = &podcast_episodes
            .inner_join(episodes.on(url.like(cleaned_url.concat("%"))))
            .inner_join(podcast_table::table.on(podcast_table::id.eq(podcast_id)))
            .filter(username.eq(username_to_find));

        let mapped_watched_episodes = query.iter().map(|e|{
            PodcastWatchedEpisodeModelWithPodcastEpisode{
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
            }
        }).collect();
        Ok(mapped_watched_episodes)
    }

    pub fn delete_by_username_and_episode(username1: String, conn: &mut DbConnection) ->Result<(),Error>{
        use crate::dbconfig::schema::episodes::username;
        use crate::dbconfig::schema::episodes::dsl::episodes;
        diesel::delete(episodes.filter(username.eq(username1)))
                                   .execute(conn).expect("");
        Ok(())
    }

    pub fn migrate_episode_urls(conn: &mut DbConnection){
        let episodes_loaded = episodes_dsl
            .load::<Episode>(conn)
            .expect("");
        episodes_loaded.iter().for_each(|e|{
            let mut cleaned_url_parsed = Url::parse(&e.episode).unwrap();
            cleaned_url_parsed.set_query(None);
            diesel::update(
                episodes_dsl.filter(episodes::id.eq(e.id)))
                .set(episodes::cleaned_url.eq(cleaned_url_parsed.to_string()))
                .execute(conn).expect("");
        });
        }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EpisodeDto {
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: EpisodeAction,
    pub started:Option<i32>,
    pub position:Option<i32>,
    pub total:Option<i32>,
    pub device: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum EpisodeAction {
    New,
    Download,
    Play,
    Delete,
}


impl EpisodeAction{
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