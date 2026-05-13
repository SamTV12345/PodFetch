use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{Author, AuthorRepository};
use uuid::Uuid;

diesel::table! {
    audiobookshelf_authors (id) {
        id -> Text,
        name -> Text,
        asin -> Nullable<Text>,
        description -> Nullable<Text>,
        image_path -> Nullable<Text>,
    }
}

diesel::table! {
    audiobookshelf_book_authors (book_id, author_id) {
        book_id -> Text,
        author_id -> Text,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_authors)]
struct AuthorEntity {
    id: String,
    name: String,
    asin: Option<String>,
    description: Option<String>,
    image_path: Option<String>,
}

impl From<AuthorEntity> for Author {
    fn from(v: AuthorEntity) -> Self {
        Self {
            id: v.id,
            name: v.name,
            asin: v.asin,
            description: v.description,
            image_path: v.image_path,
        }
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_book_authors)]
struct BookAuthorEntity {
    book_id: String,
    author_id: String,
}

pub struct DieselAuthorRepository {
    database: Database,
}

impl DieselAuthorRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl AuthorRepository for DieselAuthorRepository {
    type Error = PersistenceError;

    fn upsert_by_name(&self, lookup_name: &str) -> Result<Author, Self::Error> {
        use self::audiobookshelf_authors::dsl::*;
        let mut conn = self.database.connection()?;
        if let Some(existing) = audiobookshelf_authors
            .filter(name.eq(lookup_name))
            .first::<AuthorEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
        {
            return Ok(existing.into());
        }
        let entity = AuthorEntity {
            id: format!("aut_{}", Uuid::new_v4().simple()),
            name: lookup_name.to_string(),
            asin: None,
            description: None,
            image_path: None,
        };
        diesel::insert_into(audiobookshelf_authors)
            .values(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn list_for_book(&self, lookup_book_id: &str) -> Result<Vec<Author>, Self::Error> {
        use self::audiobookshelf_authors::dsl as authors_dsl;
        use self::audiobookshelf_book_authors::dsl as ba_dsl;
        let mut conn = self.database.connection()?;
        let author_ids: Vec<String> = ba_dsl::audiobookshelf_book_authors
            .filter(ba_dsl::book_id.eq(lookup_book_id))
            .select(ba_dsl::author_id)
            .load::<String>(&mut conn)
            .map_err(PersistenceError::from)?;
        if author_ids.is_empty() {
            return Ok(Vec::new());
        }
        authors_dsl::audiobookshelf_authors
            .filter(authors_dsl::id.eq_any(author_ids))
            .order(authors_dsl::name.asc())
            .load::<AuthorEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn link(&self, book_id_value: &str, author_id_value: &str) -> Result<(), Self::Error> {
        use self::audiobookshelf_book_authors::dsl::*;
        let mut conn = self.database.connection()?;
        // Insert ignoring conflict (manual)
        let exists = audiobookshelf_book_authors
            .filter(book_id.eq(book_id_value))
            .filter(author_id.eq(author_id_value))
            .first::<BookAuthorEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
            .is_some();
        if !exists {
            diesel::insert_into(audiobookshelf_book_authors)
                .values(BookAuthorEntity {
                    book_id: book_id_value.to_string(),
                    author_id: author_id_value.to_string(),
                })
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(())
    }

    fn unlink_all_for_book(&self, lookup_book_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_book_authors::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_book_authors.filter(book_id.eq(lookup_book_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
