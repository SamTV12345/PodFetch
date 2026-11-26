//! Event handlers of the notifications context.
//!
//! Each reaction is its own handler, so the event bus retries them
//! independently: a Telegram outage is retried without creating the in-app
//! notification a second time.

use crate::inbound::{EpisodeDownloadFailed, EpisodeDownloaded};
use crate::service::{NotificationKind, NotificationService};
use crate::telegram::{TelegramNotifier, new_episode_message};
use podfetch_events::{EventBus, HandlerError};
use std::sync::Arc;

pub const INBOX_ON_DOWNLOAD: &str = "notifications.inbox_on_download";
pub const INBOX_ON_DOWNLOAD_FAILED: &str = "notifications.inbox_on_download_failed";
pub const TELEGRAM_ON_DOWNLOAD: &str = "notifications.telegram_on_download";

/// Registers the handlers of this context. Telegram is only subscribed when
/// it is configured.
pub fn subscribe(
    bus: &EventBus,
    service: Arc<NotificationService>,
    telegram: Option<TelegramNotifier>,
) {
    let inbox = service.clone();
    bus.subscribe::<EpisodeDownloaded, _, _>(INBOX_ON_DOWNLOAD, move |event, _| {
        let inbox = inbox.clone();
        async move {
            inbox
                .notify(NotificationKind::Download, event.episode_name)
                .map(|_| ())
                .map_err(|e| HandlerError::retry(e.to_string()))
        }
    });

    let inbox = service;
    bus.subscribe::<EpisodeDownloadFailed, _, _>(INBOX_ON_DOWNLOAD_FAILED, move |event, _| {
        let inbox = inbox.clone();
        async move {
            inbox
                .notify(
                    NotificationKind::DownloadFailed,
                    format!("{} ({})", event.episode_name, event.reason),
                )
                .map(|_| ())
                .map_err(|e| HandlerError::retry(e.to_string()))
        }
    });

    if let Some(telegram) = telegram {
        bus.subscribe::<EpisodeDownloaded, _, _>(TELEGRAM_ON_DOWNLOAD, move |event, _| {
            let telegram = telegram.clone();
            async move {
                let text = new_episode_message(&event.episode_name, &event.podcast_name);
                tokio::task::spawn_blocking(move || telegram.send_html(text))
                    .await
                    .map_err(|e| HandlerError::retry(e.to_string()))?
                    .map_err(HandlerError::retry)
            }
        });
    }
}
