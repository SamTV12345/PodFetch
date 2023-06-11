use std::collections::HashMap;
use std::io::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, QueryableByName, Insertable, RunQueryDsl, QueryDsl, BoolExpressionMethods, OptionalExtension, sql_query};
use diesel::dsl::max;
use crate::dbconfig::schema::episodes;
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use diesel::ExpressionMethods;
use diesel::query_builder::QueryBuilder;
use crate::db::DB;
use crate::{DbConnection, MyQueryBuilder};
use crate::dbconfig::schema::podcast_episodes::dsl::podcast_episodes;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::models::models::{PodcastHistoryItem, PodcastWatchedEpisodeModelWithPodcastEpisode};

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
}


impl Episode{
    pub fn insert_episode(&self, conn: &mut DbConnection) -> Result<Episode, diesel::result::Error> {
        use crate::dbconfig::schema::episodes::dsl::*;

        let res = episodes.filter(timestamp.eq(self.clone().timestamp)
            .and(device.eq(self.clone().device))
            .and(podcast.eq(self.clone().podcast))
            .and(episode.eq(self.clone().episode))
            .and(timestamp.eq(self.clone().timestamp)))
            .first::<Episode>(conn)
            .optional()
            .expect("");

        if res.is_some() {
            return Ok(res.unwrap())
        }

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
                total.eq(&self.total)
                ))
            .get_result(conn)
    }

    pub fn convert_to_episode_dto(&self) -> EpisodeDto {
        EpisodeDto {
            podcast: self.podcast.clone(),
            episode: self.episode.clone(),
            timestamp: self.timestamp.clone(),
            guid: self.guid.clone(),
            action: EpisodeAction::from_string(&self.action),
            started: self.started,
            position: self.position,
            total: self.total,
            device: self.clone().device,
        }
    }

    pub fn convert_to_episode(episode_dto: &EpisodeDto, username: String)->Episode{
        Episode {
            id: 0,
            username,
            device: episode_dto.device.clone(),
            podcast: episode_dto.podcast.clone(),
            episode: episode_dto.episode.clone(),
            timestamp: episode_dto.timestamp.clone(),
            guid: episode_dto.guid.clone(),
            action: episode_dto.action.clone().to_string(),
            started: episode_dto.started,
            position: episode_dto.position,
            total: episode_dto.total,
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
                                                 episode_1: String) ->Option<Episode>{

        use crate::dbconfig::schema::episodes::dsl::*;
        use crate::dbconfig::schema::podcasts::dsl::*;
        use crate::dbconfig::schema::podcasts::table as podcast_episode_table;
        use diesel::JoinOnDsl;
        use diesel::Table;

        let query = episodes
            .inner_join(podcast_episode_table.on(podcast.eq(rssfeed)))
            .filter(username.eq(username1))
            .filter(episode.eq(episode_1))
            .order_by(timestamp.desc())
            .select(episodes::all_columns())
            .first::<Episode>(conn)
            .optional()
            .expect("");
        return query;
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

    pub fn get_last_watched_episodes(username1: String, conn: &mut DbConnection)
                                     ->Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>{
        use crate::dbconfig::schema::episodes::dsl::*;
        use crate::dbconfig::schema::podcast_episodes::dsl::*;
        use diesel::JoinOnDsl;
        let mut map:HashMap<String,Podcast> = HashMap::new();

        let query = podcast_episodes
            .inner_join(episodes.on(podcast.eq(url)))
            .filter(username.eq(username1))
            .load::<(PodcastEpisode,Episode)>(conn).expect("ERror loading");


        query.iter().map(|e|{
            let opt_podcast = map.get(&*e.clone().1.podcast);
            if opt_podcast.is_none(){
                let podcast_found = DB::get_podcast_by_rss_feed(e.clone().1.podcast, conn);
                map.insert(e.clone().1.podcast.clone(),podcast_found.clone());
            }
            let found_podcast = map.get(&e.clone().1.podcast).cloned().unwrap();
            PodcastWatchedEpisodeModelWithPodcastEpisode{
                id: e.clone().1.id,
                podcast_id: found_podcast.id,
                episode_id: e.clone().0.episode_id,
                url: e.clone().0.url,
                name: e.clone().0.name,
                image_url: e.clone().0.image_url,
                watched_time: e.clone().1.position.unwrap(),
                date: e.clone().1.timestamp,
                total_time: e.clone().0.total_time,
                podcast_episode: e.clone().0,
                podcast: found_podcast.clone(),
            }
        }).collect()
    }

    pub fn delete_by_username_and_episode(username1: String, conn: &mut DbConnection) ->Result<(),Error>{
        use crate::dbconfig::schema::episodes::username;
        use crate::dbconfig::schema::episodes::dsl::episodes;
        diesel::delete(episodes.filter(username.eq(username1)))
                                   .execute(conn).expect("");
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

    pub fn to_string(&self) -> String {
        match self {
            EpisodeAction::New => "new".to_string(),
            EpisodeAction::Download => "download".to_string(),
            EpisodeAction::Play => "play".to_string(),
            EpisodeAction::Delete => "delete".to_string(),
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