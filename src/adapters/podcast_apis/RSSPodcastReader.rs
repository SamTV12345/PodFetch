use std::sync::{Arc, Mutex};
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::redirect::Policy;
use rss::{Channel, Item};
use crate::constants::inner_constants::COMMON_USER_AGENT;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;
use crate::utils::error::{map_channel_error, CustomError};
use crate::utils::reqwest_client::get_sync_client;

fn read_from_podcast_rss_feed(rss_feed: &str) -> Result<(Podcast, Vec<PodcastEpisode>), CustomError> {
    let returned_data_from_podcast_insert = do_request_to_podcast_server(rss_feed);
    parse_response(returned_data_from_podcast_insert)
}





impl Into<PodcastEpisode> for Item {
    fn into(self) -> PodcastEpisode {
        let mut episode = PodcastEpisode::default();
        if let Some(title) = self.title() {
            episode.name = title.to_string();
        }

        if let Some(link) = self.enclosure() {
            episode.url = link.url().to_string();
        }

        if let Some(pub_date) = self.pub_date() {
            episode.date_of_recording = pub_date.to_string();
        }

        if let Some(description) = self.description() {
            episode.description = description.to_string();
        }

        if let Some(guid) = self.guid() {
            episode.guid = guid.to_string();
        }

        if let Some(itunes_ext) = self.itunes_ext() {
            if let Some(image) = &itunes_ext.image {
                episode.image_url = image.to_string();
            }

            if let Some(duration) = &itunes_ext.duration {
                episode.total_time = duration.parse().unwrap();
            }
        }
        episode
    }
}

impl Into<Podcast> for Channel {
    fn into(self) -> Podcast {
        let mut episode = Podcast::default();

        if let Some(title) = self.title() {
            episode.name = title.to_string();
        }

        if let Some(link) = self.link() {
            episode.rssfeed = link.to_string();
        }

        if let Some(pub_date) = self.last_build_date() {
            episode.last_build_date = Some(pub_date.to_string());
        }

        if let Some(itunes_ext) = self.itunes_ext() {
            if let Some(image) = &itunes_ext.image {
                episode.original_image_url = image.to_string();
            }

            if let Some(language) = &self.language {
                episode.language = Some(language.to_string());
            }

            if let Some(keywords) = &itunes_ext.keywords {
                episode.keywords = Some(keywords.to_string());
            }

            if let Some(summary) = &itunes_ext.summary {
                episode.summary = Some(summary.to_string());
            }

            if let Some(explicit) = &itunes_ext.explicit {
                episode.explicit = Option::from(explicit.to_string());
            }

            if let Some(author) = &itunes_ext.author {
                episode.author = Some(author.to_string());
            }

            if let Some(new_feed) = &itunes_ext.new_feed_url {
                episode.rssfeed = new_feed.to_string();
            }

        }

        episode
    }
}



fn parse_response(rqtype: RequestReturnType) -> Result<(Podcast, Vec<PodcastEpisode>),
    CustomError>  {
    let channel = Channel::read_from(rqtype.content.as_bytes())
        .map_err(map_channel_error)?;

    let podcast_to_insert = channel.clone().into();


    if rqtype.url != podcast_to_insert.rssfeed {
        // The itunes extension new feed url is different from the original feed url
        // We need to update the podcast with the new feed url
        let returned_data_from_podcast_insert = do_request_to_podcast_server(podcast_to_insert.rssfeed);
        parse_response(returned_data_from_podcast_insert)?;
    }

    let podcast_episodes = channel.items.into_iter().map(|item| item.into()).collect();

    Ok((podcast_to_insert, podcast_episodes))
}


fn do_request_to_podcast_server(rss_feed: &str) -> RequestReturnType {
    let is_redirected = Arc::new(Mutex::new(false)); // Variable to store the redirection status
    let client = get_sync_client()
        .redirect(Policy::custom({
            let is_redirected = Arc::clone(&is_redirected);

            move |attempt| {
                if !attempt.previous().is_empty() {
                    *is_redirected.lock().unwrap() = true;
                }
                attempt.follow()
            }
        }))
        .build()
        .unwrap();
    let mut header_map = HeaderMap::new();
    header_map.append(
        ACCEPT,
        "application/rss+xml,application/xml".parse().unwrap(),
    );
    header_map.append("User-Agent", COMMON_USER_AGENT.parse().unwrap());
    let result = client
        .get(rss_feed)
        .headers(header_map)
        .send()
        .unwrap();
    let url = result.url().clone().to_string();
    let content = result.text().unwrap().clone();

    RequestReturnType { url, content }
}

struct RequestReturnType {
    pub url: String,
    pub content: String,
}