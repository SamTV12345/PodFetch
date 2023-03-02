use std::fs::create_dir;
use std::io;
use feed_rs::parser;
use crate::constants::constants::{ITUNES_URL};
use reqwest::blocking::ClientBuilder;
use reqwest::ClientBuilder as AsyncClientBuilder;
use crate::service::file_service::{check_if_podcast_episode_downloaded, check_if_podcast_main_image_downloaded};
use regex::Regex;
use serde_json::Value;
use crate::db::{DB};
use crate::models::itunes_models::Podcast;
use crate::service::path_service::PathService;

pub async fn find_podcast(podcast: &str)-> Value {
    let client = AsyncClientBuilder::new().build().unwrap();
    let result = client.get(ITUNES_URL.to_owned()+podcast).send().await.unwrap();
    log::info!("Found podcast: {}", result.url());
    return result.json().await.unwrap();
}


// Used for creating/updating podcasts
pub fn insert_podcast_episodes(podcast: Podcast){
    let client = ClientBuilder::new().build().unwrap();
    let result = client.get(podcast.clone().rssfeed).send().unwrap();
    let bytes = result.bytes().unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    let vec = get_media_urls(&text);

    let feed = parser::parse(&*bytes).unwrap();
    for (i,item) in feed.entries.iter().enumerate(){
        let mut db = DB::new().unwrap();
        let result = db.get_podcast_episode_by_id(&item.id);

        if result.unwrap().is_none() {
            // Insert new podcast episode
            db.insert_podcast_episodes(podcast.clone(), &vec[i].to_owned(),
                                       item, &feed.logo
                .clone().unwrap().uri,&item.summary.clone().unwrap().content);
        }
    }
}

pub fn schedule_episode_download(podcast: Podcast){
    let mut db = DB::new().unwrap();
    let result = db.get_last_5_podcast_episodes(podcast.id).unwrap();
    for podcast_episode in result {

        let podcast_episode_cloned = podcast_episode.clone();
        let podcast_cloned = podcast.clone();
        let suffix = get_url_file_suffix(&podcast_episode_cloned.url);
        let image_suffix = get_url_file_suffix(&podcast_episode_cloned.image_url);

        let image_save_path = format!("podcasts/{}/{}/image.{}",
                                      podcast.directory,
                                      podcast_episode_cloned.episode_id,
                                      image_suffix);
        let image_podcast_path = format!("podcasts/{}/image.{}",
                                         podcast.directory,
                                         image_suffix);

        let podcast_save_path = PathService::get_podcast_episode_path(&podcast.directory.clone(),
                                                                      &podcast_episode_cloned.episode_id,
                                                                      &suffix);
        let duration = mp3_duration::from_path(podcast_save_path.clone()).unwrap();


        if !check_if_podcast_episode_downloaded(&podcast_cloned.directory, podcast_episode
            .episode_id) {
            let client = ClientBuilder::new().build().unwrap();
            let mut resp = client.get(podcast_episode.url).send().unwrap();
            let mut image_response = client.get(podcast_episode.image_url).send().unwrap();

            create_dir(format!("podcasts/{}/{}", podcast.directory,
                               podcast_episode_cloned.episode_id)).expect("Error creating directory");

            let mut podcast_out = std::fs::File::create(podcast_save_path.clone()).unwrap();
            let mut image_out = std::fs::File::create(image_save_path.clone())
                .unwrap();

            if !check_if_podcast_main_image_downloaded(&podcast_cloned.directory) {
                let mut image_podcast = std::fs::File::create(image_podcast_path)
                    .unwrap();
                io::copy(&mut image_response, &mut image_podcast).expect("failed to copy content");
            }

            io::copy(&mut resp, &mut podcast_out).expect("failed to copy content");
            db.update_total_podcast_time_and_image(&podcast_episode_cloned.episode_id, duration
                .as_secs() as i32, &image_save_path,
                                                   &podcast_save_path.clone())
                .expect("TODO: panic message");
            io::copy(&mut image_response, &mut image_out).expect("failed to copy content");
        }
        else{
            db.update_total_podcast_time_and_image(&podcast_episode_cloned.episode_id, duration
                .as_secs() as i32, &image_save_path,
                                                   &podcast_save_path.clone())
                .expect("Error saving total time of podcast episode.");
        }
    }
}

fn get_media_urls(text: &str)-> Vec<String> {
    let mut urls = Vec::new();
    let re = Regex::new(r#"<enclosure.*?url="(.*?)".*?/>"#).unwrap();
    for capture in re.captures_iter(text){
        let url = capture.get(1).unwrap().as_str();
        urls.push(url.to_owned())
    }
    return urls;
}

pub fn get_url_file_suffix(url: &str) -> String {
    let re = Regex::new(r#"\.(\w+)(?:\?.*)?$"#).unwrap();
    let capture = re.captures(&url).unwrap();
    return capture.get(1).unwrap().as_str().to_owned();
}