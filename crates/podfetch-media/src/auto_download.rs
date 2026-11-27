//! Auto-download: when the catalog refreshed a podcast, download its latest
//! episodes, in the background and at most `max_parallel_downloads` at a
//! time.
//!
//! The event handler only fills the queue and returns: downloads can take
//! longer than an event handler may run, and a retried handler must not
//! start a second download of the same file.

use crate::catalog::Catalog;
use crate::settings::MediaSettingsSource;
use common_infrastructure::error::CustomError;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// Port to the code that downloads one episode and announces it.
pub trait DownloadRunner: Send + Sync {
    /// Downloads the episode (by catalog id) unless its file is there
    /// already.
    fn download(&self, episode_id: &str) -> Result<(), CustomError>;
}

pub struct AutoDownload {
    catalog: Arc<dyn Catalog>,
    settings: Arc<dyn MediaSettingsSource>,
    queue: DownloadQueue,
}

impl AutoDownload {
    pub fn new(
        catalog: Arc<dyn Catalog>,
        settings: Arc<dyn MediaSettingsSource>,
        runner: Arc<dyn DownloadRunner>,
    ) -> Self {
        Self {
            queue: DownloadQueue::new(runner, settings.clone()),
            catalog,
            settings,
        }
    }

    /// Queues the podcast's latest episodes that aren't downloaded yet, if
    /// auto-download is on for it. Returns how many were queued.
    pub fn podcast_refreshed(&self, podcast_id: &str) -> Result<usize, CustomError> {
        let settings = self.settings.for_podcast(podcast_id)?;
        if !settings.auto_download {
            return Ok(0);
        }
        let queued = self
            .catalog
            .latest_episodes(podcast_id, settings.prefill)?
            .into_iter()
            // The user deleted the file on purpose; a downloaded one is there.
            .filter(|e| !e.file.deleted && e.file.download_location.is_none())
            .filter(|e| self.queue.enqueue(&e.id))
            .count();
        Ok(queued)
    }

    pub fn queue(&self) -> &DownloadQueue {
        &self.queue
    }
}

/// Runs downloads in background threads, a bounded number at a time, and
/// never the same episode twice at once.
#[derive(Clone)]
pub struct DownloadQueue {
    inner: Arc<Inner>,
}

struct Inner {
    runner: Arc<dyn DownloadRunner>,
    settings: Arc<dyn MediaSettingsSource>,
    state: Mutex<State>,
    idle: Condvar,
}

#[derive(Default)]
struct State {
    waiting: VecDeque<String>,
    /// Waiting or running.
    known: HashSet<String>,
    running: usize,
}

impl DownloadQueue {
    pub fn new(runner: Arc<dyn DownloadRunner>, settings: Arc<dyn MediaSettingsSource>) -> Self {
        Self {
            inner: Arc::new(Inner {
                runner,
                settings,
                state: Mutex::new(State::default()),
                idle: Condvar::new(),
            }),
        }
    }

    /// Queues the episode; `false` if it is waiting or running already.
    pub fn enqueue(&self, episode_id: &str) -> bool {
        {
            let mut state = self.inner.lock();
            if !state.known.insert(episode_id.to_string()) {
                return false;
            }
            state.waiting.push_back(episode_id.to_string());
        }
        Inner::pump(&self.inner);
        true
    }

    /// Waits until nothing is waiting or running; `false` on timeout.
    pub fn wait_idle(&self, timeout: Duration) -> bool {
        let state = self.inner.lock();
        let (state, _) = self
            .inner
            .idle
            .wait_timeout_while(state, timeout, |s| !s.known.is_empty())
            .unwrap_or_else(PoisonError::into_inner);
        state.known.is_empty()
    }
}

impl Inner {
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Starts waiting downloads while fewer than the limit run.
    fn pump(inner: &Arc<Self>) {
        let limit = inner
            .settings
            .max_parallel_downloads()
            .unwrap_or_else(|err| {
                tracing::warn!("Could not read the download limit, using 1: {err}");
                1
            });
        let mut state = inner.lock();
        while state.running < limit.max(1) {
            let Some(episode_id) = state.waiting.pop_front() else {
                break;
            };
            state.running += 1;
            let worker = inner.clone();
            std::thread::spawn(move || {
                if let Err(err) = worker.runner.download(&episode_id) {
                    tracing::error!("Error downloading podcast episode {episode_id}: {err}");
                }
                {
                    let mut state = worker.lock();
                    state.running -= 1;
                    state.known.remove(&episode_id);
                    if state.known.is_empty() {
                        worker.idle.notify_all();
                    }
                }
                Inner::pump(&worker);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::Chapter;
    use crate::catalog::{MediaEpisode, MediaPodcast};
    use crate::files::EpisodeFile;
    use crate::settings::MediaSettings;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Settings {
        auto_download: bool,
        limit: usize,
    }

    impl MediaSettingsSource for Settings {
        fn for_podcast(&self, _: &str) -> Result<MediaSettings, CustomError> {
            Ok(MediaSettings {
                auto_download: self.auto_download,
                prefill: 5,
                ..MediaSettings::default()
            })
        }
        fn max_parallel_downloads(&self) -> Result<usize, CustomError> {
            Ok(self.limit)
        }
    }

    /// Counts downloads and the most running at once; each takes `delay`.
    #[derive(Default)]
    struct Runner {
        done: Mutex<Vec<String>>,
        running: AtomicUsize,
        most_at_once: AtomicUsize,
        delay: Duration,
    }

    impl DownloadRunner for Runner {
        fn download(&self, episode_id: &str) -> Result<(), CustomError> {
            let now = self.running.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_at_once.fetch_max(now, Ordering::SeqCst);
            std::thread::sleep(self.delay);
            self.running.fetch_sub(1, Ordering::SeqCst);
            self.done.lock().unwrap().push(episode_id.to_string());
            Ok(())
        }
    }

    struct Episodes(Vec<MediaEpisode>);

    impl Catalog for Episodes {
        fn podcast(&self, _: &str) -> Result<MediaPodcast, CustomError> {
            unreachable!()
        }
        fn episodes_of_podcast(&self, _: &str) -> Result<Vec<MediaEpisode>, CustomError> {
            unreachable!()
        }
        fn latest_episodes(&self, _: &str, n: i32) -> Result<Vec<MediaEpisode>, CustomError> {
            assert_eq!(n, 5, "the prefill setting decides how many");
            Ok(self.0.clone())
        }
        fn position_of_episode(&self, _: &str, _: &str) -> Result<usize, CustomError> {
            unreachable!()
        }
        fn episode_count(&self, _: &str) -> Result<usize, CustomError> {
            unreachable!()
        }
        fn track_number(&self, _: &str, _: &str) -> Result<i64, CustomError> {
            unreachable!()
        }
        fn podcast_cover_downloaded(&self, _: &str) -> bool {
            unreachable!()
        }
        fn set_numbered(&self, _: &MediaEpisode, _: bool) -> Result<(), CustomError> {
            unreachable!()
        }
        fn set_duration(&self, _: &MediaEpisode, _: i32) -> Result<(), CustomError> {
            unreachable!()
        }
        fn save_chapter(&self, _: &MediaEpisode, _: &Chapter) -> Result<(), CustomError> {
            unreachable!()
        }
        fn set_podcast_image(&self, _: &MediaPodcast, _: &str) -> Result<(), CustomError> {
            unreachable!()
        }
    }

    fn episode(id: &str, file: EpisodeFile) -> MediaEpisode {
        MediaEpisode {
            id: id.to_string(),
            file,
            ..MediaEpisode::default()
        }
    }

    fn auto_download(
        auto_download: bool,
        limit: usize,
        episodes: Vec<MediaEpisode>,
        delay: Duration,
    ) -> (AutoDownload, Arc<Runner>) {
        let runner = Arc::new(Runner {
            delay,
            ..Runner::default()
        });
        let auto = AutoDownload::new(
            Arc::new(Episodes(episodes)),
            Arc::new(Settings {
                auto_download,
                limit,
            }),
            runner.clone(),
        );
        (auto, runner)
    }

    fn done(runner: &Runner) -> Vec<String> {
        let mut done = runner.done.lock().unwrap().clone();
        done.sort();
        done
    }

    #[test]
    fn downloads_the_latest_episodes_not_on_disk() {
        let downloaded = EpisodeFile {
            download_location: Some("Local".into()),
            ..EpisodeFile::default()
        };
        let deleted = EpisodeFile {
            deleted: true,
            ..EpisodeFile::default()
        };
        let (auto, runner) = auto_download(
            true,
            2,
            vec![
                episode("new-1", EpisodeFile::default()),
                episode("on-disk", downloaded),
                episode("deleted-by-user", deleted),
                episode("new-2", EpisodeFile::default()),
            ],
            Duration::ZERO,
        );

        assert_eq!(auto.podcast_refreshed("p1").unwrap(), 2);
        assert!(auto.queue().wait_idle(Duration::from_secs(5)));
        assert_eq!(done(&runner), ["new-1", "new-2"]);
    }

    #[test]
    fn nothing_happens_without_auto_download() {
        let (auto, runner) = auto_download(
            false,
            2,
            vec![episode("new-1", EpisodeFile::default())],
            Duration::ZERO,
        );

        assert_eq!(auto.podcast_refreshed("p1").unwrap(), 0);
        assert!(auto.queue().wait_idle(Duration::from_secs(1)));
        assert!(done(&runner).is_empty());
    }

    #[test]
    fn runs_at_most_the_limit_at_once() {
        let episodes = (0..6)
            .map(|i| episode(&format!("e{i}"), EpisodeFile::default()))
            .collect();
        let (auto, runner) = auto_download(true, 2, episodes, Duration::from_millis(50));

        auto.podcast_refreshed("p1").unwrap();
        assert!(auto.queue().wait_idle(Duration::from_secs(10)));

        assert_eq!(done(&runner).len(), 6);
        assert_eq!(runner.most_at_once.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_queued_episode_is_not_queued_again() {
        let (auto, runner) = auto_download(
            true,
            1,
            vec![episode("e1", EpisodeFile::default())],
            Duration::from_millis(100),
        );

        assert_eq!(auto.podcast_refreshed("p1").unwrap(), 1);
        // A second refresh (or a redelivered event) while it downloads.
        assert_eq!(auto.podcast_refreshed("p1").unwrap(), 0);
        assert!(auto.queue().wait_idle(Duration::from_secs(5)));
        assert_eq!(done(&runner), ["e1"]);

        // Once done it can be queued again (the runner skips files on disk).
        assert_eq!(auto.podcast_refreshed("p1").unwrap(), 1);
        assert!(auto.queue().wait_idle(Duration::from_secs(5)));
    }
}
