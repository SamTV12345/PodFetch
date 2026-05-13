//! Aggregates a book with its joined authors/narrators/series/audio_files/chapters.

use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::book::{
    AuthorRepository, Book, BookAggregate, BookAudioFileRepository, BookChapterRepository,
    BookRepository, NarratorRepository, SeriesRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AudiobookshelfBookService {
    pub book_repository: Arc<dyn BookRepository<Error = CustomError>>,
    pub author_repository: Arc<dyn AuthorRepository<Error = CustomError>>,
    pub narrator_repository: Arc<dyn NarratorRepository<Error = CustomError>>,
    pub series_repository: Arc<dyn SeriesRepository<Error = CustomError>>,
    pub audio_file_repository: Arc<dyn BookAudioFileRepository<Error = CustomError>>,
    pub chapter_repository: Arc<dyn BookChapterRepository<Error = CustomError>>,
}

impl AudiobookshelfBookService {
    pub fn list_by_library(&self, library_id: &str) -> Result<Vec<Book>, CustomError> {
        self.book_repository.list_by_library(library_id)
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Book>, CustomError> {
        self.book_repository.find_by_id(id)
    }

    pub fn hydrate(&self, book: Book) -> Result<BookAggregate, CustomError> {
        let authors = self.author_repository.list_for_book(&book.id)?;
        let narrators = self.narrator_repository.list_for_book(&book.id)?;
        let series = self.series_repository.list_for_book(&book.id)?;
        let audio_files = self.audio_file_repository.list_for_book(&book.id)?;
        let chapters = self.chapter_repository.list_for_book(&book.id)?;
        Ok(BookAggregate {
            book,
            authors,
            narrators,
            series,
            audio_files,
            chapters,
        })
    }
}
