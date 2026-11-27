use common_infrastructure::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── Audiobookshelf: Library ─────────────────────────────────────────────────

use crate::domain::library::{Library, LibraryRepository, MediaType};
use crate::persistence::library::DieselLibraryRepository;

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

use crate::domain::media_progress::{MediaProgress, MediaProgressRepository};
use crate::persistence::media_progress::DieselMediaProgressRepository;

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

use crate::domain::playback_session::{PlaybackSession, PlaybackSessionRepository};
use crate::persistence::playback_session::DieselPlaybackSessionRepository;

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

use crate::domain::book::{
    Author, AuthorRepository, Book, BookAudioFile, BookAudioFileRepository, BookChapter,
    BookChapterRepository, BookRepository, Narrator, NarratorRepository, Series, SeriesRepository,
};
use crate::persistence::author::DieselAuthorRepository;
use crate::persistence::book::DieselBookRepository;
use crate::persistence::book_audio_file::DieselBookAudioFileRepository;
use crate::persistence::book_chapter::DieselBookChapterRepository;
use crate::persistence::narrator::DieselNarratorRepository;
use crate::persistence::series::DieselSeriesRepository;

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

use crate::domain::listening_session::{ListeningSession, ListeningSessionRepository};
use crate::persistence::listening_session::DieselListeningSessionRepository;

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
