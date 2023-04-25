use std::ops::Deref;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use diesel::{Queryable, QueryableByName, Insertable, SqliteConnection, RunQueryDsl, QueryDsl};
use crate::schema::episodes;
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use diesel::ExpressionMethods;

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

#[serde(rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug)]
pub enum EpisodeActionRaw {
    New,
    Download,
    Play,
    Delete,
}