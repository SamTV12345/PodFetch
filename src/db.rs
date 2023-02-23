use feed_rs::model::Entry;
use rusqlite::{Connection, params, Result, Statement};
use rusqlite::types::Value;
use serde::de::Unexpected::Str;
use crate::constants::constants::DB_NAME;
use crate::models::itunes_models::{Podcast, PodcastEpisode};


pub struct DB{
    conn: Connection,
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
        Ok(DB{conn})
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
            })
        })?;

        let iter = podcast_iter.next().map(|podcast| podcast.unwrap());
        Ok((iter))
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
        Ok((iter))
    }

    pub fn insert_podcast_episodes(&self, podcast: Podcast, link: &str, item: &Entry, image_url:
    &str){
        self.conn.execute("INSERT INTO podcast_episodes (podcast_id,\
                        episode_id, name, url, date, image_url) VALUES (?1,?2, ?3, ?4, ?5, ?6)",
                                  (&podcast.id, &item.id, &item.title.as_ref()
                                      .unwrap()
                                      .content,
                                   link, &item.published.unwrap(), image_url))
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
        let mut stmt = self.conn.prepare("select * from podcast_episodes where podcast_id = ?1 \
        order by date(date) desc limit 5")?;
        Ok(Self::extract_statement(stmt, podcast_id, ))
    }


    pub fn get_podcast_episodes_of_podcast(&self, podcast_id: i64,  last_id: Option<String>) ->
                                                                      Result<Vec<PodcastEpisode>>{
        let mut stmt:Statement;
        match last_id {
            Some(last_id) => {
                println!("last id: {}", last_id);
                 stmt = self.conn.prepare("select * from podcast_episodes where podcast_id = ?1 \
        AND date(date) < ?2 \
        order by date(date) desc LIMIT 75")?;
                Ok(Self::extract_statement_with_episode(stmt, podcast_id, last_id))
            }
            None => {
                stmt = self.conn.prepare("select * from podcast_episodes where podcast_id\
                 = ?1 LIMIT 75")?;
                Ok(Self::extract_statement(stmt, podcast_id))
            }
        }


    }

    fn extract_statement(mut stmt: Statement, podcast_id: i64) -> Vec<PodcastEpisode> {
        let podcast_iter = stmt.query_map([&podcast_id], |row| {
            Ok(PodcastEpisode {
                id: row.get(0)?,
                podcast_id: row.get(1)?,
                episode_id: row.get(2)?,
                name: row.get(3)?,
                url: row.get(4)?,
                date: row.get(5)?,
                image_url: row.get(6)?,
            })
        }).unwrap();
        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast.unwrap());
        }
        return podcasts;
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
            })
        }).unwrap();
        let mut podcasts = Vec::new();
        for podcast in podcast_iter {
            podcasts.push(podcast.unwrap());
        }
        return podcasts;
    }
}