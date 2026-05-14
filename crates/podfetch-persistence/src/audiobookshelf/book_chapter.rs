use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{BookChapter, BookChapterRepository};

diesel::table! {
    audiobookshelf_book_chapters (id) {
        id -> Text,
        book_id -> Text,
        idx -> Integer,
        start_time -> Double,
        end_time -> Double,
        title -> Text,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_book_chapters)]
struct BookChapterEntity {
    id: String,
    book_id: String,
    idx: i32,
    start_time: f64,
    end_time: f64,
    title: String,
}

impl From<BookChapterEntity> for BookChapter {
    fn from(v: BookChapterEntity) -> Self {
        Self {
            id: v.id,
            book_id: v.book_id,
            idx: v.idx,
            start_time: v.start_time,
            end_time: v.end_time,
            title: v.title,
        }
    }
}

impl From<BookChapter> for BookChapterEntity {
    fn from(v: BookChapter) -> Self {
        Self {
            id: v.id,
            book_id: v.book_id,
            idx: v.idx,
            start_time: v.start_time,
            end_time: v.end_time,
            title: v.title,
        }
    }
}

pub struct DieselBookChapterRepository {
    database: Database,
}

impl DieselBookChapterRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl BookChapterRepository for DieselBookChapterRepository {
    type Error = PersistenceError;

    fn replace_for_book(
        &self,
        lookup_book_id: &str,
        chapters: Vec<BookChapter>,
    ) -> Result<Vec<BookChapter>, Self::Error> {
        use self::audiobookshelf_book_chapters::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_book_chapters.filter(book_id.eq(lookup_book_id)))
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        let entities: Vec<BookChapterEntity> = chapters.into_iter().map(Into::into).collect();
        for entity in &entities {
            diesel::insert_into(audiobookshelf_book_chapters)
                .values(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(entities.into_iter().map(Into::into).collect())
    }

    fn list_for_book(&self, lookup_book_id: &str) -> Result<Vec<BookChapter>, Self::Error> {
        use self::audiobookshelf_book_chapters::dsl::*;
        let mut conn = self.database.connection()?;
        audiobookshelf_book_chapters
            .filter(book_id.eq(lookup_book_id))
            .order(idx.asc())
            .load::<BookChapterEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
