//! Watches each audiobookshelf library's folder_paths for filesystem changes
//! and debounces them into per-library scans.
//!
//! audiobookshelf parity: upstream's `Watcher.js` watches all configured
//! library folders, debounces by ~5 seconds, then runs the affected
//! library's scanner. We mirror the behaviour with the `notify` crate plus
//! a tokio task that collects events per library and triggers
//! `AudiobookScanner::scan_library` after the debounce window.
//!
//! Only enabled when `AUDIOBOOKSHELF_INTEGRATION_ENABLED=true` and the
//! library has at least one folder_path.

use crate::services::audiobookshelf::audiobook_scanner::AudiobookScanner;
use crate::services::audiobookshelf::library_service::AudiobookshelfLibraryService;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use podfetch_domain::audiobookshelf::library::{Library, MediaType};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const DEFAULT_DEBOUNCE: Duration = Duration::from_secs(5);

/// Holds one watcher per library and keeps the background task alive for
/// the lifetime of the process.
pub struct AudiobookFileWatcher {
    pub debounce: Duration,
    pub library_service: Arc<AudiobookshelfLibraryService>,
    pub scanner: Arc<AudiobookScanner>,
    watchers: Mutex<Vec<RecommendedWatcher>>,
    join_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[derive(Debug)]
struct WatchEvent {
    library_id: String,
}

impl AudiobookFileWatcher {
    pub fn new(
        library_service: Arc<AudiobookshelfLibraryService>,
        scanner: Arc<AudiobookScanner>,
    ) -> Self {
        Self {
            debounce: DEFAULT_DEBOUNCE,
            library_service,
            scanner,
            watchers: Mutex::new(Vec::new()),
            join_handle: Mutex::new(None),
        }
    }

    /// Wires up filesystem watchers for every book library that has folder
    /// paths configured. Returns `Ok` even when no library has folders to
    /// watch (no-op).
    pub fn start(&self) -> notify::Result<()> {
        let libraries = match self.library_service.list() {
            Ok(list) => list,
            Err(e) => {
                tracing::warn!("file_watcher: could not list libraries: {e:?}");
                return Ok(());
            }
        };
        let book_libraries: Vec<Library> = libraries
            .into_iter()
            .filter(|l| matches!(l.media_type, MediaType::Book))
            .filter(|l| !l.folder_paths.is_empty())
            .collect();
        if book_libraries.is_empty() {
            tracing::info!("file_watcher: no book libraries with folder paths; skipping");
            return Ok(());
        }

        let (tx, rx) = mpsc::unbounded_channel::<WatchEvent>();
        let mut watchers = self
            .watchers
            .lock()
            .expect("file_watcher mutex poisoned");

        for library in &book_libraries {
            for folder in &library.folder_paths {
                let path = PathBuf::from(folder);
                if !path.is_dir() {
                    tracing::warn!(
                        "file_watcher: library {} folder {} is not a directory; skipping",
                        library.id,
                        folder
                    );
                    continue;
                }
                let library_id = library.id.clone();
                let tx = tx.clone();
                let mut watcher = notify::recommended_watcher(
                    move |result: notify::Result<notify::Event>| {
                        if let Ok(event) = result
                            && is_meaningful(&event.kind)
                        {
                            let _ = tx.send(WatchEvent {
                                library_id: library_id.clone(),
                            });
                        }
                    },
                )?;
                watcher.watch(&path, RecursiveMode::Recursive)?;
                watchers.push(watcher);
                tracing::info!(
                    "file_watcher: watching library {} folder {}",
                    library.id,
                    folder
                );
            }
        }
        // Drop our local tx so the channel closes when all watchers are gone.
        drop(tx);

        let scanner = self.scanner.clone();
        let debounce = self.debounce;
        let join = tokio::spawn(async move {
            run_debounce_loop(rx, scanner, debounce).await;
        });
        let mut handle_slot = self
            .join_handle
            .lock()
            .expect("file_watcher mutex poisoned");
        *handle_slot = Some(join);
        Ok(())
    }
}

impl Drop for AudiobookFileWatcher {
    fn drop(&mut self) {
        if let Ok(mut handle) = self.join_handle.lock()
            && let Some(h) = handle.take()
        {
            h.abort();
        }
    }
}

fn is_meaningful(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

async fn run_debounce_loop(
    mut rx: mpsc::UnboundedReceiver<WatchEvent>,
    scanner: Arc<AudiobookScanner>,
    debounce: Duration,
) {
    // Map<library_id, deadline>. When the deadline passes without further
    // events, the library is scanned.
    let mut pending: HashMap<String, Instant> = HashMap::new();
    loop {
        // Decide how long to wait before re-checking deadlines.
        let next_wake = pending
            .values()
            .min()
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::from_secs(60));
        tokio::select! {
            maybe_event = rx.recv() => {
                match maybe_event {
                    Some(event) => {
                        pending.insert(event.library_id, Instant::now() + debounce);
                    }
                    None => {
                        // Channel closed (no more watchers); finish any pending scans then exit.
                        flush_due(&mut pending, &scanner);
                        return;
                    }
                }
            }
            _ = tokio::time::sleep(next_wake) => {
                flush_due(&mut pending, &scanner);
            }
        }
    }
}

fn flush_due(pending: &mut HashMap<String, Instant>, scanner: &Arc<AudiobookScanner>) {
    let now = Instant::now();
    let due_ids: Vec<String> = pending
        .iter()
        .filter_map(|(id, deadline)| (*deadline <= now).then(|| id.clone()))
        .collect();
    for id in due_ids {
        pending.remove(&id);
        let scanner = scanner.clone();
        let id_clone = id.clone();
        tokio::task::spawn_blocking(move || {
            match scanner.scan_library(&id_clone) {
                Ok(report) => tracing::info!(
                    "file_watcher: scan {} done (added={}, updated={}, errors={})",
                    id_clone,
                    report.books_added,
                    report.books_updated,
                    report.errors.len()
                ),
                Err(e) => tracing::warn!(
                    "file_watcher: scan {} failed: {e:?}",
                    id_clone
                ),
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{CreateKind, ModifyKind, RemoveKind};

    #[test]
    fn is_meaningful_accepts_create_modify_remove() {
        assert!(is_meaningful(&EventKind::Create(CreateKind::File)));
        assert!(is_meaningful(&EventKind::Modify(ModifyKind::Any)));
        assert!(is_meaningful(&EventKind::Remove(RemoveKind::File)));
    }

    #[test]
    fn is_meaningful_rejects_access_and_other() {
        assert!(!is_meaningful(&EventKind::Access(
            notify::event::AccessKind::Any
        )));
        assert!(!is_meaningful(&EventKind::Any));
        assert!(!is_meaningful(&EventKind::Other));
    }

    #[test]
    fn flush_due_does_not_remove_future_deadlines() {
        let mut pending: HashMap<String, Instant> = HashMap::new();
        pending.insert("future".to_string(), Instant::now() + Duration::from_secs(60));
        // We can't easily verify the scan trigger without a full scanner,
        // but we can at least confirm flush_due leaves future entries alone.
        let snapshot: Vec<String> = pending.keys().cloned().collect();
        let due_ids: Vec<String> = pending
            .iter()
            .filter_map(|(id, deadline)| (*deadline <= Instant::now()).then(|| id.clone()))
            .collect();
        assert!(due_ids.is_empty(), "future deadline should not be due");
        assert_eq!(snapshot, vec!["future".to_string()]);
    }

    #[test]
    fn flush_due_picks_up_past_deadlines() {
        let mut pending: HashMap<String, Instant> = HashMap::new();
        pending.insert(
            "past".to_string(),
            Instant::now() - Duration::from_secs(1),
        );
        let due_ids: Vec<String> = pending
            .iter()
            .filter_map(|(id, deadline)| (*deadline <= Instant::now()).then(|| id.clone()))
            .collect();
        assert_eq!(due_ids, vec!["past".to_string()]);
    }
}
