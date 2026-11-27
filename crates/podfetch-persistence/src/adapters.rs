use crate::db::Database;
use chrono::NaiveDateTime;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── DeviceSyncGroup ─────────────────────────────────────────────────────────

use crate::device_sync_group::DieselDeviceSyncGroupRepository;
use podfetch_domain::device_sync_group::{DeviceSyncGroup, DeviceSyncGroupRepository};

pub struct DeviceSyncGroupRepositoryImpl {
    inner: DieselDeviceSyncGroupRepository,
}

impl DeviceSyncGroupRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceSyncGroupRepository::new(database),
        }
    }
}

impl DeviceSyncGroupRepository for DeviceSyncGroupRepositoryImpl {
    type Error = CustomError;

    fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<DeviceSyncGroup>, Self::Error> {
        self.inner.get_by_user_id(user_id).map_err(Into::into)
    }

    fn replace_all(&self, user_id: Uuid, groups: Vec<DeviceSyncGroup>) -> Result<(), Self::Error> {
        self.inner.replace_all(user_id, groups).map_err(Into::into)
    }
}

// ── Device ──────────────────────────────────────────────────────────────────

use crate::device::DieselDeviceRepository;
use podfetch_domain::device::{Device, DeviceRepository};

pub struct DeviceRepositoryImpl {
    inner: DieselDeviceRepository,
}

impl DeviceRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselDeviceRepository::new(database),
        }
    }
}

impl DeviceRepository for DeviceRepositoryImpl {
    type Error = CustomError;

    fn create(&self, device: Device) -> Result<Device, CustomError> {
        self.inner.create(device).map_err(Into::into)
    }

    fn get_devices_of_user(&self, user_id_to_find: Uuid) -> Result<Vec<Device>, CustomError> {
        self.inner
            .get_devices_of_user(user_id_to_find)
            .map_err(Into::into)
    }

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), CustomError> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }
}

// ── GpodderSetting ──────────────────────────────────────────────────────────

use crate::gpodder_setting::DieselGpodderSettingRepository;
use podfetch_domain::gpodder_setting::{GpodderSetting, GpodderSettingRepository};

pub struct GpodderSettingRepositoryImpl {
    inner: DieselGpodderSettingRepository,
}

impl GpodderSettingRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselGpodderSettingRepository::new(database),
        }
    }
}

impl GpodderSettingRepository for GpodderSettingRepositoryImpl {
    type Error = CustomError;

    fn get_setting(
        &self,
        user_id: Uuid,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<Option<GpodderSetting>, Self::Error> {
        self.inner
            .get_setting(user_id, scope, scope_id)
            .map_err(Into::into)
    }

    fn save_setting(&self, setting: GpodderSetting) -> Result<GpodderSetting, Self::Error> {
        self.inner.save_setting(setting).map_err(Into::into)
    }
}

// ── PodcastEpisodeChapter ───────────────────────────────────────────────────

use crate::podcast_episode_chapter::DieselPodcastEpisodeChapterRepository;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapter, PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};

pub struct PodcastEpisodeChapterRepositoryImpl {
    inner: DieselPodcastEpisodeChapterRepository,
}

impl PodcastEpisodeChapterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeChapterRepository::new(database),
        }
    }
}

impl PodcastEpisodeChapterRepository for PodcastEpisodeChapterRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        self.inner.upsert(chapter).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }
}

// ── PodcastSettings ─────────────────────────────────────────────────────────

use crate::podcast_settings::DieselPodcastSettingsRepository;
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};

pub struct PodcastSettingsRepositoryImpl {
    inner: DieselPodcastSettingsRepository,
}

impl PodcastSettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastSettingsRepository::new(database),
        }
    }
}

impl PodcastSettingsRepository for PodcastSettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self, podcast_id: Uuid) -> Result<Option<PodcastSetting>, Self::Error> {
        self.inner.get_settings(podcast_id).map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        self.inner.upsert_settings(setting).map_err(Into::into)
    }
}

// ── Settings ────────────────────────────────────────────────────────────────

use crate::settings::DieselSettingsRepository;
use podfetch_domain::settings::{Setting, SettingRepository};

pub struct SettingsRepositoryImpl {
    inner: DieselSettingsRepository,
}

impl SettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSettingsRepository::new(database),
        }
    }
}

impl SettingRepository for SettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        self.inner.get_settings().map_err(Into::into)
    }

    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error> {
        self.inner.update_settings(setting).map_err(Into::into)
    }

    fn insert_default_settings(&self) -> Result<(), Self::Error> {
        self.inner.insert_default_settings().map_err(Into::into)
    }
}

// ── Subscription ────────────────────────────────────────────────────────────

use crate::subscription::DieselSubscriptionRepository;
use podfetch_domain::subscription::{
    GPodderAvailablePodcast, SubscriptionModelChanges, SubscriptionRepository,
};

pub struct SubscriptionRepositoryImpl {
    inner: DieselSubscriptionRepository,
}

impl SubscriptionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSubscriptionRepository::new(database),
        }
    }
}

impl SubscriptionRepository for SubscriptionRepositoryImpl {
    type Error = CustomError;

    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error> {
        self.inner.delete_by_user_id(user_id).map_err(Into::into)
    }

    fn get_device_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_device_subscriptions(device_id, user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn get_user_subscriptions(
        &self,
        user_id: Uuid,
        since: NaiveDateTime,
        timestamp: i64,
    ) -> Result<SubscriptionModelChanges, Self::Error> {
        self.inner
            .get_user_subscriptions(user_id, since, timestamp)
            .map_err(Into::into)
    }

    fn update_subscriptions(
        &self,
        device_id: &str,
        user_id: Uuid,
        add: &[String],
        remove: &[String],
    ) -> Result<Vec<Vec<String>>, Self::Error> {
        self.inner
            .update_subscriptions(device_id, user_id, add, remove)
            .map_err(Into::into)
    }

    fn get_available_gpodder_podcasts(&self) -> Result<Vec<GPodderAvailablePodcast>, Self::Error> {
        self.inner
            .get_available_gpodder_podcasts()
            .map_err(Into::into)
    }

    fn get_active_device_podcast_urls(
        &self,
        device_id: &str,
        user_id: Uuid,
    ) -> Result<Vec<String>, Self::Error> {
        self.inner
            .get_active_device_podcast_urls(device_id, user_id)
            .map_err(Into::into)
    }
}

// ── Audiobookshelf: Library ─────────────────────────────────────────────────

use crate::audiobookshelf::library::DieselLibraryRepository;
use podfetch_domain::audiobookshelf::library::{Library, LibraryRepository, MediaType};

pub struct LibraryRepositoryImpl {
    inner: DieselLibraryRepository,
}

impl LibraryRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselLibraryRepository::new(database),
        }
    }
}

impl LibraryRepository for LibraryRepositoryImpl {
    type Error = CustomError;

    fn list(&self) -> Result<Vec<Library>, Self::Error> {
        self.inner.list().map_err(Into::into)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Library>, Self::Error> {
        self.inner.find_by_id(id).map_err(Into::into)
    }

    fn find_first_by_media_type(
        &self,
        media_type: &MediaType,
    ) -> Result<Option<Library>, Self::Error> {
        self.inner
            .find_first_by_media_type(media_type)
            .map_err(Into::into)
    }

    fn upsert(&self, library: Library) -> Result<Library, Self::Error> {
        self.inner.upsert(library).map_err(Into::into)
    }

    fn delete(&self, id: &str) -> Result<usize, Self::Error> {
        self.inner.delete(id).map_err(Into::into)
    }
}

// ── Audiobookshelf: MediaProgress ───────────────────────────────────────────

use crate::audiobookshelf::media_progress::DieselMediaProgressRepository;
use podfetch_domain::audiobookshelf::media_progress::{MediaProgress, MediaProgressRepository};

pub struct MediaProgressRepositoryImpl {
    inner: DieselMediaProgressRepository,
}

impl MediaProgressRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselMediaProgressRepository::new(database),
        }
    }
}

impl MediaProgressRepository for MediaProgressRepositoryImpl {
    type Error = CustomError;

    fn list_for_user(&self, user_id: Uuid) -> Result<Vec<MediaProgress>, Self::Error> {
        self.inner.list_for_user(user_id).map_err(Into::into)
    }

    fn find(
        &self,
        user_id: Uuid,
        library_item_id: &str,
        episode_id: Option<&str>,
    ) -> Result<Option<MediaProgress>, Self::Error> {
        self.inner
            .find(user_id, library_item_id, episode_id)
            .map_err(Into::into)
    }

    fn upsert(&self, progress: MediaProgress) -> Result<MediaProgress, Self::Error> {
        self.inner.upsert(progress).map_err(Into::into)
    }

    fn delete(&self, id: &str) -> Result<usize, Self::Error> {
        self.inner.delete(id).map_err(Into::into)
    }
}

// ── Audiobookshelf: PlaybackSession ─────────────────────────────────────────

use crate::audiobookshelf::playback_session::DieselPlaybackSessionRepository;
use podfetch_domain::audiobookshelf::playback_session::{
    PlaybackSession, PlaybackSessionRepository,
};

pub struct PlaybackSessionRepositoryImpl {
    inner: DieselPlaybackSessionRepository,
}

impl PlaybackSessionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPlaybackSessionRepository::new(database),
        }
    }
}

impl PlaybackSessionRepository for PlaybackSessionRepositoryImpl {
    type Error = CustomError;

    fn create(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error> {
        self.inner.create(session).map_err(Into::into)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<PlaybackSession>, Self::Error> {
        self.inner.find_by_id(id).map_err(Into::into)
    }

    fn update(&self, session: PlaybackSession) -> Result<PlaybackSession, Self::Error> {
        self.inner.update(session).map_err(Into::into)
    }

    fn delete(&self, id: &str) -> Result<usize, Self::Error> {
        self.inner.delete(id).map_err(Into::into)
    }
}

// ── Audiobookshelf: Book + relations ────────────────────────────────────────

use crate::audiobookshelf::author::DieselAuthorRepository;
use crate::audiobookshelf::book::DieselBookRepository;
use crate::audiobookshelf::book_audio_file::DieselBookAudioFileRepository;
use crate::audiobookshelf::book_chapter::DieselBookChapterRepository;
use crate::audiobookshelf::narrator::DieselNarratorRepository;
use crate::audiobookshelf::series::DieselSeriesRepository;
use podfetch_domain::audiobookshelf::book::{
    Author, AuthorRepository, Book, BookAudioFile, BookAudioFileRepository, BookChapter,
    BookChapterRepository, BookRepository, Narrator, NarratorRepository, Series, SeriesRepository,
};

pub struct BookRepositoryImpl {
    inner: DieselBookRepository,
}
impl BookRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselBookRepository::new(database),
        }
    }
}
impl BookRepository for BookRepositoryImpl {
    type Error = CustomError;
    fn list_by_library(&self, library_id: &str) -> Result<Vec<Book>, Self::Error> {
        self.inner.list_by_library(library_id).map_err(Into::into)
    }
    fn find_by_id(&self, id: &str) -> Result<Option<Book>, Self::Error> {
        self.inner.find_by_id(id).map_err(Into::into)
    }
    fn find_by_folder_path(&self, p: &str) -> Result<Option<Book>, Self::Error> {
        self.inner.find_by_folder_path(p).map_err(Into::into)
    }
    fn upsert(&self, book: Book) -> Result<Book, Self::Error> {
        self.inner.upsert(book).map_err(Into::into)
    }
    fn delete(&self, id: &str) -> Result<usize, Self::Error> {
        self.inner.delete(id).map_err(Into::into)
    }
}

pub struct AuthorRepositoryImpl {
    inner: DieselAuthorRepository,
}
impl AuthorRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselAuthorRepository::new(database),
        }
    }
}
impl AuthorRepository for AuthorRepositoryImpl {
    type Error = CustomError;
    fn upsert_by_name(&self, name: &str) -> Result<Author, Self::Error> {
        self.inner.upsert_by_name(name).map_err(Into::into)
    }
    fn list_for_book(&self, book_id: &str) -> Result<Vec<Author>, Self::Error> {
        self.inner.list_for_book(book_id).map_err(Into::into)
    }
    fn link(&self, book_id: &str, author_id: &str) -> Result<(), Self::Error> {
        self.inner.link(book_id, author_id).map_err(Into::into)
    }
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error> {
        self.inner.unlink_all_for_book(book_id).map_err(Into::into)
    }
}

pub struct NarratorRepositoryImpl {
    inner: DieselNarratorRepository,
}
impl NarratorRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselNarratorRepository::new(database),
        }
    }
}
impl NarratorRepository for NarratorRepositoryImpl {
    type Error = CustomError;
    fn upsert_by_name(&self, name: &str) -> Result<Narrator, Self::Error> {
        self.inner.upsert_by_name(name).map_err(Into::into)
    }
    fn list_for_book(&self, book_id: &str) -> Result<Vec<Narrator>, Self::Error> {
        self.inner.list_for_book(book_id).map_err(Into::into)
    }
    fn link(&self, book_id: &str, narrator_id: &str) -> Result<(), Self::Error> {
        self.inner.link(book_id, narrator_id).map_err(Into::into)
    }
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error> {
        self.inner.unlink_all_for_book(book_id).map_err(Into::into)
    }
}

pub struct SeriesRepositoryImpl {
    inner: DieselSeriesRepository,
}
impl SeriesRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSeriesRepository::new(database),
        }
    }
}
impl SeriesRepository for SeriesRepositoryImpl {
    type Error = CustomError;
    fn upsert_by_name(&self, name: &str) -> Result<Series, Self::Error> {
        self.inner.upsert_by_name(name).map_err(Into::into)
    }
    fn list_for_book(&self, book_id: &str) -> Result<Vec<(Series, Option<String>)>, Self::Error> {
        self.inner.list_for_book(book_id).map_err(Into::into)
    }
    fn link(
        &self,
        book_id: &str,
        series_id: &str,
        sequence: Option<&str>,
    ) -> Result<(), Self::Error> {
        self.inner
            .link(book_id, series_id, sequence)
            .map_err(Into::into)
    }
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error> {
        self.inner.unlink_all_for_book(book_id).map_err(Into::into)
    }
}

pub struct BookAudioFileRepositoryImpl {
    inner: DieselBookAudioFileRepository,
}
impl BookAudioFileRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselBookAudioFileRepository::new(database),
        }
    }
}
impl BookAudioFileRepository for BookAudioFileRepositoryImpl {
    type Error = CustomError;
    fn replace_for_book(
        &self,
        book_id: &str,
        files: Vec<BookAudioFile>,
    ) -> Result<Vec<BookAudioFile>, Self::Error> {
        self.inner
            .replace_for_book(book_id, files)
            .map_err(Into::into)
    }
    fn list_for_book(&self, book_id: &str) -> Result<Vec<BookAudioFile>, Self::Error> {
        self.inner.list_for_book(book_id).map_err(Into::into)
    }
}

pub struct BookChapterRepositoryImpl {
    inner: DieselBookChapterRepository,
}
impl BookChapterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselBookChapterRepository::new(database),
        }
    }
}
impl BookChapterRepository for BookChapterRepositoryImpl {
    type Error = CustomError;
    fn replace_for_book(
        &self,
        book_id: &str,
        chapters: Vec<BookChapter>,
    ) -> Result<Vec<BookChapter>, Self::Error> {
        self.inner
            .replace_for_book(book_id, chapters)
            .map_err(Into::into)
    }
    fn list_for_book(&self, book_id: &str) -> Result<Vec<BookChapter>, Self::Error> {
        self.inner.list_for_book(book_id).map_err(Into::into)
    }
}

// ── Audiobookshelf: ListeningSession ────────────────────────────────────────

use crate::audiobookshelf::listening_session::DieselListeningSessionRepository;
use podfetch_domain::audiobookshelf::listening_session::{
    ListeningSession, ListeningSessionRepository,
};

pub struct ListeningSessionRepositoryImpl {
    inner: DieselListeningSessionRepository,
}

impl ListeningSessionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselListeningSessionRepository::new(database),
        }
    }
}

impl ListeningSessionRepository for ListeningSessionRepositoryImpl {
    type Error = CustomError;

    fn create(&self, session: ListeningSession) -> Result<ListeningSession, Self::Error> {
        self.inner.create(session).map_err(Into::into)
    }

    fn list_for_user(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ListeningSession>, Self::Error> {
        self.inner.list_for_user(user_id, limit).map_err(Into::into)
    }
}

// ── PodcastEpisodeTranscript ─────────────────────────────────────────────────

use crate::podcast_episode_transcript::DieselPodcastEpisodeTranscriptRepository;
use podfetch_domain::podcast_episode_transcript::{
    PodcastEpisodeTranscript, PodcastEpisodeTranscriptRepository, TranscriptSearchHit,
    TranscriptSegment, TranscriptStatus, UpsertTranscript,
};

pub struct PodcastEpisodeTranscriptRepositoryImpl {
    inner: DieselPodcastEpisodeTranscriptRepository,
}

impl PodcastEpisodeTranscriptRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeTranscriptRepository::new(database),
        }
    }
}

impl PodcastEpisodeTranscriptRepository for PodcastEpisodeTranscriptRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, transcript: UpsertTranscript) -> Result<Uuid, Self::Error> {
        self.inner.upsert(transcript).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }

    fn get_all(&self) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_all().map_err(Into::into)
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_by_id(id).map_err(Into::into)
    }

    fn set_file_path(&self, id: Uuid, file_path: &str) -> Result<(), Self::Error> {
        self.inner.set_file_path(id, file_path).map_err(Into::into)
    }

    fn set_status(
        &self,
        id: Uuid,
        status: TranscriptStatus,
        error: Option<&str>,
    ) -> Result<(), Self::Error> {
        self.inner.set_status(id, status, error).map_err(Into::into)
    }

    fn set_preferred(
        &self,
        episode_id: Uuid,
        preferred_id: Option<Uuid>,
    ) -> Result<(), Self::Error> {
        self.inner
            .set_preferred(episode_id, preferred_id)
            .map_err(Into::into)
    }

    fn replace_segments(
        &self,
        transcript_id: Uuid,
        segments: &[TranscriptSegment],
    ) -> Result<(), Self::Error> {
        self.inner
            .replace_segments(transcript_id, segments)
            .map_err(Into::into)
    }

    fn get_segments(&self, transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error> {
        self.inner.get_segments(transcript_id).map_err(Into::into)
    }

    fn search(
        &self,
        query: &str,
        podcast_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<TranscriptSearchHit>, Self::Error> {
        self.inner
            .search(query, podcast_id, page, page_size)
            .map_err(Into::into)
    }
}

// ── TranscriptionJob ──────────────────────────────────────────────────────────

use crate::podcast_episode_transcript::DieselTranscriptionJobRepository;
use podfetch_domain::podcast_episode_transcript::{
    TranscriptionJob, TranscriptionJobRepository, TranscriptionJobStatus,
};

pub struct TranscriptionJobRepositoryImpl {
    inner: DieselTranscriptionJobRepository,
}

impl TranscriptionJobRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselTranscriptionJobRepository::new(database),
        }
    }
}

impl TranscriptionJobRepository for TranscriptionJobRepositoryImpl {
    type Error = CustomError;

    fn enqueue(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.enqueue(episode_id).map_err(Into::into)
    }

    fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.next_pending().map_err(Into::into)
    }

    fn set_status(
        &self,
        id: Uuid,
        status: TranscriptionJobStatus,
        error: Option<&str>,
    ) -> Result<(), Self::Error> {
        self.inner.set_status(id, status, error).map_err(Into::into)
    }

    fn increment_attempts(&self, id: Uuid) -> Result<i32, Self::Error> {
        self.inner.increment_attempts(id).map_err(Into::into)
    }

    fn reset_running_to_pending(&self) -> Result<usize, Self::Error> {
        self.inner.reset_running_to_pending().map_err(Into::into)
    }

    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }
}
