use diesel::{AsChangeset, BoolExpressionMethods};
use diesel::{ExpressionMethods, Identifiable, OptionalExtension, QueryDsl, Queryable, QueryableByName, RunQueryDsl, Selectable};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Timestamp};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::*;
use crate::models::podcast_episode::PodcastEpisode;
use crate::service::podcast_chapter::Chapter;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::error::ErrorSeverity::Critical;
use diesel::Insertable;
use crate::controllers::podcast_episode_controller::PodcastChapterDto;

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    Insertable,
    PartialEq,
    Clone,
    ToSchema,
    Serialize,
    Deserialize,
    Default,
    AsChangeset
)]
pub struct PodcastEpisodeChapter {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Integer)]
    pub episode_id: i32,
    #[diesel(sql_type = Text)]
    pub title: String,
    #[diesel(sql_type = Integer)]
    pub start_time: i32,
    #[diesel(sql_type = Integer)]
    pub end_time: i32,
    #[diesel(sql_type = Nullable<Text>)]
    pub href: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub image: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub created_at: chrono::NaiveDateTime,
    #[diesel(sql_type = Timestamp)]
    pub updated_at: chrono::NaiveDateTime,
}


impl PodcastEpisodeChapter {
    pub fn save_chapter(
        chapter_to_save: &Chapter,
        podcast_episode: &PodcastEpisode,
    ) -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episode_chapters::dsl::*;
        let mut new_chapter = PodcastEpisodeChapter {
            id: uuid::Uuid::new_v4().to_string(),
            episode_id: podcast_episode.id,
            title: chapter_to_save.title.clone().unwrap_or_default(),
            start_time: chapter_to_save.start.num_seconds() as i32,
            end_time: chapter_to_save.end.unwrap_or_default().num_seconds() as i32,
            href: chapter_to_save.link.clone().map(|link| link.url.to_string()),
            image: chapter_to_save.image.clone().map(|img| img.to_string()),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        let opt_chapter = podcast_episode_chapters
            .filter(episode_id.eq(podcast_episode.id)
            .and(start_time.eq(chapter_to_save.start.num_seconds() as i32)))
            .first::<PodcastEpisodeChapter>(&mut get_connection())
            .optional()
            .map_err(|e| map_db_error(e, Critical))?;

        match opt_chapter {
            Some(existing_chapter) => {
                new_chapter.created_at = existing_chapter.created_at;
                new_chapter.id = existing_chapter.id.clone();
                diesel::update(podcast_episode_chapters.find(existing_chapter.id))
                    .set(&new_chapter)
                    .execute(&mut get_connection())
                    .map_err(|e| map_db_error(e, Critical))?;
            }
            None => {
                diesel::insert_into(podcast_episode_chapters)
                    .values(&new_chapter)
                    .execute(&mut get_connection())
                    .map_err(|e| map_db_error(e, Critical))?;
            }
        }

        Ok(())
    }

    pub fn get_chapters_by_episode_id(episode_id_to_search: i32) -> Result<Vec<PodcastEpisodeChapter>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_episode_chapters::dsl::*;

        let chapters = podcast_episode_chapters
            .filter(episode_id.eq(episode_id_to_search))
            .load::<PodcastEpisodeChapter>(&mut get_connection())
            .map_err(|e| map_db_error(e, Critical))?;

        Ok(chapters)
    }
}

impl Into<PodcastChapterDto> for PodcastEpisodeChapter {
    fn into(self) -> PodcastChapterDto {
        PodcastChapterDto {
            id: self.id,
            title: self.title,
            start_time: self.start_time,
            end_time: self.end_time,
        }
    }
}
