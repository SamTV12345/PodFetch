use std::collections::HashMap;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, QueryableByName, Insertable, SqliteConnection, RunQueryDsl, QueryDsl, BoolExpressionMethods, OptionalExtension, sql_query};
use crate::schema::episodes;
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use diesel::ExpressionMethods;
use crate::db::DB;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
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
    pub fn insert_episode(&self, conn: &mut SqliteConnection) -> Result<Episode, diesel::result::Error> {
        use crate::schema::episodes::dsl::*;

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
    pub async fn get_actions_by_username(username1: String, conn: &mut SqliteConnection, since_date: Option<NaiveDateTime>) ->Vec<Episode>{
        use crate::schema::episodes::username;
        use crate::schema::episodes::dsl::episodes;
        use crate::schema::episodes::dsl::timestamp;
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

    pub fn get_watch_log_by_username_and_episode(username1: String, conn: &mut SqliteConnection,
                                                 episode_1: String) ->Option<Episode>{

        let res = sql_query(
            "SELECT * FROM (SELECT * FROM episodes,podcasts WHERE username=? AND episodes\
            .podcast=podcasts.rssfeed AND episodes.episode = ? ORDER BY timestamp DESC) GROUP BY \
            episode  LIMIT 10;")
            .bind::<Text, _>(username1.clone())
            .bind::<Text,_>(episode_1)
            .load::<Episode>(conn)
            .expect("");
        return if res.len() > 0 {
            Some(res[0].clone())
        } else {
            None
        }
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

    pub fn get_last_watched_episodes(username1: String, conn: &mut SqliteConnection)
        ->Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>{


        let mut map:HashMap<String,Podcast> = HashMap::new();
        let res = sql_query(
            r"SELECT * FROM (SELECT * FROM episodes e, podcast_episodes pe WHERE
            e.username=? AND pe.url=e.episode  ORDER BY timestamp DESC) GROUP BY episode  LIMIT
            10;")
            .bind::<Text, _>(username1.clone())
            .load::<(Episode,PodcastEpisode)>(conn)
            .expect("");

        res.iter().map(|e|{
            let opt_podcast = map.get(&*e.clone().0.podcast);
            if opt_podcast.is_none(){
                let podcast = DB::get_podcast_by_rss_feed(e.clone().0.podcast, conn);
                map.insert(e.clone().0.podcast.clone(),podcast.clone());
            }
            let found_podcast = map.get(&e.clone().0.podcast).cloned().unwrap();
            PodcastWatchedEpisodeModelWithPodcastEpisode{
                id: e.clone().0.id,
                podcast_id: found_podcast.id,
                episode_id: e.clone().1.episode_id,
                url: e.clone().1.url,
                name: e.clone().1.name,
                image_url: e.clone().1.image_url,
                watched_time: e.clone().0.position.unwrap(),
                date: e.clone().0.timestamp,
                total_time: e.clone().1.total_time,
                podcast_episode: e.clone().1,
                podcast: found_podcast.clone(),
            }
        }).collect()
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