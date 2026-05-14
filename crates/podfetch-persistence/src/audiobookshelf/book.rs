use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{Book, BookRepository};
use uuid::Uuid;

diesel::table! {
    audiobookshelf_books (id) {
        id -> Text,
        library_id -> Text,
        title -> Text,
        subtitle -> Nullable<Text>,
        description -> Nullable<Text>,
        publisher -> Nullable<Text>,
        published_year -> Nullable<Text>,
        published_date -> Nullable<Text>,
        isbn -> Nullable<Text>,
        asin -> Nullable<Text>,
        language -> Nullable<Text>,
        explicit -> Bool,
        cover_path -> Nullable<Text>,
        duration_seconds -> Double,
        ino -> Nullable<Text>,
        folder_path -> Text,
        last_scan -> Nullable<Timestamp>,
        added_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = audiobookshelf_books, treat_none_as_null = true)]
struct BookEntity {
    id: String,
    library_id: String,
    title: String,
    subtitle: Option<String>,
    description: Option<String>,
    publisher: Option<String>,
    published_year: Option<String>,
    published_date: Option<String>,
    isbn: Option<String>,
    asin: Option<String>,
    language: Option<String>,
    explicit: bool,
    cover_path: Option<String>,
    duration_seconds: f64,
    ino: Option<String>,
    folder_path: String,
    last_scan: Option<NaiveDateTime>,
    added_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<BookEntity> for Book {
    fn from(v: BookEntity) -> Self {
        Self {
            id: v.id,
            library_id: v.library_id,
            title: v.title,
            subtitle: v.subtitle,
            description: v.description,
            publisher: v.publisher,
            published_year: v.published_year,
            published_date: v.published_date,
            isbn: v.isbn,
            asin: v.asin,
            language: v.language,
            explicit: v.explicit,
            cover_path: v.cover_path,
            duration_seconds: v.duration_seconds,
            ino: v.ino,
            folder_path: v.folder_path,
            last_scan: v.last_scan,
            added_at: v.added_at,
            updated_at: v.updated_at,
        }
    }
}

impl From<Book> for BookEntity {
    fn from(v: Book) -> Self {
        Self {
            id: v.id,
            library_id: v.library_id,
            title: v.title,
            subtitle: v.subtitle,
            description: v.description,
            publisher: v.publisher,
            published_year: v.published_year,
            published_date: v.published_date,
            isbn: v.isbn,
            asin: v.asin,
            language: v.language,
            explicit: v.explicit,
            cover_path: v.cover_path,
            duration_seconds: v.duration_seconds,
            ino: v.ino,
            folder_path: v.folder_path,
            last_scan: v.last_scan,
            added_at: v.added_at,
            updated_at: v.updated_at,
        }
    }
}

pub fn new_book_id() -> String {
    format!("li_book_{}", Uuid::new_v4().simple())
}

pub struct DieselBookRepository {
    database: Database,
}

impl DieselBookRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl BookRepository for DieselBookRepository {
    type Error = PersistenceError;

    fn list_by_library(&self, lookup_library_id: &str) -> Result<Vec<Book>, Self::Error> {
        use self::audiobookshelf_books::dsl::*;
        let mut conn = self.database.connection()?;
        audiobookshelf_books
            .filter(library_id.eq(lookup_library_id))
            .order(title.asc())
            .load::<BookEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_by_id(&self, lookup_id: &str) -> Result<Option<Book>, Self::Error> {
        use self::audiobookshelf_books::dsl::*;
        let mut conn = self.database.connection()?;
        audiobookshelf_books
            .filter(id.eq(lookup_id))
            .first::<BookEntity>(&mut conn)
            .optional()
            .map(|r| r.map(Into::into))
            .map_err(Into::into)
    }

    fn find_by_folder_path(&self, lookup_folder: &str) -> Result<Option<Book>, Self::Error> {
        use self::audiobookshelf_books::dsl::*;
        let mut conn = self.database.connection()?;
        audiobookshelf_books
            .filter(folder_path.eq(lookup_folder))
            .first::<BookEntity>(&mut conn)
            .optional()
            .map(|r| r.map(Into::into))
            .map_err(Into::into)
    }

    fn upsert(&self, book: Book) -> Result<Book, Self::Error> {
        use self::audiobookshelf_books::dsl::*;
        let mut conn = self.database.connection()?;
        let entity = BookEntity::from(book);
        let exists = audiobookshelf_books
            .filter(id.eq(&entity.id))
            .first::<BookEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
            .is_some();
        if exists {
            diesel::update(audiobookshelf_books.filter(id.eq(&entity.id)))
                .set(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        } else {
            diesel::insert_into(audiobookshelf_books)
                .values(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(entity.into())
    }

    fn delete(&self, lookup_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_books::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_books.filter(id.eq(lookup_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
