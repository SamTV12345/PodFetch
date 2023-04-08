use std::env::var;
use frankenstein::{ParseMode, SendMessageParams, TelegramApi};
use crate::API;
use crate::constants::constants::TELEGRAM_BOT_CHAT_ID;
use crate::models::itunes_models::{Podcast, PodcastEpisode};

// Thanks to https://github.com/ayrat555/frankenstein/issues/76#issuecomment-1173534397
fn parse_telegram_message_string(bad: String) -> String {
    let mut parsed = String::new();
    for c in bad.chars() {
        match c {
            '[' => {
                parsed.push_str("\\[");
            }
            ']' => {
                parsed.push_str("\\]");
            }
            '.' => {
                parsed.push_str("\\.");
            }
            '!' => {
                parsed.push_str("\\!");
            }
            '-' => {
                parsed.push_str("\\-");
            }
            '=' => {
                parsed.push_str("\\=");
            }
            '#' => {
                parsed.push_str(r"\#");
            }
            '*' => {
                parsed.push_str("\\*");
            }
            '(' => {
                parsed.push_str("\\(");
            }
            ')' => {
                parsed.push_str("\\)");
            }
            '+' => {
                parsed.push_str("\\+");
            }
            '}' => {
                parsed.push_str("\\}");
            }
            '{' => {
                parsed.push_str("\\{");
            }
            '_' => {
                parsed.push_str("\\_");
            }
            _ => {
                let mut b = [0; 4];
                parsed.push_str(c.encode_utf8(&mut b))
            }
        }
    }
    parsed
}


pub fn send_new_episode_notification(podcast_episode: PodcastEpisode, podcast: Podcast){
    let episode_text = format!("Episode {} of podcast {} \
    was downloaded successfully and is ready to be listened to.",podcast_episode.name, podcast
        .name);
    let message_to_send = format!(r"<strong>New episode available</strong>: {}",
                                  episode_text);

    let message = SendMessageParams::builder()
        .chat_id(var(TELEGRAM_BOT_CHAT_ID).unwrap())
        .text(message_to_send)
        .disable_web_page_preview(true)
        .parse_mode(ParseMode::Html)
        .build();
    let telegram_res = API.get().unwrap().send_message(&message);
    match telegram_res {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error sending telegram message: {}", e);
        }
    }
}