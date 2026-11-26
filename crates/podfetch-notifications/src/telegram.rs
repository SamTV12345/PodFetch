use common_infrastructure::config::TelegramConfig;
use frankenstein::ParseMode;
use frankenstein::TelegramApi;
use frankenstein::client_ureq::Bot;
use frankenstein::methods::SendMessageParams;

/// Sends messages to the configured Telegram chat.
#[derive(Clone)]
pub struct TelegramNotifier {
    config: TelegramConfig,
}

impl TelegramNotifier {
    pub fn new(config: TelegramConfig) -> Self {
        Self { config }
    }

    /// Sends an HTML message. Blocking: call it off the async runtime.
    pub fn send_html(&self, text: String) -> Result<(), String> {
        let message = SendMessageParams::builder()
            .chat_id(self.config.telegram_chat_id.clone())
            .text(text)
            .parse_mode(ParseMode::Html)
            .build();
        Bot::new(&self.config.telegram_bot_token)
            .send_message(&message)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub(crate) fn new_episode_message(episode_name: &str, podcast_name: &str) -> String {
    let episode_name = escape_html(episode_name);
    let podcast_name = escape_html(podcast_name);
    format!(
        "<strong>New episode available</strong>: Episode {episode_name} of podcast \
         {podcast_name} was downloaded successfully and is ready to be listened to."
    )
}

/// Telegram rejects HTML messages with unescaped `<`, `>` or `&`, which
/// episode titles regularly contain.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_escaped_for_telegram_html() {
        let message = new_episode_message("Q&A <live>", "Tech & Talk");
        assert!(message.contains("Q&amp;A &lt;live&gt;"));
        assert!(message.contains("Tech &amp; Talk"));
        assert!(message.starts_with("<strong>New episode available</strong>"));
    }
}
