use crate::files::domain::{EpisodeFile, EpisodeFileRepository};
use chrono::NaiveDateTime;
use common_infrastructure::db::{Database, PersistenceError};
use common_infrastructure::error::CustomError;
use diesel::prelude::*;

diesel::table! {
    episode_files (episode_id) {
        episode_id -> Text,
        download_time -> Nullable<Timestamp>,
        file_episode_path -> Nullable<Text>,
        file_image_path -> Nullable<Text>,
        download_location -> Nullable<Text>,
        deleted -> Bool,
    }
}

#[derive(Queryable, Insertable, Debug, Clone, PartialEq, Eq)]
#[diesel(table_name = episode_files)]
struct FileRow {
    episode_id: String,
    download_time: Option<NaiveDateTime>,
    file_episode_path: Option<String>,
    file_image_path: Option<String>,
    download_location: Option<String>,
    deleted: bool,
}

impl From<FileRow> for EpisodeFile {
    fn from(row: FileRow) -> Self {
        Self {
            episode_id: row.episode_id,
            download_time: row.download_time,
            file_episode_path: row.file_episode_path,
            file_image_path: row.file_image_path,
            download_location: row.download_location,
            deleted: row.deleted,
        }
    }
}

impl From<&EpisodeFile> for FileRow {
    fn from(file: &EpisodeFile) -> Self {
        Self {
            episode_id: file.episode_id.clone(),
            download_time: file.download_time,
            file_episode_path: file.file_episode_path.clone(),
            file_image_path: file.file_image_path.clone(),
            download_location: file.download_location.clone(),
            deleted: file.deleted,
        }
    }
}

pub struct DieselEpisodeFileRepository {
    database: Database,
}

impl DieselEpisodeFileRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl EpisodeFileRepository for DieselEpisodeFileRepository {
    fn find(&self, episode_id: &str) -> Result<Option<EpisodeFile>, CustomError> {
        let row = episode_files::table
            .filter(episode_files::episode_id.eq(episode_id))
            .first::<FileRow>(&mut self.database.connection()?)
            .optional()
            .map_err(PersistenceError::from)?;
        Ok(row.map(Into::into))
    }

    fn modify(
        &self,
        episode_id: &str,
        change: &mut dyn FnMut(&mut EpisodeFile),
    ) -> Result<(), CustomError> {
        let mut conn = self.database.connection()?;
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let mut file = episode_files::table
                .filter(episode_files::episode_id.eq(episode_id))
                .first::<FileRow>(conn)
                .optional()?
                .map_or_else(|| EpisodeFile::none(episode_id), Into::into);
            change(&mut file);
            diesel::delete(episode_files::table.filter(episode_files::episode_id.eq(episode_id)))
                .execute(conn)?;
            if !file.is_empty() {
                diesel::insert_into(episode_files::table)
                    .values(FileRow::from(&file))
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(PersistenceError::from)?;
        Ok(())
    }
}
