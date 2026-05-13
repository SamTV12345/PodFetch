use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{Narrator, NarratorRepository};
use uuid::Uuid;

diesel::table! {
    audiobookshelf_narrators (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    audiobookshelf_book_narrators (book_id, narrator_id) {
        book_id -> Text,
        narrator_id -> Text,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_narrators)]
struct NarratorEntity {
    id: String,
    name: String,
}

impl From<NarratorEntity> for Narrator {
    fn from(v: NarratorEntity) -> Self {
        Self {
            id: v.id,
            name: v.name,
        }
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_book_narrators)]
struct BookNarratorEntity {
    book_id: String,
    narrator_id: String,
}

pub struct DieselNarratorRepository {
    database: Database,
}

impl DieselNarratorRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl NarratorRepository for DieselNarratorRepository {
    type Error = PersistenceError;

    fn upsert_by_name(&self, lookup_name: &str) -> Result<Narrator, Self::Error> {
        use self::audiobookshelf_narrators::dsl::*;
        let mut conn = self.database.connection()?;
        if let Some(existing) = audiobookshelf_narrators
            .filter(name.eq(lookup_name))
            .first::<NarratorEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
        {
            return Ok(existing.into());
        }
        let entity = NarratorEntity {
            id: format!("nar_{}", Uuid::new_v4().simple()),
            name: lookup_name.to_string(),
        };
        diesel::insert_into(audiobookshelf_narrators)
            .values(entity.clone())
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        Ok(entity.into())
    }

    fn list_for_book(&self, lookup_book_id: &str) -> Result<Vec<Narrator>, Self::Error> {
        use self::audiobookshelf_book_narrators::dsl as bn_dsl;
        use self::audiobookshelf_narrators::dsl as nar_dsl;
        let mut conn = self.database.connection()?;
        let nar_ids: Vec<String> = bn_dsl::audiobookshelf_book_narrators
            .filter(bn_dsl::book_id.eq(lookup_book_id))
            .select(bn_dsl::narrator_id)
            .load::<String>(&mut conn)
            .map_err(PersistenceError::from)?;
        if nar_ids.is_empty() {
            return Ok(Vec::new());
        }
        nar_dsl::audiobookshelf_narrators
            .filter(nar_dsl::id.eq_any(nar_ids))
            .order(nar_dsl::name.asc())
            .load::<NarratorEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn link(&self, book_id_value: &str, narrator_id_value: &str) -> Result<(), Self::Error> {
        use self::audiobookshelf_book_narrators::dsl::*;
        let mut conn = self.database.connection()?;
        let exists = audiobookshelf_book_narrators
            .filter(book_id.eq(book_id_value))
            .filter(narrator_id.eq(narrator_id_value))
            .first::<BookNarratorEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?
            .is_some();
        if !exists {
            diesel::insert_into(audiobookshelf_book_narrators)
                .values(BookNarratorEntity {
                    book_id: book_id_value.to_string(),
                    narrator_id: narrator_id_value.to_string(),
                })
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(())
    }

    fn unlink_all_for_book(&self, lookup_book_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_book_narrators::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_book_narrators.filter(book_id.eq(lookup_book_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
