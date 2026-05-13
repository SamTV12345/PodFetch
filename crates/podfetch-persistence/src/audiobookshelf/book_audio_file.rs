use crate::db::{Database, PersistenceError};
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use podfetch_domain::audiobookshelf::book::{BookAudioFile, BookAudioFileRepository};

diesel::table! {
    audiobookshelf_book_audio_files (id) {
        id -> Text,
        book_id -> Text,
        idx -> Integer,
        ino -> Nullable<Text>,
        path -> Text,
        relative_path -> Text,
        ext -> Text,
        mime_type -> Text,
        duration -> Double,
        bitrate -> Integer,
        codec -> Text,
        channels -> Integer,
        sample_rate -> Integer,
        track_num -> Nullable<Integer>,
        disc_num -> Nullable<Integer>,
        embedded_cover_path -> Nullable<Text>,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = audiobookshelf_book_audio_files)]
struct BookAudioFileEntity {
    id: String,
    book_id: String,
    idx: i32,
    ino: Option<String>,
    path: String,
    relative_path: String,
    ext: String,
    mime_type: String,
    duration: f64,
    bitrate: i32,
    codec: String,
    channels: i32,
    sample_rate: i32,
    track_num: Option<i32>,
    disc_num: Option<i32>,
    embedded_cover_path: Option<String>,
}

impl From<BookAudioFileEntity> for BookAudioFile {
    fn from(v: BookAudioFileEntity) -> Self {
        Self {
            id: v.id,
            book_id: v.book_id,
            idx: v.idx,
            ino: v.ino,
            path: v.path,
            relative_path: v.relative_path,
            ext: v.ext,
            mime_type: v.mime_type,
            duration: v.duration,
            bitrate: v.bitrate,
            codec: v.codec,
            channels: v.channels,
            sample_rate: v.sample_rate,
            track_num: v.track_num,
            disc_num: v.disc_num,
            embedded_cover_path: v.embedded_cover_path,
        }
    }
}

impl From<BookAudioFile> for BookAudioFileEntity {
    fn from(v: BookAudioFile) -> Self {
        Self {
            id: v.id,
            book_id: v.book_id,
            idx: v.idx,
            ino: v.ino,
            path: v.path,
            relative_path: v.relative_path,
            ext: v.ext,
            mime_type: v.mime_type,
            duration: v.duration,
            bitrate: v.bitrate,
            codec: v.codec,
            channels: v.channels,
            sample_rate: v.sample_rate,
            track_num: v.track_num,
            disc_num: v.disc_num,
            embedded_cover_path: v.embedded_cover_path,
        }
    }
}

pub struct DieselBookAudioFileRepository {
    database: Database,
}

impl DieselBookAudioFileRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl BookAudioFileRepository for DieselBookAudioFileRepository {
    type Error = PersistenceError;

    fn replace_for_book(
        &self,
        lookup_book_id: &str,
        files: Vec<BookAudioFile>,
    ) -> Result<Vec<BookAudioFile>, Self::Error> {
        use self::audiobookshelf_book_audio_files::dsl::*;
        let mut conn = self.database.connection()?;
        diesel::delete(audiobookshelf_book_audio_files.filter(book_id.eq(lookup_book_id)))
            .execute(&mut conn)
            .map_err(PersistenceError::from)?;
        let entities: Vec<BookAudioFileEntity> = files.into_iter().map(Into::into).collect();
        for entity in &entities {
            diesel::insert_into(audiobookshelf_book_audio_files)
                .values(entity.clone())
                .execute(&mut conn)
                .map_err(PersistenceError::from)?;
        }
        Ok(entities.into_iter().map(Into::into).collect())
    }

    fn list_for_book(&self, lookup_book_id: &str) -> Result<Vec<BookAudioFile>, Self::Error> {
        use self::audiobookshelf_book_audio_files::dsl::*;
        let mut conn = self.database.connection()?;
        audiobookshelf_book_audio_files
            .filter(book_id.eq(lookup_book_id))
            .order(idx.asc())
            .load::<BookAudioFileEntity>(&mut conn)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
