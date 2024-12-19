mod podindex_api;

use std::cell::LazyCell;
use std::time::SystemTime;
use actix_web::web::Data;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use sha1::{Digest, Sha1};
use crate::adapters::podcast_apis::podindex_api::ByFeedPodindex;
use crate::constants::inner_constants::{COMMON_USER_AGENT, ENVIRONMENT_SERVICE, ITUNES_URL};
use crate::domain::models::podcast::podcast::Podcast;
use crate::utils::error::{map_reqwest_error, CustomError};


static HTTP_CLIENT: LazyCell<reqwest::Client> = LazyCell::new(|| reqwest::Client::new());

pub async fn find_podcast(podcast: &str) -> Value {
    let query = vec![("term", podcast), ("entity", "podcast")];
    let result = HTTP_CLIENT
        .get(ITUNES_URL)
        .query(&query)
        .send()
        .await
        .unwrap();
    log::info!("Found podcast: {}", result.url());
    let res_of_search = result.json().await;

    if let Ok(res) = res_of_search {
        res
    } else {
        log::error!(
                "Error searching for podcast: {}",
                res_of_search.err().unwrap()
            );
        serde_json::from_str("{}").unwrap()
    }
}

pub async fn find_podcast_on_podindex(podcast: &str) -> Result<Value, CustomError> {

    let headers = compute_podindex_header();

    let query = vec![("q", podcast)];

    let result = HTTP_CLIENT
        .get("https://api.podcastindex.org/api/1.0/search/byterm")
        .query(&query)
        .headers(headers)
        .send()
        .await
        .map_err(map_reqwest_error)?;

    log::info!("Found podcast: {}", result.url());

    let status = result.status();
    let possible_json = result.text().await.map_err(map_reqwest_error)?;

    if status.is_client_error() || status.is_server_error() {
        log::error!("Error searching for podcast: {}", possible_json);
        Err(CustomError::BadRequest(possible_json))
    } else {
        let res_of_search = serde_json::from_str(&possible_json);

        if let Ok(res) = res_of_search {
            Ok(res)
        } else {
            log::error!(
                    "Error searching for podcast: {}",
                    res_of_search.err().unwrap()
                );
            Ok(serde_json::from_str("{}").unwrap())
        }
    }
}


pub async fn insert_podcast_from_podindex(
    id: i32,
    lobby: Data<ChatServerHandle>,
) -> Result<Podcast, CustomError> {
    let resp = HTTP_CLIENT
        .get(
            "https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id=".to_owned()
                + &id.to_string(),
        )
        .headers(compute_podindex_header())
        .send()
        .await
        .unwrap();

    println!("Result: {:?}", resp);

    let podcast = resp.json::<ByFeedPodindex>().await.unwrap();

    self.handle_insert_of_podcast(
        PodcastInsertModel {
            title: unwrap_string(&podcast["feed"]["title"]),
            id,
            feed_url: unwrap_string(&podcast["feed"]["url"]),
            image_url: unwrap_string(&podcast["feed"]["image"]),
        },
        lobby,
        None,
    )
        .await
}


fn compute_podindex_header() -> HeaderMap {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut headers = HeaderMap::new();
    let non_hashed_string = ENVIRONMENT_SERVICE
        .get()
        .unwrap()
        .podindex_api_key
        .clone()
        .to_owned()
        + &*ENVIRONMENT_SERVICE
        .get()
        .unwrap()
        .podindex_api_secret
        .clone()
        + &seconds.to_string();
    let mut hasher = Sha1::new();

    hasher.update(non_hashed_string);

    let hashed_auth_key = format!("{:x}", hasher.finalize());

    headers.insert(
        "User-Agent",
        HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
    );
    headers.insert(
        "X-Auth-Key",
        HeaderValue::from_str(&ENVIRONMENT_SERVICE.get().unwrap().podindex_api_key).unwrap(),
    );
    headers.insert(
        "X-Auth-Date",
        HeaderValue::from_str(&seconds.to_string()).unwrap(),
    );
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&hashed_auth_key).unwrap(),
    );

    headers
}