use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapter, PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};

diesel::table! {
    podcast_episode_chapters (id) {
        id -> Text,
        episode_id -> Integer,
        title -> Text,
        start_time -> Integer,
        end_time -> Integer,
        href -> Nullable<Text>,
        image -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = podcast_episode_chapters)]
struct PodcastEpisodeChapterEntity {
    id: String,
    episode_id: i32,
    title: String,
    start_time: i32,
    end_time: i32,
    href: Option<String>,
    image: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = podcast_episode_chapters)]
struct PodcastEpisodeChapterInsertEntity {
    id: String,
    episode_id: i32,
    title: String,
    start_time: i32,
    end_time: i32,
    href: Option<String>,
    image: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<PodcastEpisodeChapterEntity> for PodcastEpisodeChapter {
    fn from(value: PodcastEpisodeChapterEntity) -> Self {
        Self {
            id: value.id,
            episode_id: value.episode_id,
            title: value.title,
            start_time: value.start_time,
            end_time: value.end_time,
            href: value.href,
            image: value.image,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

pub struct DieselPodcastEpisodeChapterRepository {
    database: Database,
}

impl DieselPodcastEpisodeChapterRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PodcastEpisodeChapterRepository for DieselPodcastEpisodeChapterRepository {
    type Error = PersistenceError;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        use self::podcast_episode_chapters::dsl as pec_dsl;
        use self::podcast_episode_chapters::table as pec_table;

        let now = chrono::Utc::now().naive_utc();
        let existing = pec_table
            .filter(pec_dsl::episode_id.eq(chapter.episode_id))
            .filter(pec_dsl::start_time.eq(chapter.start_time))
            .first::<PodcastEpisodeChapterEntity>(&mut self.database.connection()?)
            .optional()?;

        let chapter_to_store = match &existing {
            Some(existing) => PodcastEpisodeChapterInsertEntity {
                id: existing.id.clone(),
                episode_id: chapter.episode_id,
                title: chapter.title,
                start_time: chapter.start_time,
                end_time: chapter.end_time,
                href: chapter.href,
                image: chapter.image,
                created_at: existing.created_at,
                updated_at: now,
            },
            None => PodcastEpisodeChapterInsertEntity {
                id: uuid::Uuid::new_v4().to_string(),
                episode_id: chapter.episode_id,
                title: chapter.title,
                start_time: chapter.start_time,
                end_time: chapter.end_time,
                href: chapter.href,
                image: chapter.image,
                created_at: now,
                updated_at: now,
            },
        };

        match existing {
            Some(existing) => diesel::update(pec_table.find(existing.id))
                .set((
                    pec_dsl::episode_id.eq(chapter_to_store.episode_id),
                    pec_dsl::title.eq(chapter_to_store.title),
                    pec_dsl::start_time.eq(chapter_to_store.start_time),
                    pec_dsl::end_time.eq(chapter_to_store.end_time),
                    pec_dsl::href.eq(chapter_to_store.href),
                    pec_dsl::image.eq(chapter_to_store.image),
                    pec_dsl::updated_at.eq(chapter_to_store.updated_at),
                ))
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
            None => diesel::insert_into(pec_table)
                .values(chapter_to_store)
                .execute(&mut self.database.connection()?)
                .map(|_| ())
                .map_err(Into::into),
        }
    }

    fn get_by_episode_id(
        &self,
        episode_id_to_search: i32,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error> {
        use self::podcast_episode_chapters::dsl as pec_dsl;
        use self::podcast_episode_chapters::table as pec_table;

        pec_table
            .filter(pec_dsl::episode_id.eq(episode_id_to_search))
            .load::<PodcastEpisodeChapterEntity>(&mut self.database.connection()?)
            .map(|chapters| chapters.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}
