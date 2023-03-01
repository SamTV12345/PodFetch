use std::time::SystemTime;
use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use rusqlite::{Connection, params, Result, Statement};
use crate::constants::constants::DB_NAME;
use crate::models::itunes_models::{Podcast, PodcastEpisode};
use crate::models::models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedModel, PodcastWatchedPostModel};
use crate::service::mapping_service::MappingService;


pub struct DB{
    conn: Connection,
    mapping_service: MappingService
}

impl DB{
    pub fn new() -> Result<DB>{
        let conn = Connection::open(DB_NAME)?;
        conn.execute("create table if not exists Podcast (
             id integer primary key,
             name text not null unique,
             directory text not null,
             rssfeed text not null,
             image_url text not null)", []).expect("Error creating table");
        conn.execute("create table if not exists podcast_episodes (
             id integer primary key,
             podcast_id integer not null,
             episode_id TEXT not null,
             name text not null,
             url text not null,
             date text not null,
             image_url text not null,
             total_time integer DEFAULT 0 not null,
             local_url text DEFAULT '' not null,
             local_image_url text DEFAULT '' not null,
             description text DEFAULT '' not null,
             FOREIGN KEY (podcast_id) REFERENCES Podcast(id))", []).expect("Error creating table");
        conn.execute("CREATE INDEX IF NOT EXISTS podcast_episodes_podcast_id_index ON podcast_episodes (podcast_id)", []).expect("Error creating index");
        // status 0 = not downloaded, 1 = downloaded, 2 = error
        conn.execute("create table if not exists queue (
             id integer primary key,
             podcast_id integer not null,
             download_url text not null,
             episode_id TEXT not null,
             status integer not null,
             FOREIGN KEY (podcast_id) REFERENCES Podcast(id),
             FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id))",
                       []).expect("Error creating table");
        conn.execute("CREATE table if not exists Podcast_History (
             id integer primary key,
             podcast_id integer not null,
             episode_id TEXT not null,
             watched_time integer not null,
             date text not null,
             FOREIGN KEY (podcast_id) REFERENCES Podcast(id))",
                       []).expect("Error creating table");
        Ok(DB{conn, mapping_service: MappingService::new()})
    }

    pub fn get_podcasts(&self) -> Result<Vec<Podcast>>{
        let mut stmt = self.conn.prepare("select * from Podcast")?;
        let podcast_iter = stmt.query_map([], |row| {
            Ok(Podcast {
                id: row.get(0)?,
                name: row.get(1)?,
                directory: row.get(2)?,
                rssfeed: row.get(3)?,
                image_url: row.get(4)?
            })
        })?;

        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast?);
        }
        Ok(podcasts)
    }

    pub fn get_podcast(&self, id: i64) -> Result<Podcast>{
        let mut stmt = self.conn.prepare("select * from Podcast where id = ?1")?;
        let podcast_iter = stmt.query_map([&id], |row| {
            Ok(Podcast {
                id: row.get(0)?,
                name: row.get(1)?,
                directory: row.get(2)?,
                rssfeed: row.get(3)?,
                image_url: row.get(4)?
            })
        })?;

        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast?);
        }
        Ok(podcasts[0].clone())
    }

    pub fn get_podcast_episode_by_id(&self, podcast_id: &str) -> Result<Option<PodcastEpisode>>{
        let mut stmt = self.conn.prepare("select * from podcast_episodes where episode_id = ?1")?;
        let mut podcast_iter = stmt.query_map([&podcast_id], |row| {
            Ok(PodcastEpisode {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                name: row.get(3)?,
                url: row.get(4)?,
                date: row.get(5)?,
                image_url: row.get(6)?,
                total_time: row.get(7)?,
                local_url: row.get(8)?,
                local_image_url: row.get(9)?,
                description: row.get(10)?
            })
        })?;

        let iter = podcast_iter.next().map(|podcast| podcast.unwrap());
        Ok(iter)
    }


    pub fn get_podcast_episode_by_track_id(&self, podcast_id: i64) ->
                                                                   Result<Option<Podcast>>{
        let mut stmt = self.conn.prepare("select * from Podcast where directory = ?1")?;
        let mut podcast_iter = stmt.query_map([&podcast_id], |row| {
            Ok(Podcast {
                id: row.get(0)?,
                name: row.get(1)?,
                directory: row.get(2)?,
                rssfeed: row.get(3)?,
                image_url: row.get(4)?
            })
        })?;

        let iter = podcast_iter.next().map(|podcast| podcast.unwrap());
        Ok(iter)
    }

    pub fn insert_podcast_episodes(&self, podcast: Podcast, link: &str, item: &Entry, image_url:
    &str, episode_description: &str){
        self.conn.execute("INSERT INTO podcast_episodes (podcast_id,\
                        episode_id, name, url, date, image_url, description) VALUES (?1,?2, ?3, ?4, \
                        ?5, ?6, ?7)",
                                  (&podcast.id, &item.id, &item.title.as_ref()
                                      .unwrap()
                                      .content,
                                   link, &item.published.unwrap(), image_url, episode_description))
            .expect("Error inserting podcast episode");
    }

    pub fn add_podcast_to_database(&self, collection_name:String, collection_id:String,
                                   feed_url:String, image_url: String){
        self.conn.execute("INSERT INTO Podcast (name, directory, rssfeed, image_url) VALUES (?1, \
        ?2, ?3, ?4)",
                                  [collection_name, collection_id, feed_url, image_url])
            .expect("Error inserting podcast into database");
    }

    pub fn get_last_5_podcast_episodes(&self, podcast_id: i64) -> Result<Vec<PodcastEpisode>>{
        let stmt = self.conn.prepare("select * from podcast_episodes where podcast_id = ?1 \
        order by date(date) desc limit 5")?;
        Ok(Self::extract_podcast_episodes(stmt, podcast_id, ))
    }


    pub fn get_podcast_episodes_of_podcast(&self, podcast_id: i64,  last_id: Option<String>) ->
                                                                      Result<Vec<PodcastEpisode>>{
        let stmt:Statement;
        match last_id {
            Some(last_id) => {
                 stmt = self.conn.prepare("select * from podcast_episodes where podcast_id = ?1 \
        AND date(date) < ?2 \
        order by date(date) desc LIMIT 75")?;
                Ok(Self::extract_statement_with_episode(stmt, podcast_id, last_id))
            }
            None => {
                stmt = self.conn.prepare("select * from podcast_episodes where podcast_id\
                 = ?1 LIMIT 75")?;
                Ok(Self::extract_podcast_episodes(stmt, podcast_id))
            }
        }


    }

    fn extract_podcast_episodes(mut stmt: Statement, podcast_id: i64) -> Vec<PodcastEpisode>  {
        let podcast_iter = stmt.query_map([&podcast_id], |row| {
            Ok(PodcastEpisode {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                name: row.get(3)?,
                url: row.get(4)?,
                date: row.get(5)?,
                image_url: row.get(6)?,
                total_time: row.get(7)?,
                local_url: row.get(8)?,
                local_image_url: row.get(9)?,
                description: row.get(10)?
            })
        }).unwrap();
        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast.unwrap());
        }
        return podcasts;
    }

    fn extract_watchtime_log(mut stmt: Statement, podcast_episode_id: &str) -> Result<Option<PodcastWatchedModel>> {
        let mut podcast_iter = stmt.query_map([podcast_episode_id], |row| {
            Ok(PodcastWatchedModel {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                watched_time: row.get(3)?,
                date: row.get(4)?,

            })
        }).unwrap();
        let iter = podcast_iter.next().map(|podcast| podcast.unwrap());
        Ok(iter)
    }


    fn extract_statement_with_episode(mut stmt: Statement, podcast_id: i64,podcast_episode: String )
        ->
                                                                               Vec<PodcastEpisode> {
        let podcast_iter = stmt.query_map(params![podcast_id, podcast_episode], |row| {
            Ok(PodcastEpisode {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                name: row.get(3)?,
                url: row.get(4)?,
                date: row.get(5)?,
                image_url: row.get(6)?,
                total_time: row.get(7)?,
                local_url: row.get(8)?,
                local_image_url: row.get(9)?,
                description: row.get(10)?
            })
        }).unwrap();
        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast.unwrap());
        }
        return podcasts;
    }

    pub fn log_watchtime(&self, watch_model: PodcastWatchedPostModel)->Result<()> {
        let result = self.get_podcast_episode_by_id(&watch_model.podcast_episode_id).unwrap();

        match result {
            Some(podcast_episode) => {
                let now = SystemTime::now();
                let now: DateTime<Utc> = now.into();
                let now: &str = &now.to_rfc3339();
                self.conn.execute("INSERT INTO Podcast_History (podcast_id, episode_id, \
                watched_time, date) VALUES (?1, \
        ?2, ?3, ?4)",
                                  (&podcast_episode.podcast_id, &podcast_episode.episode_id,
                                   &watch_model.time, &now))
                    .expect("TODO: panic message");
                Ok(())
            }
            None => {
                panic!("Podcast not found")
            }
        }
    }

    pub fn get_watchtime(&self, podcast_id: &str) ->Result<PodcastWatchedModel>{
        let result = self.get_podcast_episode_by_id(podcast_id).unwrap();

        match result {
            Some(podcast) => {
                let stmt = self.conn.prepare("SELECT * FROM PODCAST_HISTORY WHERE episode_id \
                = ?1 ORDER BY datetime(date) DESC LIMIT 1")?;
                match Self::extract_watchtime_log(stmt, podcast_id ){
                    Ok(Some(podcast)) => {
                        Ok(podcast)
                    }
                    Ok(None) => {
                        Ok(PodcastWatchedModel {
                            id: 0,
                            podcast_id: podcast.podcast_id,
                            episode_id: podcast.episode_id,
                            watched_time: 0,
                            date: "".to_string(),
                        })
                    }
                    Err(e) => {
                        panic!("Error: {}", e)
                    }
                }
            }
            None => {
                panic!("Podcast not found")
            }
        }
    }


    pub fn get_last_watched_podcasts(&self) -> Result<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>{
        let mut stmt = self.conn.prepare("SELECT * FROM (SELECT * FROM Podcast_History ORDER BY datetime(date) DESC) GROUP BY episode_id  LIMIT 10;")?;
        let podcast_iter = stmt.query_map([], |row| {
            Ok(PodcastWatchedModel {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                watched_time: row.get(3)?,
                date: row.get(4)?,
            })
        })?;

        let podcast_watch_episode = podcast_iter.map(|podcast_watched_model| {
            let podcast_watched_model = podcast_watched_model.unwrap();
            let optional_podcast = self.get_podcast_episode_by_id(&podcast_watched_model.episode_id)
                .unwrap();

            match optional_podcast {
                Some(podcast_episode) => {
                    let podcast_dto = self.mapping_service.map_podcastepisode_to_dto(&podcast_episode);
                    let podcast = self.get_podcast(podcast_episode.podcast_id).unwrap();
                    PodcastWatchedEpisodeModelWithPodcastEpisode{
                        id: podcast_watched_model.id,
                        watched_time: podcast_watched_model.watched_time,
                        podcast_id: podcast_watched_model.podcast_id,
                        episode_id: podcast_watched_model.episode_id,
                        date: podcast_watched_model.date,
                        url: podcast_episode.clone().url,
                        name: podcast_episode.clone().name,
                        image_url: podcast_episode.clone().image_url,
                        total_time: podcast_episode.clone().total_time,
                        podcast_episode: podcast_dto,
                        podcast
                    }
                }
                None => {
                    panic!("Podcast not found");
                }
            }
        }).collect::<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>();
        Ok(podcast_watch_episode)
    }

    pub fn update_total_podcast_time_and_image(&self, episode_id: &str, time: u64, image_url:
        &str, url: &str ) -> Result<()> {
        let result = self.get_podcast_episode_by_id(episode_id).unwrap();

        match result {
            Some(podcast) => {
                let mut stmt = self.conn.prepare("UPDATE podcast_episodes SET total_time = ?1, \
                local_image_url = ?3, local_url = ?4 \
                WHERE episode_id = ?2")?;
                stmt.execute(params![time, podcast.episode_id, &image_url, &url])?;
                Ok(())
            }
            None => {
                panic!("Podcast not found")
            }
        }
    }

    pub fn update_podcast_image(self,id: &str, image_url: &str) -> Result<()> {
        let mut stmt = self.conn.prepare("UPDATE Podcast SET image_url = ?1 \
        WHERE directory = ?2")?;
        stmt.execute(params![&image_url, id])?;
        Ok(())
    }

    pub fn get_podcast_by_directory(self, podcast_id: &str)->Result<Option<Podcast>>{
        let mut stmt = self.conn.prepare("SELECT * FROM Podcast WHERE directory = ?1")?;
        let mut podcast_iter = stmt.query_map([], |row| {
            Ok(Podcast {
                id: row.get(0)?,
                name: row.get(1)?,
                directory: row.get(2)?,
                rssfeed: row.get(3)?,
                image_url: row.get(4)?
            })
        })?;
        let iter = podcast_iter.next().map(|podcast| podcast.unwrap());
        Ok(iter)
    }
}