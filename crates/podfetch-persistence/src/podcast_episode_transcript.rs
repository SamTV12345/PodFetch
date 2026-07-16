use crate::db::{Database, DBType, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable, Selectable};
use diesel::sql_types::{BigInt, Double, Integer, Nullable, Text};
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, QueryableByName, RunQueryDsl,
};
use podfetch_domain::podcast_episode_transcript::{
    PodcastEpisodeTranscript, PodcastEpisodeTranscriptRepository, TranscriptSegment,
    TranscriptSearchHit, TranscriptSource, TranscriptStatus, TranscriptionJob,
    TranscriptionJobRepository, TranscriptionJobStatus, UpsertTranscript,
};
use std::ops::DerefMut;
use uuid::Uuid;

// Note: `text_search` (postgres tsvector / sqlite FTS mirror) is intentionally
// NOT declared here — it's a generated/trigger-maintained column Diesel never
// needs to read or write directly.
diesel::table! {
    podcast_episode_transcripts (id) {
        id -> Text,
        episode_id -> Text,
        source -> Text,
        original_url -> Nullable<Text>,
        file_path -> Nullable<Text>,
        mime_type -> Text,
        language -> Nullable<Text>,
        is_preferred -> Bool,
        status -> Text,
        error -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    podcast_episode_transcript_segments (id) {
        id -> Text,
        transcript_id -> Text,
        idx -> Integer,
        start_ms -> Nullable<Integer>,
        end_ms -> Nullable<Integer>,
        speaker -> Nullable<Text>,
        text -> Text,
    }
}

diesel::table! {
    transcription_jobs (id) {
        id -> Text,
        episode_id -> Text,
        status -> Text,
        attempts -> Integer,
        error -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

// ── Entities ─────────────────────────────────────────────────────────────

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = podcast_episode_transcripts)]
struct TranscriptEntity {
    id: String,
    episode_id: String,
    source: String,
    original_url: Option<String>,
    file_path: Option<String>,
    mime_type: String,
    language: Option<String>,
    is_preferred: bool,
    status: String,
    error: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable, Clone)]
#[diesel(table_name = podcast_episode_transcripts)]
struct TranscriptInsertEntity {
    id: String,
    episode_id: String,
    source: String,
    original_url: Option<String>,
    file_path: Option<String>,
    mime_type: String,
    language: Option<String>,
    is_preferred: bool,
    status: String,
    error: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<TranscriptEntity> for PodcastEpisodeTranscript {
    fn from(value: TranscriptEntity) -> Self {
        Self {
            id: Uuid::parse_str(&value.id).expect("valid uuid in db"),
            episode_id: Uuid::parse_str(&value.episode_id).expect("valid uuid in db"),
            source: TranscriptSource::from_str(&value.source).expect("valid source in db"),
            original_url: value.original_url,
            file_path: value.file_path,
            mime_type: value.mime_type,
            language: value.language,
            is_preferred: value.is_preferred,
            status: TranscriptStatus::from_str(&value.status).expect("valid status in db"),
            error: value.error,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = podcast_episode_transcript_segments)]
struct SegmentEntity {
    id: String,
    transcript_id: String,
    idx: i32,
    start_ms: Option<i32>,
    end_ms: Option<i32>,
    speaker: Option<String>,
    text: String,
}

impl From<SegmentEntity> for TranscriptSegment {
    fn from(value: SegmentEntity) -> Self {
        Self {
            idx: value.idx,
            start_ms: value.start_ms,
            end_ms: value.end_ms,
            speaker: value.speaker,
            text: value.text,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = transcription_jobs)]
struct TranscriptionJobEntity {
    id: String,
    episode_id: String,
    status: String,
    attempts: i32,
    error: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<TranscriptionJobEntity> for TranscriptionJob {
    fn from(value: TranscriptionJobEntity) -> Self {
        Self {
            id: Uuid::parse_str(&value.id).expect("valid uuid in db"),
            episode_id: Uuid::parse_str(&value.episode_id).expect("valid uuid in db"),
            status: TranscriptionJobStatus::from_str(&value.status).expect("valid status in db"),
            attempts: value.attempts,
            error: value.error,
        }
    }
}

// ── Full-text search ─────────────────────────────────────────────────────

/// Row shape shared by both the SQLite and Postgres full-text search
/// queries. `rank` is read as `Double` (f64) for both backends — the
/// Postgres query casts `ts_rank(...)` to `double precision` so a single
/// row struct works for either connection type — and narrowed to `f32` when
/// mapped into `TranscriptSearchHit`.
///
/// Both backends' SQL normalizes `rank` to the same "higher is better"
/// convention: SQLite's `bm25()` is natively lower/negative-is-better, so the
/// query negates it (`-bm25(...)`); Postgres's `ts_rank()` is already
/// higher-is-better and is used unmodified. Both queries then `ORDER BY rank
/// DESC`, so the most relevant hit is always first regardless of backend.
#[derive(QueryableByName)]
struct SearchHitRow {
    #[diesel(sql_type = Text)]
    episode_id: String,
    #[diesel(sql_type = Text)]
    transcript_id: String,
    #[diesel(sql_type = Nullable<Integer>)]
    start_ms: Option<i32>,
    #[diesel(sql_type = Text)]
    snippet: String,
    #[diesel(sql_type = Double)]
    rank: f64,
}

impl From<SearchHitRow> for TranscriptSearchHit {
    fn from(value: SearchHitRow) -> Self {
        Self {
            episode_id: Uuid::parse_str(&value.episode_id).expect("valid uuid in db"),
            transcript_id: Uuid::parse_str(&value.transcript_id).expect("valid uuid in db"),
            start_ms: value.start_ms,
            snippet: value.snippet,
            rank: value.rank as f32,
        }
    }
}

/// Sanitizes free-form user input for SQLite FTS5 `MATCH` queries.
///
/// User input must never be passed to `MATCH` verbatim: FTS5 query syntax
/// includes operators (`AND`, `OR`, `NOT`, `NEAR`, column filters, `"..."`
/// phrase quoting, etc.) and unbalanced quotes raise a syntax error. This
/// strips `"` characters and turns each remaining word into a quoted prefix
/// term, e.g. `quick fox` -> `"quick"* "fox"*`, so every word is matched
/// literally as a prefix rather than being interpreted as FTS5 syntax.
/// Postgres doesn't need this — `websearch_to_tsquery` already hardens
/// arbitrary input.
#[cfg(feature = "sqlite")]
fn sanitize_sqlite_match_query(query: &str) -> String {
    query
        .replace('"', "")
        .split_whitespace()
        .map(|word| format!("\"{word}\"*"))
        .collect::<Vec<_>>()
        .join(" ")
}

// ── PodcastEpisodeTranscript repository ─────────────────────────────────

pub struct DieselPodcastEpisodeTranscriptRepository {
    database: Database,
}

impl DieselPodcastEpisodeTranscriptRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl PodcastEpisodeTranscriptRepository for DieselPodcastEpisodeTranscriptRepository {
    type Error = PersistenceError;

    fn upsert(&self, transcript: UpsertTranscript) -> Result<Uuid, Self::Error> {
        use self::podcast_episode_transcripts::dsl as pet_dsl;
        use self::podcast_episode_transcripts::table as pet_table;

        let mut conn = self.database.connection()?;
        let now = chrono::Utc::now().naive_utc();
        let episode_id = transcript.episode_id.to_string();
        let source_str = transcript.source.as_str().to_string();

        // Upsert identity is (episode_id, original_url). For rows without an
        // original_url (i.e. the single generated transcript per episode) the
        // source additionally scopes the match, so a generated transcript
        // never collides with a future feed transcript that also lacks a URL.
        //
        // The SELECT-then-INSERT below is not atomic on its own: two
        // concurrent upserts for the same NULL-url identity could both miss
        // the SELECT and both attempt an INSERT (NULL is never equal to NULL,
        // so the `uq_transcripts_episode_url` unique index can't catch this
        // case). Wrapping in a transaction doesn't remove that race by
        // itself, but the partial unique index
        // `uq_transcripts_episode_generated` (episode_id WHERE source =
        // 'generated') is the real backstop: it guarantees the DB rejects a
        // second generated-source row for the same episode even under
        // concurrent access, and doing the work inside a transaction keeps
        // the read/write pair consistent and ensures a unique-violation
        // rolls back cleanly rather than leaving a partial update.
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let mut query = pet_table
                .filter(pet_dsl::episode_id.eq(episode_id.clone()))
                .into_boxed();
            query = match &transcript.original_url {
                Some(url) => query.filter(pet_dsl::original_url.eq(url.clone())),
                None => query
                    .filter(pet_dsl::original_url.is_null())
                    .filter(pet_dsl::source.eq(source_str.clone())),
            };
            let existing = query.first::<TranscriptEntity>(conn).optional()?;

            match existing {
                Some(existing) => {
                    let id = existing.id.clone();
                    diesel::update(pet_table.find(id.clone()))
                        .set((
                            pet_dsl::mime_type.eq(transcript.mime_type),
                            pet_dsl::language.eq(transcript.language),
                            pet_dsl::updated_at.eq(now),
                        ))
                        .execute(conn)?;
                    Ok(Uuid::parse_str(&id).expect("valid uuid in db"))
                }
                None => {
                    let id = Uuid::new_v4();
                    let entity = TranscriptInsertEntity {
                        id: id.to_string(),
                        episode_id,
                        source: source_str,
                        original_url: transcript.original_url,
                        file_path: None,
                        mime_type: transcript.mime_type,
                        language: transcript.language,
                        is_preferred: false,
                        status: TranscriptStatus::Pending.as_str().to_string(),
                        error: None,
                        created_at: now,
                        updated_at: now,
                    };
                    diesel::insert_into(pet_table).values(entity).execute(conn)?;
                    Ok(id)
                }
            }
        })
        .map_err(Into::into)
    }

    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
        use self::podcast_episode_transcripts::dsl as pet_dsl;
        use self::podcast_episode_transcripts::table as pet_table;

        pet_table
            .filter(pet_dsl::episode_id.eq(episode_id.to_string()))
            .load::<TranscriptEntity>(&mut self.database.connection()?)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error> {
        use self::podcast_episode_transcripts::table as pet_table;

        pet_table
            .find(id.to_string())
            .first::<TranscriptEntity>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(Into::into))
            .map_err(Into::into)
    }

    fn set_file_path(&self, id: Uuid, file_path: &str) -> Result<(), Self::Error> {
        use self::podcast_episode_transcripts::dsl as pet_dsl;
        use self::podcast_episode_transcripts::table as pet_table;

        let now = chrono::Utc::now().naive_utc();
        diesel::update(pet_table.find(id.to_string()))
            .set((
                pet_dsl::file_path.eq(file_path),
                pet_dsl::updated_at.eq(now),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn set_status(&self, id: Uuid, status: TranscriptStatus, error: Option<&str>) -> Result<(), Self::Error> {
        use self::podcast_episode_transcripts::dsl as pet_dsl;
        use self::podcast_episode_transcripts::table as pet_table;

        let now = chrono::Utc::now().naive_utc();
        diesel::update(pet_table.find(id.to_string()))
            .set((
                pet_dsl::status.eq(status.as_str()),
                pet_dsl::error.eq(error),
                pet_dsl::updated_at.eq(now),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn set_preferred(&self, episode_id: Uuid, preferred_id: Option<Uuid>) -> Result<(), Self::Error> {
        use self::podcast_episode_transcripts::dsl as pet_dsl;
        use self::podcast_episode_transcripts::table as pet_table;

        let mut conn = self.database.connection()?;
        let now = chrono::Utc::now().naive_utc();
        let episode_id_str = episode_id.to_string();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::update(pet_table.filter(pet_dsl::episode_id.eq(episode_id_str.clone())))
                .set((pet_dsl::is_preferred.eq(false), pet_dsl::updated_at.eq(now)))
                .execute(conn)?;

            if let Some(preferred_id) = preferred_id {
                diesel::update(pet_table.find(preferred_id.to_string()))
                    .set((pet_dsl::is_preferred.eq(true), pet_dsl::updated_at.eq(now)))
                    .execute(conn)?;
            }

            Ok(())
        })
        .map_err(Into::into)
    }

    fn replace_segments(&self, transcript_id: Uuid, segments: &[TranscriptSegment]) -> Result<(), Self::Error> {
        use self::podcast_episode_transcript_segments::dsl as seg_dsl;
        use self::podcast_episode_transcript_segments::table as seg_table;

        let mut conn = self.database.connection()?;
        let transcript_id_str = transcript_id.to_string();
        let entities: Vec<SegmentEntity> = segments
            .iter()
            .map(|segment| SegmentEntity {
                id: Uuid::new_v4().to_string(),
                transcript_id: transcript_id_str.clone(),
                idx: segment.idx,
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
                speaker: segment.speaker.clone(),
                text: segment.text.clone(),
            })
            .collect();

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(seg_table.filter(seg_dsl::transcript_id.eq(transcript_id_str.clone())))
                .execute(conn)?;
            for entity in &entities {
                diesel::insert_into(seg_table).values(entity).execute(conn)?;
            }
            Ok(())
        })
        .map_err(Into::into)
    }

    fn get_segments(&self, transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error> {
        use self::podcast_episode_transcript_segments::dsl as seg_dsl;
        use self::podcast_episode_transcript_segments::table as seg_table;

        seg_table
            .filter(seg_dsl::transcript_id.eq(transcript_id.to_string()))
            .order(seg_dsl::idx.asc())
            .load::<SegmentEntity>(&mut self.database.connection()?)
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn search(
        &self,
        query: &str,
        podcast_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<TranscriptSearchHit>, Self::Error> {
        let mut conn = self.database.connection()?;
        let podcast_id_str = podcast_id.map(|id| id.to_string());
        // `page` is a zero-based page index; `page_size` rows per page.
        let offset = page.saturating_mul(page_size);

        let rows: Vec<SearchHitRow> = match conn.deref_mut() {
            #[cfg(feature = "sqlite")]
            DBType::Sqlite(conn) => {
                let match_query = sanitize_sqlite_match_query(query);
                // An empty (or whitespace-only) query sanitizes to "", and
                // `MATCH ''` raises an FTS5 syntax error rather than matching
                // nothing. Short-circuit before running any SQL so an
                // empty/blank query simply yields no results.
                if match_query.is_empty() {
                    return Ok(Vec::new());
                }
                // Positional `?` placeholders only (no `?N` back-references):
                // diesel binds values in call order, so the podcast_id filter
                // is bound twice rather than reusing a single numbered param.
                diesel::sql_query(
                    "SELECT t.episode_id AS episode_id, s.transcript_id AS transcript_id, \
                     s.start_ms AS start_ms, \
                     highlight(transcript_segments_fts, 0, '<b>', '</b>') AS snippet, \
                     -bm25(transcript_segments_fts) AS rank \
                     FROM transcript_segments_fts \
                     JOIN podcast_episode_transcript_segments s \
                       ON s.rowid = transcript_segments_fts.rowid \
                     JOIN podcast_episode_transcripts t ON t.id = s.transcript_id \
                     JOIN podcast_episodes e ON e.id = t.episode_id \
                     WHERE transcript_segments_fts MATCH ? \
                       AND (? IS NULL OR e.podcast_id = ?) \
                     ORDER BY rank DESC LIMIT ? OFFSET ?",
                )
                .bind::<Text, _>(match_query)
                .bind::<Nullable<Text>, _>(podcast_id_str.clone())
                .bind::<Nullable<Text>, _>(podcast_id_str)
                .bind::<BigInt, _>(page_size)
                .bind::<BigInt, _>(offset)
                .load::<SearchHitRow>(conn)?
            }
            #[cfg(feature = "postgresql")]
            DBType::Postgresql(conn) => {
                // Mirrors the SQLite short-circuit above: an empty/blank
                // query has no meaningful `websearch_to_tsquery` match, so
                // skip the query entirely rather than rely on the DB to
                // handle it uniformly.
                if query.trim().is_empty() {
                    return Ok(Vec::new());
                }
                // `$1` is referenced three times in the query text but bound
                // only once — Postgres resolves repeated `$N` markers to the
                // same bound value.
                diesel::sql_query(
                    "SELECT t.episode_id AS episode_id, s.transcript_id AS transcript_id, \
                     s.start_ms AS start_ms, \
                     ts_headline('simple', s.text, websearch_to_tsquery('simple', $1), \
                                 'StartSel=<b>,StopSel=</b>') AS snippet, \
                     ts_rank(s.text_search, websearch_to_tsquery('simple', $1))::double precision \
                       AS rank \
                     FROM podcast_episode_transcript_segments s \
                     JOIN podcast_episode_transcripts t ON t.id = s.transcript_id \
                     JOIN podcast_episodes e ON e.id = t.episode_id \
                     WHERE s.text_search @@ websearch_to_tsquery('simple', $1) \
                       AND ($2::text IS NULL OR e.podcast_id = $2) \
                     ORDER BY rank DESC LIMIT $3 OFFSET $4",
                )
                .bind::<Text, _>(query)
                .bind::<Nullable<Text>, _>(podcast_id_str)
                .bind::<BigInt, _>(page_size)
                .bind::<BigInt, _>(offset)
                .load::<SearchHitRow>(conn)?
            }
        };

        Ok(rows.into_iter().map(Into::into).collect())
    }
}

// ── TranscriptionJob repository ─────────────────────────────────────────

pub struct DieselTranscriptionJobRepository {
    database: Database,
}

impl DieselTranscriptionJobRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl TranscriptionJobRepository for DieselTranscriptionJobRepository {
    type Error = PersistenceError;

    fn enqueue(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        let mut conn = self.database.connection()?;
        let episode_id_str = episode_id.to_string();

        let existing = tj_table
            .filter(tj_dsl::episode_id.eq(episode_id_str.clone()))
            .first::<TranscriptionJobEntity>(&mut conn)
            .optional()?;
        if existing.is_some() {
            return Ok(None);
        }

        let now = chrono::Utc::now().naive_utc();
        let id = Uuid::new_v4();
        let entity = TranscriptionJobEntity {
            id: id.to_string(),
            episode_id: episode_id_str,
            status: TranscriptionJobStatus::Pending.as_str().to_string(),
            attempts: 0,
            error: None,
            created_at: now,
            updated_at: now,
        };
        diesel::insert_into(tj_table)
            .values(entity.clone())
            .execute(&mut conn)?;

        Ok(Some(entity.into()))
    }

    fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        tj_table
            .filter(tj_dsl::status.eq(TranscriptionJobStatus::Pending.as_str()))
            .order(tj_dsl::created_at.asc())
            .first::<TranscriptionJobEntity>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(Into::into))
            .map_err(Into::into)
    }

    fn set_status(&self, id: Uuid, status: TranscriptionJobStatus, error: Option<&str>) -> Result<(), Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        let now = chrono::Utc::now().naive_utc();
        diesel::update(tj_table.find(id.to_string()))
            .set((
                tj_dsl::status.eq(status.as_str()),
                tj_dsl::error.eq(error),
                tj_dsl::updated_at.eq(now),
            ))
            .execute(&mut self.database.connection()?)
            .map(|_| ())
            .map_err(Into::into)
    }

    fn increment_attempts(&self, id: Uuid) -> Result<i32, Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        let mut conn = self.database.connection()?;
        let now = chrono::Utc::now().naive_utc();
        let id_str = id.to_string();

        diesel::update(tj_table.find(id_str.clone()))
            .set((
                tj_dsl::attempts.eq(tj_dsl::attempts + 1),
                tj_dsl::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        tj_table
            .find(id_str)
            .select(tj_dsl::attempts)
            .first::<i32>(&mut conn)
            .map_err(Into::into)
    }

    fn reset_running_to_pending(&self) -> Result<usize, Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        let now = chrono::Utc::now().naive_utc();
        diesel::update(
            tj_table.filter(tj_dsl::status.eq(TranscriptionJobStatus::Running.as_str())),
        )
        .set((
            tj_dsl::status.eq(TranscriptionJobStatus::Pending.as_str()),
            tj_dsl::updated_at.eq(now),
        ))
        .execute(&mut self.database.connection()?)
        .map_err(Into::into)
    }

    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        use self::transcription_jobs::dsl as tj_dsl;
        use self::transcription_jobs::table as tj_table;

        tj_table
            .filter(tj_dsl::episode_id.eq(episode_id.to_string()))
            .first::<TranscriptionJobEntity>(&mut self.database.connection()?)
            .optional()
            .map(|row| row.map(Into::into))
            .map_err(Into::into)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::db::{database, test_db::setup};

    // Shares the crate-wide DB test lock (which also runs migrations) so these
    // tests serialize against every other DB-touching test module. The lock
    // lives in `crate::db::test_db` — never reintroduce a module-local one, or
    // parallel `run_migrations()` calls race on `__diesel_schema_migrations`.

    mod seed_schema {
        diesel::table! {
            podcasts (id) {
                id -> Text,
                name -> Text,
                directory_id -> Text,
                rssfeed -> Text,
                image_url -> Text,
                active -> Bool,
                original_image_url -> Text,
                directory_name -> Text,
            }
        }

        diesel::table! {
            podcast_episodes (id) {
                id -> Text,
                podcast_id -> Text,
                episode_id -> Text,
                name -> Text,
                url -> Text,
                date_of_recording -> Text,
                image_url -> Text,
                total_time -> Integer,
                description -> Text,
                guid -> Text,
                deleted -> Bool,
                episode_numbering_processed -> Bool,
            }
        }
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::podcasts)]
    struct SeedPodcast {
        id: String,
        name: String,
        directory_id: String,
        rssfeed: String,
        image_url: String,
        active: bool,
        original_image_url: String,
        directory_name: String,
    }

    #[derive(diesel::Insertable)]
    #[diesel(table_name = seed_schema::podcast_episodes)]
    struct SeedEpisode {
        id: String,
        podcast_id: String,
        episode_id: String,
        name: String,
        url: String,
        date_of_recording: String,
        image_url: String,
        total_time: i32,
        description: String,
        guid: String,
        deleted: bool,
        episode_numbering_processed: bool,
    }

    fn seed_podcast() -> String {
        use seed_schema::podcasts;
        let id = uuid::Uuid::new_v4().to_string();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(podcasts::table)
            .values(SeedPodcast {
                id: id.clone(),
                name: format!("Test Podcast {id}"),
                directory_id: uuid::Uuid::new_v4().to_string(),
                rssfeed: format!("https://example.com/feed/{id}.xml"),
                image_url: "https://example.com/img.png".to_string(),
                active: true,
                original_image_url: "https://example.com/img.png".to_string(),
                directory_name: format!("podcast-{id}"),
            })
            .execute(&mut conn)
            .expect("seed podcast");
        id
    }

    fn seed_episode(podcast_id: &str) -> Uuid {
        use seed_schema::podcast_episodes;
        let id = Uuid::new_v4();
        let mut conn = database().connection().expect("db connection");
        diesel::insert_into(podcast_episodes::table)
            .values(SeedEpisode {
                id: id.to_string(),
                podcast_id: podcast_id.to_string(),
                episode_id: uuid::Uuid::new_v4().to_string(),
                name: "Test Episode".to_string(),
                url: format!("https://example.com/ep/{id}.mp3"),
                date_of_recording: "2024-01-01".to_string(),
                image_url: "https://example.com/ep.png".to_string(),
                total_time: 3600,
                description: "Test description".to_string(),
                guid: uuid::Uuid::new_v4().to_string(),
                deleted: false,
                episode_numbering_processed: false,
            })
            .execute(&mut conn)
            .expect("seed episode");
        id
    }

    fn upsert_transcript(episode_id: Uuid, original_url: Option<&str>) -> UpsertTranscript {
        UpsertTranscript {
            episode_id,
            source: if original_url.is_some() {
                TranscriptSource::Feed
            } else {
                TranscriptSource::Generated
            },
            original_url: original_url.map(|s| s.to_string()),
            mime_type: "text/vtt".to_string(),
            language: Some("en".to_string()),
        }
    }

    /// Tests in this module share one process-wide sqlite DB (see `setup()`)
    /// and nothing truncates `transcription_jobs` between tests. `next_pending`
    /// and `reset_running_to_pending` operate over the *whole* table, so a
    /// pending/running row left behind by an earlier test would make a later
    /// test's assertions depend on run order. Clear the table up front (while
    /// still holding the shared lock) so each job test starts from empty.
    fn clear_transcription_jobs() {
        use self::transcription_jobs::table as tj_table;
        let mut conn = database().connection().expect("db connection");
        diesel::delete(tj_table).execute(&mut conn).expect("clear transcription_jobs");
    }

    fn make_segment(idx: i32, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            idx,
            start_ms: Some(idx * 1000),
            end_ms: Some(idx * 1000 + 500),
            speaker: None,
            text: text.to_string(),
        }
    }

    // ── PodcastEpisodeTranscript tests ──────────────────────────────────

    #[test]
    fn upsert_is_idempotent_on_episode_and_url() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let id1 = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/t.vtt")))
            .expect("first upsert");
        let id2 = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/t.vtt")))
            .expect("second upsert");

        assert_eq!(id1, id2, "upsert should return the same id for the same key");

        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        assert_eq!(rows.len(), 1, "expected exactly one transcript row");
        assert_eq!(rows[0].id, id1);
    }

    #[test]
    fn upsert_treats_generated_null_url_as_single_identity() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let id1 = repo
            .upsert(upsert_transcript(episode_id, None))
            .expect("first upsert");
        let id2 = repo
            .upsert(upsert_transcript(episode_id, None))
            .expect("second upsert");

        assert_eq!(id1, id2);

        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        assert_eq!(rows.len(), 1, "expected a single generated transcript row");
        assert_eq!(rows[0].source, TranscriptSource::Generated);
        assert_eq!(rows[0].original_url, None);
    }

    #[test]
    fn db_rejects_second_generated_transcript_row_inserted_directly() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        // Repo-level upsert: a single generated row is created and repeated
        // upserts stay idempotent against it (existing behavior).
        let id1 = repo
            .upsert(upsert_transcript(episode_id, None))
            .expect("first upsert");
        let id2 = repo
            .upsert(upsert_transcript(episode_id, None))
            .expect("second upsert");
        assert_eq!(id1, id2, "upsert must stay idempotent for generated transcripts");

        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        assert_eq!(rows.len(), 1, "expected exactly one generated transcript row");

        // Bypass the repo's SELECT-then-INSERT logic entirely and try to
        // insert a second generated-source row directly. This is the
        // scenario the app-level check can't prevent under a race; the
        // partial unique index `uq_transcripts_episode_generated` must be
        // the one to reject it.
        use self::podcast_episode_transcripts::table as pet_table;
        let mut conn = database().connection().expect("db connection");
        let now = chrono::Utc::now().naive_utc();
        let duplicate = TranscriptInsertEntity {
            id: Uuid::new_v4().to_string(),
            episode_id: episode_id.to_string(),
            source: TranscriptSource::Generated.as_str().to_string(),
            original_url: None,
            file_path: None,
            mime_type: "text/vtt".to_string(),
            language: Some("en".to_string()),
            is_preferred: false,
            status: TranscriptStatus::Pending.as_str().to_string(),
            error: None,
            created_at: now,
            updated_at: now,
        };
        let result = diesel::insert_into(pet_table)
            .values(duplicate)
            .execute(&mut conn);

        assert!(
            result.is_err(),
            "DB must reject a second generated-source row for the same episode"
        );

        // Still exactly one row after the rejected insert attempt.
        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        assert_eq!(rows.len(), 1, "duplicate generated row must not have been inserted");
    }

    #[test]
    fn get_by_id_and_set_file_path_and_status() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let id = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/a.vtt")))
            .expect("upsert");

        let fetched = repo.get_by_id(id).expect("get_by_id").expect("row exists");
        assert_eq!(fetched.status, TranscriptStatus::Pending);
        assert_eq!(fetched.file_path, None);

        repo.set_file_path(id, "/data/transcripts/a.vtt").expect("set_file_path");
        repo.set_status(id, TranscriptStatus::Downloaded, None).expect("set_status");

        let updated = repo.get_by_id(id).expect("get_by_id").expect("row exists");
        assert_eq!(updated.file_path, Some("/data/transcripts/a.vtt".to_string()));
        assert_eq!(updated.status, TranscriptStatus::Downloaded);

        repo.set_status(id, TranscriptStatus::Failed, Some("boom")).expect("set_status failed");
        let failed = repo.get_by_id(id).expect("get_by_id").expect("row exists");
        assert_eq!(failed.status, TranscriptStatus::Failed);
        assert_eq!(failed.error, Some("boom".to_string()));
    }

    #[test]
    fn get_by_id_returns_none_for_unknown_id() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        assert!(repo.get_by_id(Uuid::new_v4()).expect("get_by_id").is_none());
    }

    #[test]
    fn set_preferred_clears_other_rows() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let id1 = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/1.vtt")))
            .expect("upsert 1");
        let id2 = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/2.vtt")))
            .expect("upsert 2");

        repo.set_preferred(episode_id, Some(id1)).expect("set preferred 1");
        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        let row1 = rows.iter().find(|r| r.id == id1).unwrap();
        let row2 = rows.iter().find(|r| r.id == id2).unwrap();
        assert!(row1.is_preferred);
        assert!(!row2.is_preferred);

        repo.set_preferred(episode_id, Some(id2)).expect("set preferred 2");
        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        let row1 = rows.iter().find(|r| r.id == id1).unwrap();
        let row2 = rows.iter().find(|r| r.id == id2).unwrap();
        assert!(!row1.is_preferred, "previous preferred should be cleared");
        assert!(row2.is_preferred);

        repo.set_preferred(episode_id, None).expect("clear preferred");
        let rows = repo.get_by_episode_id(episode_id).expect("get rows");
        assert!(rows.iter().all(|r| !r.is_preferred));
    }

    #[test]
    fn replace_segments_replaces_and_orders_by_idx() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);
        let transcript_id = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/seg.vtt")))
            .expect("upsert");

        let three = vec![
            make_segment(2, "third"),
            make_segment(0, "first"),
            make_segment(1, "second"),
        ];
        repo.replace_segments(transcript_id, &three).expect("replace 3");

        let segments = repo.get_segments(transcript_id).expect("get segments");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].idx, 0);
        assert_eq!(segments[1].idx, 1);
        assert_eq!(segments[2].idx, 2);

        let two = vec![make_segment(0, "only-first"), make_segment(1, "only-second")];
        repo.replace_segments(transcript_id, &two).expect("replace 2");

        let segments = repo.get_segments(transcript_id).expect("get segments after replace");
        assert_eq!(segments.len(), 2, "old segments must be gone");
        assert_eq!(segments[0].text, "only-first");
        assert_eq!(segments[1].text, "only-second");
    }

    #[test]
    fn search_finds_hit_with_highlighted_snippet_and_start_ms() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());

        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);
        let transcript_id = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/search-a.vtt")))
            .expect("upsert transcript a");
        repo.replace_segments(
            transcript_id,
            &[
                make_segment(0, "fox fox fox fox fox"),
                make_segment(1, "completely unrelated segment text"),
                // A long segment where "fox" appears only once, diluted by a
                // lot of unrelated filler text. bm25 penalizes document
                // length and rewards term frequency, so this segment should
                // rank strictly lower (less relevant) than segment 0, which
                // is short and dense with the query term.
                make_segment(
                    2,
                    "in a very long segment full of unrelated filler padding \
                     words that go on for quite a while just to dilute the \
                     term frequency and length normalization the fox appears \
                     only once here amidst all of this extra surrounding \
                     text added purely to reduce the relevance score",
                ),
            ],
        )
        .expect("replace segments a");

        // A second episode whose segments must never show up in a search
        // scoped to the first episode's podcast.
        let other_podcast_id = seed_podcast();
        let other_episode_id = seed_episode(&other_podcast_id);
        let other_transcript_id = repo
            .upsert(upsert_transcript(other_episode_id, Some("https://example.com/search-b.vtt")))
            .expect("upsert transcript b");
        repo.replace_segments(
            other_transcript_id,
            &[make_segment(0, "no matching keyword lives in this segment")],
        )
        .expect("replace segments b");

        let hits = repo.search("fox", None, 0, 20).expect("search");
        assert!(!hits.is_empty(), "expected at least one hit for 'fox'");
        let hit = hits
            .iter()
            .find(|h| h.episode_id == episode_id)
            .expect("hit for seeded episode");
        assert_eq!(hit.transcript_id, transcript_id);
        assert_eq!(hit.start_ms, Some(0));
        assert!(
            hit.snippet.contains("<b>fox</b>"),
            "snippet should highlight the match: {}",
            hit.snippet
        );
        assert!(
            hits.iter().all(|h| h.episode_id != other_episode_id),
            "the other episode has no 'fox' segment and must not match"
        );

        // Rank convention: higher is more relevant on both backends. Segment
        // 0 ("fox fox fox fox fox", start_ms 0) is short and dense with the
        // query term; segment 2 (start_ms 2000) is a long segment where
        // "fox" appears only once amid filler. The denser/shorter segment
        // must score as MORE relevant, i.e. have the numerically GREATER
        // rank.
        let dense_hit = hits
            .iter()
            .find(|h| h.episode_id == episode_id && h.start_ms == Some(0))
            .expect("hit for the dense 'fox' segment");
        let diluted_hit = hits
            .iter()
            .find(|h| h.episode_id == episode_id && h.start_ms == Some(2000))
            .expect("hit for the diluted 'fox' segment");
        assert!(
            dense_hit.rank > diluted_hit.rank,
            "the more relevant hit must have the greater rank (dense={}, diluted={})",
            dense_hit.rank,
            diluted_hit.rank
        );
    }

    #[test]
    fn search_with_empty_or_whitespace_query_returns_empty_without_error() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());

        // Seed a segment that a non-empty query WOULD match, so an empty
        // result below proves the empty-query short-circuit fired rather
        // than merely reflecting an empty table.
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);
        let transcript_id = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/search-empty.vtt")))
            .expect("upsert transcript");
        repo.replace_segments(
            transcript_id,
            &[make_segment(0, "this segment would match a real query")],
        )
        .expect("replace segments");

        let sanity = repo.search("segment", None, 0, 20).expect("sanity search");
        assert!(
            sanity.iter().any(|h| h.episode_id == episode_id),
            "sanity check: a real query must match the seeded segment"
        );

        let empty = repo.search("", None, 0, 20).expect("empty query search must not error");
        assert!(empty.is_empty(), "empty query must return no hits");

        let whitespace = repo
            .search("   ", None, 0, 20)
            .expect("whitespace-only query search must not error");
        assert!(whitespace.is_empty(), "whitespace-only query must return no hits");
    }

    #[test]
    fn search_with_podcast_id_filter_excludes_other_podcasts() {
        let _guard = setup();
        let repo = DieselPodcastEpisodeTranscriptRepository::new(database());

        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);
        let transcript_id = repo
            .upsert(upsert_transcript(episode_id, Some("https://example.com/search-c.vtt")))
            .expect("upsert transcript c");
        repo.replace_segments(
            transcript_id,
            &[make_segment(0, "a unique zebra herd crossed the plain")],
        )
        .expect("replace segments c");

        let other_podcast_id = seed_podcast();

        // Filtering by the *other* podcast must find nothing, even though the
        // word exists in the DB (just under a different podcast_id).
        let filtered_out = repo
            .search("zebra", Uuid::parse_str(&other_podcast_id).ok(), 0, 20)
            .expect("search filtered by other podcast");
        assert!(
            filtered_out.iter().all(|h| h.episode_id != episode_id),
            "hit from a different podcast must be excluded"
        );

        // Filtering by the correct podcast must still find it.
        let filtered_in = repo
            .search("zebra", Uuid::parse_str(&podcast_id).ok(), 0, 20)
            .expect("search filtered by matching podcast");
        assert!(
            filtered_in.iter().any(|h| h.episode_id == episode_id),
            "hit from the matching podcast must be included"
        );
    }

    #[test]
    fn sanitize_sqlite_match_query_handles_empty_input() {
        assert_eq!(sanitize_sqlite_match_query(""), "");
        assert_eq!(sanitize_sqlite_match_query("   "), "");
    }

    #[test]
    fn sanitize_sqlite_match_query_strips_quotes() {
        assert_eq!(sanitize_sqlite_match_query(r#"say "hi""#), "\"say\"* \"hi\"*");
        assert_eq!(sanitize_sqlite_match_query("\"\"\"quoted\"\"\""), "\"quoted\"*");
    }

    #[test]
    fn sanitize_sqlite_match_query_joins_multiple_words_as_prefix_terms() {
        assert_eq!(sanitize_sqlite_match_query("quick fox"), "\"quick\"* \"fox\"*");
        assert_eq!(sanitize_sqlite_match_query("one"), "\"one\"*");
    }

    // ── TranscriptionJob tests ───────────────────────────────────────────

    #[test]
    fn job_queue_lifecycle() {
        let _guard = setup();
        clear_transcription_jobs();
        let repo = DieselTranscriptionJobRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let job = repo
            .enqueue(episode_id)
            .expect("enqueue")
            .expect("first enqueue should create a job");
        assert_eq!(job.status, TranscriptionJobStatus::Pending);
        assert_eq!(job.attempts, 0);

        let pending = repo.next_pending().expect("next_pending").expect("job pending");
        assert_eq!(pending.id, job.id);

        repo.set_status(job.id, TranscriptionJobStatus::Running, None)
            .expect("set_status running");

        assert!(
            repo.next_pending().expect("next_pending").is_none(),
            "no more pending jobs"
        );

        // Enqueuing again for the same episode must not create a second row.
        let second = repo.enqueue(episode_id).expect("second enqueue");
        assert!(second.is_none());

        let reset_count = repo.reset_running_to_pending().expect("reset running");
        assert_eq!(reset_count, 1);

        let pending_again = repo
            .next_pending()
            .expect("next_pending after reset")
            .expect("job should be pending again");
        assert_eq!(pending_again.id, job.id);
        assert_eq!(pending_again.status, TranscriptionJobStatus::Pending);
    }

    #[test]
    fn increment_attempts_and_get_by_episode_id() {
        let _guard = setup();
        clear_transcription_jobs();
        let repo = DieselTranscriptionJobRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        let job = repo.enqueue(episode_id).expect("enqueue").expect("created");

        let attempts1 = repo.increment_attempts(job.id).expect("increment 1");
        assert_eq!(attempts1, 1);
        let attempts2 = repo.increment_attempts(job.id).expect("increment 2");
        assert_eq!(attempts2, 2);

        let fetched = repo
            .get_by_episode_id(episode_id)
            .expect("get_by_episode_id")
            .expect("job exists");
        assert_eq!(fetched.attempts, 2);

        repo.set_status(job.id, TranscriptionJobStatus::Failed, Some("oops"))
            .expect("set_status failed");
        let failed = repo
            .get_by_episode_id(episode_id)
            .expect("get_by_episode_id")
            .expect("job exists");
        assert_eq!(failed.status, TranscriptionJobStatus::Failed);
        assert_eq!(failed.error, Some("oops".to_string()));
    }

    #[test]
    fn get_by_episode_id_returns_none_when_no_job() {
        let _guard = setup();
        let repo = DieselTranscriptionJobRepository::new(database());
        let podcast_id = seed_podcast();
        let episode_id = seed_episode(&podcast_id);

        assert!(
            repo.get_by_episode_id(episode_id)
                .expect("get_by_episode_id")
                .is_none()
        );
    }
}
