use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{AsChangeset, Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::library::{Library, LibraryRepository, MediaType};

diesel::table! {
    audiobookshelf_libraries (id) {
        id -> Text,
        name -> Text,
        media_type -> Text,
        icon -> Text,
        display_order -> Integer,
        folder_paths -> Text,
        metadata_precedence -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

#[derive(Queryable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = audiobookshelf_libraries)]
struct LibraryEntity {
    id: String,
    name: String,
    media_type: String,
    icon: String,
    display_order: i32,
    folder_paths: String,
    metadata_precedence: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl LibraryEntity {
    fn into_domain(self) -> Library {
        Library {
            id: self.id,
            name: self.name,
            media_type: MediaType::parse(&self.media_type).unwrap_or(MediaType::Podcast),
            icon: self.icon,
            display_order: self.display_order,
            folder_paths: serde_json::from_str(&self.folder_paths).unwrap_or_default(),
            metadata_precedence: serde_json::from_str(&self.metadata_precedence).unwrap_or_default(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    fn from_domain(value: Library) -> Self {
        Self {
            id: value.id,
            name: value.name,
            media_type: value.media_type.as_str().to_string(),
            icon: value.icon,
            display_order: value.display_order,
            folder_paths: serde_json::to_string(&value.folder_paths).unwrap_or("[]".to_string()),
            metadata_precedence: serde_json::to_string(&value.metadata_precedence)
                .unwrap_or("[]".to_string()),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub struct DieselLibraryRepository {
    database: Database,
}

impl DieselLibraryRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl LibraryRepository for DieselLibraryRepository {
    type Error = PersistenceError;

    fn list(&self) -> Result<Vec<Library>, Self::Error> {
        use self::audiobookshelf_libraries::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_libraries
            .order(display_order.asc())
            .load::<LibraryEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(LibraryEntity::into_domain).collect())
            .map_err(Into::into)
    }

    fn find_by_id(&self, lookup_id: &str) -> Result<Option<Library>, Self::Error> {
        use self::audiobookshelf_libraries::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_libraries
            .filter(id.eq(lookup_id))
            .first::<LibraryEntity>(&mut conn)
            .optional()
            .map(|row| row.map(LibraryEntity::into_domain))
            .map_err(Into::into)
    }

    fn find_first_by_media_type(
        &self,
        media_type_lookup: &MediaType,
    ) -> Result<Option<Library>, Self::Error> {
        use self::audiobookshelf_libraries::dsl::*;

        let mut conn = self.database.connection()?;
        audiobookshelf_libraries
            .filter(media_type.eq(media_type_lookup.as_str()))
            .order(display_order.asc())
            .first::<LibraryEntity>(&mut conn)
            .optional()
            .map(|row| row.map(LibraryEntity::into_domain))
            .map_err(Into::into)
    }

    fn upsert(&self, library: Library) -> Result<Library, Self::Error> {
        use self::audiobookshelf_libraries::dsl::*;

        let mut conn = self.database.connection()?;
        let entity = LibraryEntity::from_domain(library);
        let existing = audiobookshelf_libraries
            .filter(id.eq(&entity.id))
            .first::<LibraryEntity>(&mut conn)
            .optional()
            .map_err(PersistenceError::from)?;

        if existing.is_some() {
            diesel::update(audiobookshelf_libraries.filter(id.eq(&entity.id)))
                .set(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        } else {
            diesel::insert_into(audiobookshelf_libraries)
                .values(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(entity.into_domain())
    }

    fn delete(&self, lookup_id: &str) -> Result<usize, Self::Error> {
        use self::audiobookshelf_libraries::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_libraries.filter(id.eq(lookup_id)))
            .execute(&mut conn)
            .map_err(Into::into)
    }
}
