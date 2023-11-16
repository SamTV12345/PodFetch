use crate::constants::inner_constants::{TELEGRAM_BOT_CHAT_ID, TELEGRAM_BOT_TOKEN};
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use frankenstein::{Api, ParseMode, SendMessageParams, TelegramApi};
use std::env::var;

pub fn send_new_episode_notification(podcast_episode: PodcastEpisode, podcast: Podcast) {
    let api = Api::new(&var(TELEGRAM_BOT_TOKEN).unwrap());

    let episode_text = format!(
        "Episode {} of podcast {} \
    was downloaded successfully and is ready to be listened to.",
        podcast_episode.name, podcast.name
    );
    let message_to_send = format!(r"<strong>New episode available</strong>: {}", episode_text);

    let message = SendMessageParams::builder()
        .chat_id(var(TELEGRAM_BOT_CHAT_ID).unwrap())
        .text(message_to_send)
        .disable_web_page_preview(true)
        .parse_mode(ParseMode::Html)
        .build();
    let telegram_res = api.send_message(&message);
    match telegram_res {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error sending telegram message: {}", e);
        }
    }
}
