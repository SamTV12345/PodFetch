use chrono::Utc;
use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::library::{Library, LibraryRepository, MediaType};
use std::sync::Arc;

const DEFAULT_PODCASTS_LIBRARY_ID: &str = "lib_default_podcasts";
const DEFAULT_AUDIOBOOKS_LIBRARY_ID: &str = "lib_default_audiobooks";

const DEFAULT_METADATA_PRECEDENCE: &[&str] = &[
    "folderStructure",
    "audioMetatags",
    "nfoFile",
    "txtFiles",
    "opfFile",
    "absMetadata",
];

#[derive(Clone)]
pub struct AudiobookshelfLibraryService {
    repository: Arc<dyn LibraryRepository<Error = CustomError>>,
}

impl AudiobookshelfLibraryService {
    pub fn new(repository: Arc<dyn LibraryRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Result<Vec<Library>, CustomError> {
        self.repository.list()
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Library>, CustomError> {
        self.repository.find_by_id(id)
    }

    pub fn find_default_podcasts_library(&self) -> Result<Option<Library>, CustomError> {
        self.repository
            .find_first_by_media_type(&MediaType::Podcast)
    }

    /// Idempotent: creates the default Podcasts + Audiobooks libraries if none
    /// exist. Called on startup when the audiobookshelf integration is enabled.
    pub fn bootstrap_defaults(&self) -> Result<(), CustomError> {
        let existing = self.repository.list()?;
        if !existing
            .iter()
            .any(|library| matches!(library.media_type, MediaType::Podcast))
        {
            let now = Utc::now().naive_utc();
            self.repository.upsert(Library {
                id: DEFAULT_PODCASTS_LIBRARY_ID.to_string(),
                name: "Podcasts".to_string(),
                media_type: MediaType::Podcast,
                icon: "podcast".to_string(),
                display_order: 1,
                folder_paths: Vec::new(),
                metadata_precedence: Vec::new(),
                created_at: now,
                updated_at: now,
            })?;
        }
        if !existing
            .iter()
            .any(|library| matches!(library.media_type, MediaType::Book))
        {
            let now = Utc::now().naive_utc();
            self.repository.upsert(Library {
                id: DEFAULT_AUDIOBOOKS_LIBRARY_ID.to_string(),
                name: "Audiobooks".to_string(),
                media_type: MediaType::Book,
                icon: "audiobookshelf".to_string(),
                display_order: 2,
                folder_paths: Vec::new(),
                metadata_precedence: DEFAULT_METADATA_PRECEDENCE
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                created_at: now,
                updated_at: now,
            })?;
        }
        Ok(())
    }
}
