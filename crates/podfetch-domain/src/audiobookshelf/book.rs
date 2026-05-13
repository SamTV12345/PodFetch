use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Book {
    pub id: String,
    pub library_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub published_year: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: bool,
    pub cover_path: Option<String>,
    pub duration_seconds: f64,
    pub ino: Option<String>,
    pub folder_path: String,
    pub last_scan: Option<NaiveDateTime>,
    pub added_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub asin: Option<String>,
    pub description: Option<String>,
    pub image_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Narrator {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookSeriesLink {
    pub book_id: String,
    pub series_id: String,
    pub sequence: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookAudioFile {
    pub id: String,
    pub book_id: String,
    pub idx: i32,
    pub ino: Option<String>,
    pub path: String,
    pub relative_path: String,
    pub ext: String,
    pub mime_type: String,
    pub duration: f64,
    pub bitrate: i32,
    pub codec: String,
    pub channels: i32,
    pub sample_rate: i32,
    pub track_num: Option<i32>,
    pub disc_num: Option<i32>,
    pub embedded_cover_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BookChapter {
    pub id: String,
    pub book_id: String,
    pub idx: i32,
    pub start_time: f64,
    pub end_time: f64,
    pub title: String,
}

/// Aggregate snapshot of a fully-hydrated book: all joined entities resolved.
#[derive(Debug, Clone)]
pub struct BookAggregate {
    pub book: Book,
    pub authors: Vec<Author>,
    pub narrators: Vec<Narrator>,
    pub series: Vec<(Series, Option<String>)>, // (series, sequence)
    pub audio_files: Vec<BookAudioFile>,
    pub chapters: Vec<BookChapter>,
}

pub trait BookRepository: Send + Sync {
    type Error;

    fn list_by_library(&self, library_id: &str) -> Result<Vec<Book>, Self::Error>;
    fn find_by_id(&self, id: &str) -> Result<Option<Book>, Self::Error>;
    fn find_by_folder_path(&self, folder_path: &str) -> Result<Option<Book>, Self::Error>;
    fn upsert(&self, book: Book) -> Result<Book, Self::Error>;
    fn delete(&self, id: &str) -> Result<usize, Self::Error>;
}

pub trait AuthorRepository: Send + Sync {
    type Error;

    fn upsert_by_name(&self, name: &str) -> Result<Author, Self::Error>;
    fn list_for_book(&self, book_id: &str) -> Result<Vec<Author>, Self::Error>;
    fn link(&self, book_id: &str, author_id: &str) -> Result<(), Self::Error>;
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error>;
}

pub trait NarratorRepository: Send + Sync {
    type Error;

    fn upsert_by_name(&self, name: &str) -> Result<Narrator, Self::Error>;
    fn list_for_book(&self, book_id: &str) -> Result<Vec<Narrator>, Self::Error>;
    fn link(&self, book_id: &str, narrator_id: &str) -> Result<(), Self::Error>;
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error>;
}

pub trait SeriesRepository: Send + Sync {
    type Error;

    fn upsert_by_name(&self, name: &str) -> Result<Series, Self::Error>;
    fn list_for_book(&self, book_id: &str) -> Result<Vec<(Series, Option<String>)>, Self::Error>;
    fn link(
        &self,
        book_id: &str,
        series_id: &str,
        sequence: Option<&str>,
    ) -> Result<(), Self::Error>;
    fn unlink_all_for_book(&self, book_id: &str) -> Result<usize, Self::Error>;
}

pub trait BookAudioFileRepository: Send + Sync {
    type Error;

    fn replace_for_book(
        &self,
        book_id: &str,
        files: Vec<BookAudioFile>,
    ) -> Result<Vec<BookAudioFile>, Self::Error>;
    fn list_for_book(&self, book_id: &str) -> Result<Vec<BookAudioFile>, Self::Error>;
}

pub trait BookChapterRepository: Send + Sync {
    type Error;

    fn replace_for_book(
        &self,
        book_id: &str,
        chapters: Vec<BookChapter>,
    ) -> Result<Vec<BookChapter>, Self::Error>;
    fn list_for_book(&self, book_id: &str) -> Result<Vec<BookChapter>, Self::Error>;
}
