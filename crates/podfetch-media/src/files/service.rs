use super::domain::{EpisodeFile, EpisodeFileRepository};
use chrono::NaiveDateTime;
use common_infrastructure::error::CustomError;
use std::sync::Arc;

pub struct EpisodeFileService {
    repository: Arc<dyn EpisodeFileRepository>,
}

impl EpisodeFileService {
    pub fn new(repository: Arc<dyn EpisodeFileRepository>) -> Self {
        Self { repository }
    }

    /// The file state of the episode; empty if it was never downloaded.
    pub fn file(&self, episode_id: &str) -> Result<EpisodeFile, CustomError> {
        Ok(self
            .repository
            .find(episode_id)?
            .unwrap_or_else(|| EpisodeFile::none(episode_id)))
    }

    /// Records where the downloaded audio file and its image ended up. No
    /// image clears the stored one.
    pub fn set_paths(
        &self,
        episode_id: &str,
        image_path: Option<&str>,
        episode_path: &str,
    ) -> Result<(), CustomError> {
        self.repository.modify(episode_id, &mut |file| {
            file.file_episode_path = Some(episode_path.to_string());
            file.file_image_path = image_path.map(str::to_string);
        })
    }

    /// Records that the episode was downloaded to `location` at `time`.
    pub fn record_download(
        &self,
        episode_id: &str,
        location: Option<String>,
        time: NaiveDateTime,
    ) -> Result<(), CustomError> {
        self.repository.modify(episode_id, &mut |file| {
            file.download_location.clone_from(&location);
            file.download_time = Some(time);
        })
    }

    /// Forgets the download (the file is gone). Keeps whether the user
    /// deleted it.
    pub fn clear_download(&self, episode_id: &str) -> Result<(), CustomError> {
        self.repository.modify(episode_id, &mut |file| {
            *file = EpisodeFile {
                deleted: file.deleted,
                ..EpisodeFile::none(episode_id)
            };
        })
    }

    /// Marks the file as deleted by the user, or lifts that mark.
    pub fn set_deleted(&self, episode_id: &str, deleted: bool) -> Result<(), CustomError> {
        self.repository
            .modify(episode_id, &mut |file| file.deleted = deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryRepository(Mutex<HashMap<String, EpisodeFile>>);

    impl EpisodeFileRepository for MemoryRepository {
        fn find(&self, episode_id: &str) -> Result<Option<EpisodeFile>, CustomError> {
            Ok(self.0.lock().unwrap().get(episode_id).cloned())
        }

        fn modify(
            &self,
            episode_id: &str,
            change: &mut dyn FnMut(&mut EpisodeFile),
        ) -> Result<(), CustomError> {
            let mut files = self.0.lock().unwrap();
            let mut file = files
                .get(episode_id)
                .cloned()
                .unwrap_or_else(|| EpisodeFile::none(episode_id));
            change(&mut file);
            if file.is_empty() {
                files.remove(episode_id);
            } else {
                files.insert(episode_id.to_string(), file);
            }
            Ok(())
        }
    }

    fn service() -> (EpisodeFileService, Arc<MemoryRepository>) {
        let repository = Arc::new(MemoryRepository::default());
        (EpisodeFileService::new(repository.clone()), repository)
    }

    fn time() -> NaiveDateTime {
        chrono::DateTime::from_timestamp(1_780_000_000, 0)
            .unwrap()
            .naive_utc()
    }

    #[test]
    fn an_unknown_episode_has_no_file() {
        let (service, _) = service();
        assert_eq!(service.file("e1").unwrap(), EpisodeFile::none("e1"));
    }

    #[test]
    fn a_download_records_paths_location_and_time() {
        let (service, _) = service();
        service
            .set_paths("e1", Some("p/image.jpg"), "p/podcast.mp3")
            .unwrap();
        service
            .record_download("e1", Some("Local".into()), time())
            .unwrap();

        let file = service.file("e1").unwrap();
        assert_eq!(file.file_episode_path.as_deref(), Some("p/podcast.mp3"));
        assert_eq!(file.file_image_path.as_deref(), Some("p/image.jpg"));
        assert_eq!(file.download_location.as_deref(), Some("Local"));
        assert_eq!(file.download_time, Some(time()));
        assert!(!file.deleted);
    }

    #[test]
    fn setting_paths_without_image_clears_the_image() {
        let (service, _) = service();
        service.set_paths("e1", Some("image.jpg"), "a.mp3").unwrap();
        service.set_paths("e1", None, "a.opus").unwrap();

        let file = service.file("e1").unwrap();
        assert_eq!(file.file_episode_path.as_deref(), Some("a.opus"));
        assert_eq!(file.file_image_path, None);
    }

    #[test]
    fn clearing_a_download_keeps_the_deleted_mark() {
        let (service, repository) = service();
        service.set_paths("e1", Some("image.jpg"), "a.mp3").unwrap();
        service.record_download("e1", None, time()).unwrap();
        service.set_deleted("e1", true).unwrap();

        service.clear_download("e1").unwrap();

        assert_eq!(
            service.file("e1").unwrap(),
            EpisodeFile {
                deleted: true,
                ..EpisodeFile::none("e1")
            }
        );

        // Lifting the mark leaves nothing to store.
        service.set_deleted("e1", false).unwrap();
        assert!(repository.0.lock().unwrap().is_empty());
    }
}
