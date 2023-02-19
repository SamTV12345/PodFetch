use feed_rs::parser;
use crate::constants::constants::{DB_NAME, ITUNES_URL};
use reqwest::{Request, Response};
use reqwest::blocking::ClientBuilder;
use rusqlite::{Connection, OpenFlags};
use crate::models::itunes_models::{Podcast, PodcastEpisode, ResponseModel};
use crate::service::file_service::check_if_podcast_episode_downloaded;
use regex::Regex;
use rusqlite::ffi::{sqlite3_unlock_notify, SQLITE_OPEN_NOMUTEX};

pub fn find_podcast(podcast: &str)-> ResponseModel {
    let client = ClientBuilder::new().build().unwrap();
    let result = client.get(ITUNES_URL.to_owned()+podcast).send().unwrap();
    return result.json::<ResponseModel>().unwrap();
}


// Used for creating/updating podcasts
pub fn insert_podcast_episodes(podcast: Podcast){
    let connection = Connection::open(DB_NAME);
    let connection_client = connection.unwrap();
    let client = ClientBuilder::new().build().unwrap();
    let result = client.get(podcast.rssfeed).send().unwrap();
    let bytes = result.bytes().unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let vec = get_media_urls(&text);

    let feed = parser::parse(&*bytes).unwrap();
    for (i,item) in feed.entries.iter().enumerate(){

        let mut result = connection_client.prepare("select * from podcast_episodes where episode_id = ?1")
            .expect("Error getting podcasts from database");

        let result = result.query_map([&item.id], |row| {
            Ok(Podcast {
                id: row.get(0)?,
                name: row.get(1)?,
                directory: row.get(2)?,
                rssfeed: row.get(3)?,
            })
        }).expect("Error getting podcasts from database");


        if result.count() == 0 {
            // Insert new podcast episode
            connection_client.execute("INSERT INTO podcast_episodes (podcast_id,\
                        episode_id, name, url, date) VALUES (?1,?2, ?3, ?4, ?5)",
                                      (&podcast.id, &item.id, &item.title.as_ref()
                                          .unwrap()
                                          .content,
                                       &vec[i], &item.published.unwrap()))
                .expect("Error inserting podcast episode");
        }
    }
}

pub fn schedule_episode_download(podcast: Podcast){
    let connection = Connection::open(DB_NAME);
    let mut connection_client = connection.unwrap();

    // Check if last 5 episodes are downloaded
    let mut result = connection_client.prepare("select * from podcast_episodes where podcast_id =\
     ?1 ORDER BY date DESC LIMIT 5")
        .expect("Error getting podcasts from database");

    let result = result.query_map([&podcast.id], |row| {
        Ok(PodcastEpisode {
            id: row.get(0)?,
            podcast_id: row.get(1)?,
            episode_id: row.get(2)?,
            name: row.get(3)?,
            url: row.get(4)?,
            date: row.get(5)?,
        })
    }).expect("Error getting podcasts from database");

    for res in result {
        let podcast = res.unwrap();
        if !check_if_podcast_episode_downloaded(podcast.podcast_id, podcast.episode_id) {

        }
    }
}



fn get_media_urls(text: &str)-> Vec<String> {
    let mut urls = Vec::new();
    let re = Regex::new(r#"enclosure\s+url="([^"]+)""#).unwrap();
    for capture in re.captures_iter(text){
        let url = capture.get(1).unwrap().as_str();
        urls.push(url.to_owned())
    }
    return urls;
}