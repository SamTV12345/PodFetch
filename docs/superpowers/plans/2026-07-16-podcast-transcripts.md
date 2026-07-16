# Podcasting-2.0-Transkript-Unterstützung — Implementierungsplan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** PodFetch parst `<podcast:transcript>`-Tags, archiviert Transkript-Dateien, zeigt Transkripte im Player, macht sie volltextdurchsuchbar und kann fehlende Transkripte per OpenAI-kompatibler Whisper-API erzeugen.

**Architecture:** Hybrid-Speicherung (Originaldatei archiviert + normalisierte Segmente in DB), native DB-Volltextsuche (SQLite FTS5 / Postgres tsvector), DB-gestützte Job-Queue mit Tokio-Worker für Whisper. Alles folgt dem bestehenden Kapitel-Muster: Trait in `podfetch-domain` → Diesel-Repo in `podfetch-persistence` (+ Adapter in `adapters.rs`) → Service in `podfetch-web/src/services/` → Controller → React-UI.

**Tech Stack:** Rust (Axum, Diesel sync, Tokio), reqwest (+`multipart`-Feature, muss ergänzt werden), React 19 + TypeScript + TanStack Query + Zustand, socketioxide.

**Spec:** `docs/superpowers/specs/2026-07-16-podcast-transcripts-design.md` — bei Widerspruch gewinnt die Spec, außer bei der dort vermerkten Abweichung: VTT/SRT-Parser werden **handgeschrieben** (je ~50 Zeilen) statt über eine Crate — es gibt keine etablierte, gepflegte Rust-Crate für beide Formate, und die Formate sind trivial.

## Global Constraints

- Migrationen IMMER doppelt: `migrations/sqlite/<datum>_<name>/{up,down}.sql` UND `migrations/postgres/...` (gleicher Verzeichnisname). `migrations/mysql/` NICHT anfassen.
- Neue Tabellen: `diesel::table!`-Makro inline im jeweiligen Persistence-Modul deklarieren (Muster: `crates/podfetch-persistence/src/podcast_episode_chapter.rs:10-22`), NICHT in `schema.rs` — Ausnahme: die neue Spalte in `podcast_settings` (die Tabelle lebt in `crates/podfetch-persistence/src/podcast_settings.rs`).
- IDs sind UUIDs als `Text` in der DB; Domain-Typen nutzen `uuid::Uuid`; Konvertierung wie in `podcast_episode_chapter.rs:52-66`.
- Fehler-Typ überall `CustomError` (`common_infrastructure::error`); Repos liefern `PersistenceError`, Adapter in `adapters.rs` mappen auf `CustomError` (Muster `adapters.rs:517-548`).
- Transkript-Fehler sind für Feed-Refresh und Episoden-Download IMMER non-fatal (nur `tracing::error!` + Status in DB).
- Env-Var-Namen: `TRANSCRIPTION_API_BASE_URL`, `TRANSCRIPTION_API_KEY`, `TRANSCRIPTION_MODEL` (Default `"whisper-1"`).
- Test-Kommando Backend: `cargo test -p <crate> <testname>` (SQLite in-memory, Migrationen laufen automatisch via `embed_migrations!`). UI: `cd ui && npm run build`.
- Commits nur lokal — **NIEMALS pushen**.
- Code/Kommentare/Commit-Messages Englisch (Repo-Konvention).

---

### Task 1: Migrationen

**Files:**
- Create: `migrations/sqlite/2026-07-16-120000_podcast_transcripts/up.sql` + `down.sql`
- Create: `migrations/postgres/2026-07-16-120000_podcast_transcripts/up.sql` + `down.sql`
- Modify: `crates/podfetch-persistence/src/podcast_settings.rs` (Spalte `auto_transcribe` in `table!`, Entities, `From`-Impls — Muster: Spalte `auto_download`)
- Modify: `crates/podfetch-persistence/src/schema.rs` (`auto_transcribe` in `podcast_settings`)
- Modify: `crates/podfetch-domain/src/podcast_settings.rs` (`pub auto_transcribe: bool`)

**Interfaces:**
- Produces: Tabellen `podcast_episode_transcripts`, `podcast_episode_transcript_segments`, `transcript_segments_fts` (nur SQLite), `transcription_jobs`; Spalte `podcast_settings.auto_transcribe BOOLEAN NOT NULL DEFAULT FALSE`.

- [ ] **Step 1: SQLite-`up.sql` schreiben**

```sql
CREATE TABLE podcast_episode_transcripts (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    source TEXT NOT NULL, -- 'feed' | 'generated'
    original_url TEXT,
    file_path TEXT,
    mime_type TEXT NOT NULL,
    language TEXT,
    is_preferred BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'downloaded'|'parsed'|'failed'
    error TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
CREATE UNIQUE INDEX uq_transcripts_episode_url
    ON podcast_episode_transcripts (episode_id, original_url);
CREATE INDEX idx_transcripts_episode ON podcast_episode_transcripts (episode_id);

CREATE TABLE podcast_episode_transcript_segments (
    id TEXT PRIMARY KEY NOT NULL,
    transcript_id TEXT NOT NULL REFERENCES podcast_episode_transcripts(id) ON DELETE CASCADE,
    idx INTEGER NOT NULL,
    start_ms INTEGER,
    end_ms INTEGER,
    speaker TEXT,
    text TEXT NOT NULL
);
CREATE INDEX idx_segments_transcript ON podcast_episode_transcript_segments (transcript_id);

CREATE VIRTUAL TABLE transcript_segments_fts USING fts5(
    text,
    content='podcast_episode_transcript_segments',
    content_rowid='rowid'
);
CREATE TRIGGER transcript_segments_ai AFTER INSERT ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;
CREATE TRIGGER transcript_segments_ad AFTER DELETE ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text)
        VALUES ('delete', old.rowid, old.text);
END;
CREATE TRIGGER transcript_segments_au AFTER UPDATE ON podcast_episode_transcript_segments BEGIN
    INSERT INTO transcript_segments_fts(transcript_segments_fts, rowid, text)
        VALUES ('delete', old.rowid, old.text);
    INSERT INTO transcript_segments_fts(rowid, text) VALUES (new.rowid, new.text);
END;

CREATE TABLE transcription_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL UNIQUE REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending'|'running'|'done'|'failed'
    attempts INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

ALTER TABLE podcast_settings ADD COLUMN auto_transcribe BOOLEAN NOT NULL DEFAULT FALSE;
```

`down.sql`: `DROP`s in umgekehrter Reihenfolge (Trigger, FTS-Table, Tabellen); für SQLite die `auto_transcribe`-Spalte per `ALTER TABLE podcast_settings DROP COLUMN auto_transcribe;` (SQLite ≥3.35, von Diesel-Bundling erfüllt).

- [ ] **Step 2: Postgres-`up.sql` schreiben**

Identisch bis auf: `TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL` für Timestamps (Muster Chapter-Migration), **kein** FTS5-Block/Trigger, stattdessen:

```sql
ALTER TABLE podcast_episode_transcript_segments
    ADD COLUMN text_search tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', text)) STORED;
CREATE INDEX idx_segments_fts ON podcast_episode_transcript_segments USING GIN (text_search);
```

- [ ] **Step 3: `auto_transcribe` durch `podcast_settings.rs` (persistence, `schema.rs`) und `podcast_settings.rs` (domain) ziehen** — exakt jede Stelle nachziehen, an der `auto_download` vorkommt (`rg -n auto_download crates/`), inklusive Default `false` und DTO im Settings-Controller-Pfad.

- [ ] **Step 4: Kompilieren + bestehende Tests**

Run: `cargo test -p podfetch-persistence`
Expected: PASS (Migrationen laufen in in-memory SQLite durch; ein Syntaxfehler in up.sql schlägt hier sofort auf)

- [ ] **Step 5: Commit** — `git commit -m "feat(transcripts): add transcript, segment and job tables"`

---

### Task 2: Domain-Modelle & Traits

**Files:**
- Create: `crates/podfetch-domain/src/podcast_episode_transcript.rs`
- Modify: `crates/podfetch-domain/src/lib.rs` (`pub mod podcast_episode_transcript;`)

**Interfaces:**
- Produces (von allen Folge-Tasks konsumiert):

```rust
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptSource { Feed, Generated }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptStatus { Pending, Downloaded, Parsed, Failed }

#[derive(Debug, Clone)]
pub struct PodcastEpisodeTranscript {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub source: TranscriptSource,
    pub original_url: Option<String>,
    pub file_path: Option<String>,
    pub mime_type: String,
    pub language: Option<String>,
    pub is_preferred: bool,
    pub status: TranscriptStatus,
    pub error: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertTranscript {
    pub episode_id: Uuid,
    pub source: TranscriptSource,
    pub original_url: Option<String>,
    pub mime_type: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptSegment {
    pub idx: i32,
    pub start_ms: Option<i32>,
    pub end_ms: Option<i32>,
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TranscriptSearchHit {
    pub episode_id: Uuid,
    pub transcript_id: Uuid,
    pub start_ms: Option<i32>,
    pub snippet: String, // mit <b>…</b>-Highlight aus der DB
    pub rank: f32,
}

pub trait PodcastEpisodeTranscriptRepository: Send + Sync {
    type Error;
    /// Upsert über (episode_id, original_url); Rückgabe: id der Zeile.
    fn upsert(&self, transcript: UpsertTranscript) -> Result<Uuid, Self::Error>;
    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error>;
    fn get_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error>;
    fn set_file_path(&self, id: Uuid, file_path: &str) -> Result<(), Self::Error>;
    fn set_status(&self, id: Uuid, status: TranscriptStatus, error: Option<&str>) -> Result<(), Self::Error>;
    fn set_preferred(&self, episode_id: Uuid, preferred_id: Option<Uuid>) -> Result<(), Self::Error>;
    /// Löscht alte Segmente des Transkripts und fügt neue in einer Transaktion ein.
    fn replace_segments(&self, transcript_id: Uuid, segments: &[TranscriptSegment]) -> Result<(), Self::Error>;
    fn get_segments(&self, transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error>;
    fn search(&self, query: &str, podcast_id: Option<Uuid>, page: i64, page_size: i64)
        -> Result<Vec<TranscriptSearchHit>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptionJobStatus { Pending, Running, Done, Failed }

#[derive(Debug, Clone)]
pub struct TranscriptionJob {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub status: TranscriptionJobStatus,
    pub attempts: i32,
    pub error: Option<String>,
}

pub trait TranscriptionJobRepository: Send + Sync {
    type Error;
    /// Legt Job an; Err/None-Semantik: gibt Ok(None) zurück, wenn schon einer existiert (UNIQUE).
    fn enqueue(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error>;
    fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error>;
    fn set_status(&self, id: Uuid, status: TranscriptionJobStatus, error: Option<&str>) -> Result<(), Self::Error>;
    fn increment_attempts(&self, id: Uuid) -> Result<i32, Self::Error>;
    fn reset_running_to_pending(&self) -> Result<usize, Self::Error>;
    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error>;
}
```

Plus `impl TranscriptSource { pub fn as_str(&self) -> &'static str }` / `from_str` (Werte `"feed"`/`"generated"`), analog für beide Status-Enums (`"pending"`, `"downloaded"`, `"parsed"`, `"failed"` bzw. `"running"`, `"done"`).

- [ ] **Step 1: Datei mit obigem Inhalt anlegen, `lib.rs` erweitern**
- [ ] **Step 2: Unit-Test für die String-Konvertierungen im selben File (`#[cfg(test)] mod tests`)** — Roundtrip `from_str(x.as_str()) == x` für alle Varianten; unbekannter String → `None`.
- [ ] **Step 3:** Run: `cargo test -p podfetch-domain` — Expected: PASS
- [ ] **Step 4: Commit** — `feat(transcripts): add domain model and repository traits`

---

### Task 3: Diesel-Repository (CRUD + Segmente)

**Files:**
- Create: `crates/podfetch-persistence/src/podcast_episode_transcript.rs`
- Modify: `crates/podfetch-persistence/src/lib.rs`, `crates/podfetch-persistence/src/adapters.rs`

**Interfaces:**
- Consumes: Traits aus Task 2.
- Produces: `DieselPodcastEpisodeTranscriptRepository::new(database: Database)`, `DieselTranscriptionJobRepository::new(database: Database)`; Adapter `PodcastEpisodeTranscriptRepositoryImpl` / `TranscriptionJobRepositoryImpl` mit `Error = CustomError` (Muster `adapters.rs:517-548`).

- [ ] **Step 1: Failing Test schreiben** (im neuen Modul, `#[cfg(test)]`, Muster: bestehende Persistence-Tests mit in-memory-DB): Episode anlegen (Test-Helper wie in Nachbar-Tests), `upsert` zweimal mit gleicher (`episode_id`,`original_url`) → eine Zeile; `replace_segments` mit 3 Segmenten, dann mit 2 → `get_segments` liefert exakt 2 in `idx`-Reihenfolge.
- [ ] **Step 2:** Run: `cargo test -p podfetch-persistence transcript` — Expected: FAIL (Modul existiert nicht)
- [ ] **Step 3: Implementierung** — `diesel::table!`-Deklarationen für beide Tabellen inline (Spalten exakt wie Migration; `text_search` NICHT deklarieren, Diesel kennt die Spalte nicht → bei Inserts explizite Spaltenliste ist unnötig, da generated column). Entity-Structs + `From`-Impls, Methoden mit `ExpressionMethods`/`RunQueryDsl` exakt nach dem Muster von `podcast_episode_chapter.rs`. `replace_segments` in `connection.transaction(...)`: `delete(...filter(transcript_id.eq(...)))` dann Batch-`insert_into`. `set_preferred`: erst alle Zeilen der Episode auf `false`, dann optional eine auf `true` (eine Transaktion).
- [ ] **Step 4:** Run: `cargo test -p podfetch-persistence transcript` — Expected: PASS
- [ ] **Step 5: Job-Repo analog** (Failing Test: enqueue → next_pending → set_status(Running) → next_pending liefert None; zweites enqueue für gleiche Episode → Ok(None); reset_running_to_pending → wieder pending). Implementieren, Test grün.
- [ ] **Step 6: Adapter in `adapters.rs`** nach Muster `PodcastEpisodeChapterRepositoryImpl` (Delegation + Fehler-Mapping) für beide Repos.
- [ ] **Step 7:** Run: `cargo test -p podfetch-persistence` — Expected: PASS
- [ ] **Step 8: Commit** — `feat(transcripts): add diesel repositories for transcripts and jobs`

---

### Task 4: Volltextsuche im Repository

**Files:**
- Modify: `crates/podfetch-persistence/src/podcast_episode_transcript.rs`

**Interfaces:**
- Produces: `search()` gemäß Trait; Snippets mit `<b>`-Markierung; Treffer gruppiert die DB NICHT — Gruppierung „beste 3 Segmente pro Episode" macht der Service (Task 6).

- [ ] **Step 1: Failing Test**: zwei Episoden, Segmente mit bekannten Texten (`"the quick brown fox"` etc.) einfügen, `search("fox", None, 0, 20)` → Treffer enthält `episode_id`, `start_ms`, Snippet enthält `<b>fox</b>`; `search` mit `podcast_id`-Filter der anderen Episode → leer.
- [ ] **Step 2:** Run: `cargo test -p podfetch-persistence search` — Expected: FAIL
- [ ] **Step 3: Implementierung** mit `diesel::sql_query` + `QueryableByName`-Row-Struct und Backend-Weiche (dasselbe Muster, mit dem `db.rs` SQLite/Postgres unterscheidet — `Database`-Enum matchen):

SQLite:
```sql
SELECT t.episode_id, s.transcript_id, s.start_ms,
       highlight(transcript_segments_fts, 0, '<b>', '</b>') AS snippet,
       bm25(transcript_segments_fts) AS rank
FROM transcript_segments_fts
JOIN podcast_episode_transcript_segments s ON s.rowid = transcript_segments_fts.rowid
JOIN podcast_episode_transcripts t ON t.id = s.transcript_id
JOIN podcast_episodes e ON e.id = t.episode_id
WHERE transcript_segments_fts MATCH ?
  AND (?2 IS NULL OR e.podcast_id = ?2)
ORDER BY rank LIMIT ?3 OFFSET ?4
```

Postgres:
```sql
SELECT t.episode_id, s.transcript_id, s.start_ms,
       ts_headline('simple', s.text, websearch_to_tsquery('simple', $1),
                   'StartSel=<b>,StopSel=</b>') AS snippet,
       ts_rank(s.text_search, websearch_to_tsquery('simple', $1)) AS rank
FROM podcast_episode_transcript_segments s
JOIN podcast_episode_transcripts t ON t.id = s.transcript_id
JOIN podcast_episodes e ON e.id = t.episode_id
WHERE s.text_search @@ websearch_to_tsquery('simple', $1)
  AND ($2::text IS NULL OR e.podcast_id = $2)
ORDER BY rank DESC LIMIT $3 OFFSET $4
```

Query-Eingabe fürs SQLite-`MATCH` säubern: `"`-Zeichen entfernen, Wörter mit `*`-Suffix verbinden (`quick fox` → `"quick"* "fox"*`), damit Nutzereingaben nie FTS5-Syntaxfehler auslösen. Bei Postgres übernimmt `websearch_to_tsquery` die Härtung.
- [ ] **Step 4:** Run: `cargo test -p podfetch-persistence search` — Expected: PASS (SQLite; Postgres läuft in der bestehenden CI-Matrix)
- [ ] **Step 5: Commit** — `feat(transcripts): add full-text search over transcript segments`

---

### Task 5: Format-Parser

**Files:**
- Create: `crates/podfetch-web/src/services/transcript/mod.rs` (`pub mod parser; pub mod service; pub mod whisper_client; pub mod worker;` — Module erst deklarieren, wenn sie entstehen; in diesem Task nur `parser`)
- Create: `crates/podfetch-web/src/services/transcript/parser.rs`
- Create: `crates/podfetch-web/src/services/transcript/fixtures/` (`sample.json`, `sample.vtt`, `sample.srt`, `sample.html`, `broken.vtt`, `empty.json`)
- Modify: `crates/podfetch-web/src/services/mod.rs` (`pub mod transcript;`)

**Interfaces:**
- Consumes: `TranscriptSegment` aus Task 2.
- Produces:

```rust
pub enum TranscriptFormat { Json, Vtt, Srt, Html }

impl TranscriptFormat {
    /// Ordnet mime_type (primär) bzw. URL-Endung (Fallback) zu; None = nicht unterstützt.
    pub fn detect(mime_type: &str, url: Option<&str>) -> Option<Self>;
    /// Präferenzrang: Json=0 < Vtt=1 < Srt=2 < Html=3 (kleiner = besser)
    pub fn preference_rank(&self) -> u8;
}

/// Parst Rohbytes zu Segmenten. Fehler = nicht parsebar (Aufrufer setzt status).
pub fn parse(format: TranscriptFormat, raw: &[u8]) -> Result<Vec<TranscriptSegment>, TranscriptParseError>;
```

- [ ] **Step 1: Fixtures anlegen.** `sample.json` im Podcast-Namespace-Format (`{"segments":[{"speaker":"Alice","startTime":0.5,"endTime":4.2,"body":"Hello world"}, …]}`), `sample.vtt` (Header `WEBVTT`, Cues `00:00:00.500 --> 00:00:04.200` mit `<v Alice>`-Voice-Tag in einem Cue), `sample.srt` (nummerierte Cues, Kommazeit `00:00:00,500`), `sample.html` (`<p><cite>Alice:</cite> <time>0:00</time> Hello world</p>`-Struktur und ein `<p>` ohne `<time>`), `broken.vtt` (Zeitzeile kaputt), `empty.json` (`{}`).
- [ ] **Step 2: Failing Tests** (Tabelle: Datei → erwartete Segmente): pro Format Anzahl, erstes Segment (`start_ms == Some(500)`, `speaker == Some("Alice")`, Text), HTML ohne `<time>` → `start_ms == None`; `broken.vtt` → `Err`; `empty.json` → `Err`; `detect("text/vtt", None) == Some(Vtt)`, `detect("application/json", …) == Some(Json)`, `detect("application/srt", …) == Some(Srt)`, `detect("text/html", …) == Some(Html)`, `detect("text/plain", Some("https://x/y.srt")) == Some(Srt)`, `detect("application/pdf", None) == None`.
- [ ] **Step 3:** Run: `cargo test -p podfetch-web transcript::parser` — Expected: FAIL
- [ ] **Step 4: Implementieren.** JSON via `serde_json` (Struct mit `segments: Vec<{speaker?, startTime: f64?, endTime?, body}>`, Sekunden → ms gerundet). VTT/SRT handgeschrieben: Blöcke an Leerzeilen splitten, Zeitzeile per Regex `(\d+):(\d{2}):(\d{2})[.,](\d{3})` (Stunden optional für VTT `MM:SS.mmm`), `<v Name>`-Prefix als Speaker extrahieren und Tag aus dem Text strippen. HTML: mit dem bereits vorhandenen HTML-Parsing des Projekts, falls eine Crate da ist (`rg -n scraper crates/*/Cargo.toml`), sonst simples Tag-Stripping mit Regex: `<time>`-Inhalt (`H:MM:SS` oder `M:SS`) zu ms, `<cite>`-Inhalt (ohne `:`) als Speaker, Rest-Text enttaggt. Leeres Segment-Ergebnis ⇒ `Err(TranscriptParseError::Empty)`.
- [ ] **Step 5:** Run: `cargo test -p podfetch-web transcript::parser` — Expected: PASS
- [ ] **Step 6: Commit** — `feat(transcripts): add parsers for json, vtt, srt and html transcripts`

---

### Task 6: TranscriptService (Präferenz, Download+Archiv, Parse, Suche, Re-Parse)

**Files:**
- Create: `crates/podfetch-web/src/services/transcript/service.rs`
- Modify: `crates/podfetch-web/src/app_state.rs` (Service registrieren, Muster `podcast_episode_chapter_service`: Feld `pub transcript_service: Arc<TranscriptService>` + Konstruktion in `AppState::new`)

**Interfaces:**
- Consumes: Repos (Task 3), Parser (Task 5), `FileHandleWrapper::write_file` + `ENVIRONMENT_SERVICE.default_file_handler` (Storage-Abstraktion, Nutzung siehe `services/download/service.rs:304-308`), `reqwest::blocking::Client`.
- Produces:

```rust
pub struct TranscriptService { /* Arc<dyn …Repository<Error=CustomError>> Felder */ }
impl TranscriptService {
    pub fn new(transcript_repo: …, job_repo: …) -> Self;
    pub fn default_service() -> Self; // Muster PodcastEpisodeChapterService::default_service()
    /// Feed-Tags upserten (Flow 1). Rein DB, kein HTTP.
    pub fn upsert_from_feed(&self, episode_id: Uuid, tags: &[FeedTranscriptTag]) -> Result<(), CustomError>;
    /// Flow 2: alle pending-Transkripte der Episode holen, archivieren, bevorzugtes parsen.
    pub fn process_pending_for_episode(&self, episode: &PodcastEpisode) -> Result<(), CustomError>;
    /// Präferenzregeln aus der Spec anwenden (nur status='parsed'; feed>generated; Formatrang; Sprache).
    pub fn recompute_preferred(&self, episode_id: Uuid) -> Result<(), CustomError>;
    pub fn get_preferred_segments(&self, episode_id: Uuid) -> Result<Option<(PodcastEpisodeTranscript, Vec<TranscriptSegment>)>, CustomError>;
    pub fn search(&self, query: &str, podcast_id: Option<Uuid>, page: i64) -> Result<Vec<TranscriptSearchGroup>, CustomError>;
    /// true ⇔ auto_transcribe soll für diese Episode einen Job anlegen (Spec-Regel 4).
    pub fn needs_generated_transcript(&self, episode_id: Uuid) -> Result<bool, CustomError>;
    pub fn reparse_all(&self) -> Result<ReparseReport, CustomError>; // {reparsed: usize, failed: usize}
    /// Dünner Wrapper um TranscriptionJobRepository::enqueue; None = Job existiert schon.
    pub fn enqueue_job(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, CustomError>;
}

pub struct FeedTranscriptTag { pub url: String, pub mime_type: String, pub language: Option<String> }
/// Suchtreffer gruppiert: pro Episode max. 3 Segmente, Reihenfolge nach bestem Rang.
pub struct TranscriptSearchGroup { pub episode_id: Uuid, pub hits: Vec<TranscriptSearchHit> }
```

- [ ] **Step 1: Failing Tests für `recompute_preferred`** (in-memory DB via `test_support`): (a) nur generated-parsed → preferred; (b) feed-parsed kommt dazu → Präferenz wechselt; (c) feed-failed + generated-parsed → generated bleibt; (d) zwei feed-parsed (VTT+JSON) → JSON gewinnt.
- [ ] **Step 2:** Run + FAIL, dann implementieren (Auswahl in Rust-Code: filtern auf `Parsed`, sortieren nach `(source != Feed, format_rank, language != podcast_language)`, erstes Element via `set_preferred`).
- [ ] **Step 3: Failing Test für `process_pending_for_episode`** mit lokalem Mock-HTTP-Server (Test-Axum-Server auf Ephemeral-Port, liefert `fixtures/sample.vtt` als `text/vtt`; zweite Route liefert 10 MB Müll für das Größenlimit): (a) Erfolgsfall → `file_path` gesetzt, Datei existiert unter dem Episoden-Verzeichnis als `<stem>.transcript.vtt`, Status `parsed`, Segmente vorhanden, `is_preferred` gesetzt; (b) 404 → Status `failed` + `error`, Rückgabe trotzdem `Ok(())`; (c) Download > 20 MB → `failed`; (d) parsebares Format unbekannt (`application/pdf`) → nur archiviert, Status `downloaded`.
- [ ] **Step 4:** Implementieren: reqwest-blocking-GET mit `timeout(Duration::from_secs(30))`, Body-Bytes limitiert lesen (`take(20 * 1024 * 1024 + 1)` und Abbruch bei Überschreitung), `FileHandleWrapper::write_file`, `TranscriptFormat::detect(mime_type, original_url)`, `parse`, `replace_segments`, `set_status`, `recompute_preferred`. Extension fürs Archiv aus dem Format (`json|vtt|srt|html`). Tests grün.
- [ ] **Step 5: `search`-Gruppierung testen + implementieren**: Repo liefert flache Hits (page_size = 60 intern), Service gruppiert per `episode_id`, kappt auf 3 pro Episode, sortiert Gruppen nach bestem Rang. Test mit gestubbtem Repo (kleines struct, das das Trait implementiert) — 7 Hits über 2 Episoden → Gruppe A 3 Hits, Gruppe B 3, Rest verworfen.
- [ ] **Step 6: `reparse_all` + `needs_generated_transcript` implementieren** (Tests: failed-feed-Transkript ⇒ `needs == true`; pending oder parsed ⇒ `false`). AppState-Registrierung.
- [ ] **Step 7:** Run: `cargo test -p podfetch-web transcript` — Expected: PASS
- [ ] **Step 8: Commit** — `feat(transcripts): add transcript service with archive, parse and preference logic`

---

### Task 7: Feed-Parse-Hook (Flow 1)

**Files:**
- Modify: `crates/podfetch-web/src/usecases/podcast_episode/mod.rs` (in `insert_podcast_episodes`, im Item-Loop ab Zeile ~520)
- Test: ebd. `#[cfg(test)]` bzw. bestehende Testdatei des Moduls

**Interfaces:**
- Consumes: `TranscriptService::upsert_from_feed` (Task 6).
- Produces: Hilfsfunktion `fn extract_transcript_tags(item: &rss::Item) -> Vec<FeedTranscriptTag>`.

- [ ] **Step 1: Failing Test**: RSS-Item-Fixture mit zwei `<podcast:transcript>`-Tags (VTT en, JSON en) bauen — `rss::Item` mit `extensions`-BTreeMap von Hand konstruieren (`item.extensions_mut().entry("podcast".into())…` mit `rss::extension::ExtensionBuilder`: `name: "podcast:transcript"`, `attrs: {"url": …, "type": …, "language": …}`) → `extract_transcript_tags` liefert 2 Tags mit korrekten Feldern; Tag ohne `url`-Attribut wird übersprungen.
- [ ] **Step 2:** Run + FAIL, implementieren: `item.extensions().get("podcast")` → Map-Key ist der Extension-Name ohne Prefix (`"transcript"`) — **beides prüfen** (`"transcript"` und `"podcast:transcript"`), die rss-Crate normalisiert je nach Feed-Deklaration unterschiedlich. Attribute `url` (Pflicht), `type` (Default `"text/plain"`), `language` (optional).
- [ ] **Step 3: Hook einbauen**: im Item-Loop nach dem Episoden-Upsert (`opt_found_podcast_episode`-Zweig, sowohl Update- als auch Insert-Pfad) `TranscriptService::default_service().upsert_from_feed(episode_uuid, &tags)`, Fehler nur loggen (`tracing::error!`). Idempotenz-Test: `insert_podcast_episodes` zweimal auf denselben Feed → weiterhin 2 Transcript-Zeilen (Upsert-Key greift).
- [ ] **Step 4:** Run: `cargo test -p podfetch-web podcast_episode` — Expected: PASS
- [ ] **Step 5: Commit** — `feat(transcripts): extract podcast:transcript tags during feed refresh`

---

### Task 8: Download-Hook (Flow 2)

**Files:**
- Modify: `crates/podfetch-web/src/services/download/service.rs` (direkt nach dem Kapitel-Block, Zeilen 310-333)

**Interfaces:**
- Consumes: `TranscriptService::process_pending_for_episode` (Task 6). (Der auto_transcribe-Job-Enqueue kommt erst in Task 11, wenn Config und Worker existieren.)

- [ ] **Step 1: Hook einbauen** (kein eigener Test hier — die Bausteine sind in Task 6 getestet; der Hook folgt dem Stil des SponsorBlock-Blocks Zeile 335-339):

```rust
// Transcripts: download + parse feed transcripts.
// Non-fatal — must never fail the download.
let transcript_service = crate::services::transcript::service::TranscriptService::default_service();
if let Err(err) = transcript_service.process_pending_for_episode(&podcast_episode) {
    tracing::error!("Error processing transcripts for episode {}: {err}", podcast_episode.id);
}
```

- [ ] **Step 2: Smoke-Test**: bestehenden Download-Test des Moduls laufen lassen (`cargo test -p podfetch-web download`) — Expected: PASS (Hook darf nichts brechen; ohne pending-Transkripte ist er ein No-Op).
- [ ] **Step 3: Commit** — `feat(transcripts): process feed transcripts after episode download`

---

### Task 9: Konfiguration (Env + sys/config)

**Files:**
- Modify: `crates/common-infrastructure/src/config.rs` (Konstanten + `TranscriptionConfig` im `EnvironmentService`, Muster Telegram Zeile 383-388)
- Modify: `crates/podfetch-web/src/controllers/sys_info_controller.rs` (Flag `transcriptionEnabled` im Config-DTO)

**Interfaces:**
- Produces:

```rust
pub const TRANSCRIPTION_API_BASE_URL: &str = "TRANSCRIPTION_API_BASE_URL";
pub const TRANSCRIPTION_API_KEY: &str = "TRANSCRIPTION_API_KEY";
pub const TRANSCRIPTION_MODEL: &str = "TRANSCRIPTION_MODEL";

#[derive(Clone)]
pub struct TranscriptionConfig {
    pub base_url: String,          // ohne trailing slash normalisiert
    pub api_key: Option<String>,
    pub model: String,             // Default "whisper-1"
}
// im EnvironmentService: pub transcription_config: Option<TranscriptionConfig>
// None ⇔ TRANSCRIPTION_API_BASE_URL nicht gesetzt
```

- [ ] **Step 1: Failing Test** (config.rs hat bestehende Env-Tests als Muster): Env gesetzt → `Some` mit Feldern und Default-Modell; nicht gesetzt → `None`; trailing slash wird entfernt.
- [ ] **Step 2:** Implementieren, Test grün: `cargo test -p common-infrastructure`
- [ ] **Step 3: sys/config-DTO erweitern** (`transcription_enabled: bool` aus `ENVIRONMENT_SERVICE.transcription_config.is_some()`), bestehenden sys/config-Test um das Feld ergänzen.
- [ ] **Step 4:** Run: `cargo test -p podfetch-web sys` — Expected: PASS
- [ ] **Step 5: Commit** — `feat(transcripts): add transcription API configuration`

---

### Task 10: Whisper-Client

**Files:**
- Modify: `Cargo.toml` (Zeile 122: reqwest-Features um `"multipart"` ergänzen)
- Create: `crates/podfetch-web/src/services/transcript/whisper_client.rs`

**Interfaces:**
- Consumes: `TranscriptionConfig` (Task 9), `TranscriptSegment` (Task 2).
- Produces:

```rust
pub struct WhisperClient { config: TranscriptionConfig, client: reqwest::blocking::Client }
impl WhisperClient {
    pub fn new(config: TranscriptionConfig) -> Self;
    /// POST {base_url}/v1/audio/transcriptions, multipart: file=<audio>, model, response_format=verbose_json.
    /// Antwort: {"language": "...", "segments":[{"start":0.5,"end":4.2,"text":"…"}]}
    pub fn transcribe(&self, audio_path: &std::path::Path)
        -> Result<(Vec<TranscriptSegment>, Option<String> /* language */), CustomError>;
}
/// Segmente als WebVTT serialisieren (für die Archiv-Datei generierter Transkripte).
pub fn segments_to_vtt(segments: &[TranscriptSegment]) -> String;
```

- [ ] **Step 1: Failing Tests**: (a) `segments_to_vtt` — 2 Segmente → String beginnt mit `WEBVTT`, enthält `00:00:00.500 --> 00:00:04.200`; Roundtrip durch den VTT-Parser aus Task 5 ergibt die Segmente zurück. (b) `transcribe` gegen Test-Axum-Server (Ephemeral-Port), der multipart entgegennimmt und festes verbose_json liefert → Segmente korrekt (Sekunden→ms), `Authorization: Bearer`-Header nur wenn `api_key` gesetzt (Server asserted das). (c) Server liefert 500 → `Err`.
- [ ] **Step 2:** Run + FAIL, implementieren (reqwest `blocking::multipart::Form::new().file("file", path)` + `.text("model", …)` + `.text("response_format", "verbose_json")`; Timeout 600 s — Transkription dauert).
- [ ] **Step 3:** Run: `cargo test -p podfetch-web whisper` — Expected: PASS
- [ ] **Step 4: Commit** — `feat(transcripts): add openai-compatible whisper client`

---

### Task 11: Job-Worker

**Files:**
- Create: `crates/podfetch-web/src/services/transcript/worker.rs`
- Modify: `crates/podfetch-web/src/startup.rs` (Worker-Spawn beim Boot, dort wo andere Background-Tasks/Scheduler gestartet werden — `rg -n "tokio::spawn" crates/podfetch-web/src/startup.rs`)
- Modify: `crates/podfetch-web/src/server.rs` (neue Broadcast-Fn)

**Interfaces:**
- Consumes: `TranscriptionJobRepository`, `WhisperClient`, `TranscriptService` (Datei schreiben + `replace_segments` + `recompute_preferred` via neuer Methode `store_generated(&self, episode: &PodcastEpisode, segments, language) -> Result<(), CustomError>`), `ChatServerHandle`-Broadcast-Muster (`server.rs:87-99`).
- Produces: `pub async fn run_transcription_worker()` (Endlos-Loop) und `ChatServerHandle::broadcast_transcription_status(episode_id: &str, status: &str, error: Option<&str>)` (SocketIO-Event `transcriptionStatus`).

- [ ] **Step 1: Failing Test für die Job-Schleife als synchrone Funktion** `fn process_one_job(job_repo, service, client) -> Result<bool /* job gefunden */, CustomError>`: (a) kein Job → `Ok(false)`; (b) Job + Mock-Whisper-Server ok → Job `done`, generiertes Transkript `parsed` mit `source='generated'`, VTT-Datei existiert; (c) Whisper-Fehler und `attempts == 3` → `failed` + `error`, bei `attempts < 3` → zurück auf `pending`.
- [ ] **Step 2:** Run + FAIL, implementieren. Worker-Async-Loop ist dünn: `loop { spawn_blocking(process_one_job); wenn Ok(false) → sleep(15s) }`; nur starten, wenn `transcription_config.is_some()`. Beim Start einmalig `reset_running_to_pending()`. Statuswechsel broadcasten.
- [ ] **Step 3: auto_transcribe-Enqueue in den Download-Hook** (`services/download/service.rs`, direkt unter dem Block aus Task 8):

```rust
if ENVIRONMENT_SERVICE.transcription_config.is_some()
    && podcast_settings.auto_transcribe // die Settings dieses Podcasts sind an dieser Stelle bereits geladen
    && transcript_service.needs_generated_transcript(episode_uuid).unwrap_or(false)
{
    if let Err(err) = transcript_service.enqueue_job(episode_uuid) {
        tracing::error!("Error enqueuing transcription job: {err}");
    }
}
```

- [ ] **Step 4:** Run: `cargo test -p podfetch-web worker && cargo test -p podfetch-web download` — Expected: PASS
- [ ] **Step 5: Commit** — `feat(transcripts): add transcription job worker and auto-transcribe enqueue`

---

### Task 12: HTTP-Controller

**Files:**
- Create: `crates/podfetch-web/src/controllers/transcript_controller.rs`
- Modify: `crates/podfetch-web/src/controllers/mod.rs`, `crates/podfetch-web/src/startup.rs` (Router mergen, Muster `get_sponsorblock_router()`-Merge in `startup.rs:357`)

**Interfaces:**
- Consumes: `AppState.transcript_service`, Rollen-Extension (`Extension(requester): Extension<User>` + `requester.role`-Checks wie in Nachbar-Controllern), apiKey-Datei-Zugriff (Muster `/proxy/podcast/apiKey/{apiKey}` bzw. RSS-apiKey-Routen).
- Produces (alle unter `/api/v1`, DTOs mit `#[derive(Serialize, ToSchema)]`):

| Route | Handler | Auth |
|---|---|---|
| `GET /podcasts/episodes/{id}/transcripts` | `get_transcripts_of_episode` → `Vec<TranscriptDto>` (id, source, language, mime_type, status, error) | eingeloggt |
| `GET /podcasts/episodes/{id}/transcript` | `get_preferred_transcript` → `TranscriptWithSegmentsDto` oder 404 | eingeloggt |
| `GET /podcasts/episodes/{id}/transcripts/{tid}/file` | `get_transcript_file` → Datei-Stream, `Content-Type = mime_type` | eingeloggt |
| `GET /podcasts/episodes/{id}/transcripts/{tid}/file/apiKey/{api_key}` | dieselbe Logik nach apiKey-Prüfung | apiKey |
| `POST /podcasts/episodes/{id}/transcribe` | `enqueue_transcription` → 200/409 (Job existiert)/503 (`transcription_config == None`) | Uploader+ |
| `GET /transcripts/search?q&podcast_id&page` | `search_transcripts` → `Vec<TranscriptSearchGroupDto>` | eingeloggt |
| `POST /settings/transcripts/reparse` | `reparse_transcripts` → `ReparseReportDto` | Admin |

- [ ] **Step 1: Failing Integrationstests** mit `test_support` (Muster: `podcast_episode_controller.rs:838-860`): Episode ohne Transkript → `/transcript` 404, `/transcripts` `[]`; mit geseedeten Segmenten → 200 + Inhalt; `search` findet geseedetes Segment; `transcribe` ohne Config → 503; `reparse` als Nicht-Admin → 403.
- [ ] **Step 2:** Run: `cargo test -p podfetch-web transcript_controller` — Expected: FAIL
- [ ] **Step 3: Implementieren** (utoipa-Annotationen wie `podcast_episode_controller.rs:53-60`, Registrierung via `.routes(routes!(…))`).
- [ ] **Step 4:** Run — Expected: PASS
- [ ] **Step 5: Commit** — `feat(transcripts): add transcript http endpoints`

---

### Task 13: RSS-Re-Export

**Files:**
- Modify: `crates/podfetch-web/src/controllers/websocket_controller.rs` (Item-Generierung, `ItemBuilder`-Block Zeile ~280-296)

**Interfaces:**
- Consumes: `TranscriptService::get_by_episode_id` (nur `status IN ('parsed','downloaded')` und `file_path IS NOT NULL`), apiKey-Datei-Route aus Task 12.

- [ ] **Step 1: Failing Test** (bestehende RSS-Tests im File als Muster, z.B. Zeile 662 ff.): Episode mit archiviertem Transkript seeden → generierter Feed enthält `<podcast:transcript url=".../transcripts/<tid>/file/apiKey/..." type="text/vtt"/>` und die Channel-Namespace-Deklaration `xmlns:podcast="https://podcastindex.org/namespace/1.0"`.
- [ ] **Step 2:** Implementieren: Namespace über `ChannelBuilder.namespaces(BTreeMap)`, Tag über `item.extensions_mut()` + `rss::extension::ExtensionBuilder` (`name: "podcast:transcript"`, attrs url/type/language).
- [ ] **Step 3:** Run: `cargo test -p podfetch-web websocket_controller` — Expected: PASS
- [ ] **Step 4: Commit** — `feat(transcripts): expose transcripts in generated rss feeds`

---

### Task 14: UI — Transkript-Tab im Player

**Files:**
- Create: `ui/src/components/PodcastEpisodeTranscript.tsx`
- Modify: `ui/src/components/DetailedAudioPlayer.tsx` (Tab-Leiste Zeile ~118-140, Muster `chapters`-Tab)
- Modify: `ui/src/language/json/{da,de,en,es,fr,pl,zh}.json` (Keys `transcript`, `transcript-auto-scroll`, `no-transcript-available`)
- Modify: API-Client (wo `chapters`-Fetch definiert ist: `rg -n "chapters" ui/src/utils/`, gleiche Stelle) — Typ + Query für `GET /podcasts/episodes/{id}/transcript`

**Interfaces:**
- Consumes: `TranscriptWithSegmentsDto` (Task 12), Player-Store (`ui/src/store/AudioPlayerSlice.ts`: aktuelle Zeit + Seek-Mechanik — dieselbe, die `PodcastEpisodeChapterTable` fürs Springen nutzt; falls die Chapter-Tabelle direkt das `<audio>`-Element steuert, identisch verfahren).

- [ ] **Step 1: Komponente schreiben.** Props `{ podcastEpisode }`; TanStack-Query auf den Transcript-Endpoint; Rendering: Liste von Segmenten (Zeitstempel `mm:ss` + optional Speaker fett + Text), aktives Segment (currentTime zwischen start/end) hervorgehoben (`ui-text-accent`) und per `ref.scrollIntoView({block:'nearest'})` im sichtbaren Bereich gehalten; Checkbox „Auto-Scroll" (Default an); `onClick` eines Segments → Seek zu `start_ms/1000`; Segmente ohne `start_ms` → reine Textliste ohne Zeitspalte; leere Antwort → `no-transcript-available`-Hinweis.
- [ ] **Step 2: Tab ergänzen** in `DetailedAudioPlayer.tsx` — dritter Tab `transcript` exakt nach dem `chapters`-Muster (Zeilen 118-119 kopieren/anpassen, Render-Zweig Zeile 140).
- [ ] **Step 3: i18n-Keys in alle 7 Dateien** (englischer Text als Fallback in allen, echte Übersetzung für de/en; TODO-Übersetzungen sind hier ok, das Projekt handhabt fehlende Übersetzungen so).
- [ ] **Step 4: Verifizieren:** `cd ui && npm run build` — Expected: baut ohne Fehler. Manuelle Sichtprüfung, falls Dev-Server läuft (`npm run dev`), sonst Screenshot-Check beim finalen Verify.
- [ ] **Step 5: Commit** — `feat(transcripts): add transcript tab to detailed audio player`

---

### Task 15: UI — Transkript-Suche

**Files:**
- Modify: `ui/src/pages/EpisodeSearchPage.tsx`
- Modify: API-Client (Query für `GET /transcripts/search`)
- Modify: alle 7 Sprachdateien (Keys `search-in-transcripts`, `search-in-metadata`)

**Interfaces:**
- Consumes: `TranscriptSearchGroupDto` (Task 12), `PlayHandler` (`ui/src/utils/PlayHandler.ts`) zum Starten einer Episode + Seek.

- [ ] **Step 1: Umschalter einbauen** (Segmented Control „Titel/Beschreibung | Transkripte" über dem Suchfeld; bestehende Suche unverändert als erster Modus).
- [ ] **Step 2: Transkript-Modus:** debounced Query auf `/transcripts/search?q=…`; Ergebnis-Rendering pro Episoden-Gruppe: Episoden-Card (Bild/Name wie bestehende Suchtreffer) + bis zu 3 Snippets (Snippet-HTML mit `<b>` — über `dangerouslySetInnerHTML` NUR nach Escaping alles außer `<b>/</b>`: erst HTML-escapen, dann `&lt;b&gt;`/`&lt;/b&gt;` zurückersetzen) + Zeitstempel-Badge; Klick → `PlayHandler`-Start + Seek zu `start_ms`.
- [ ] **Step 3:** `cd ui && npm run build` — Expected: PASS
- [ ] **Step 4: Commit** — `feat(transcripts): add transcript full-text search to episode search page`

---

### Task 16: UI — Transkribieren-Aktion & Podcast-Einstellung

**Files:**
- Modify: Podcast-Settings-UI (Ort: `rg -n "auto_download" ui/src` — derselbe Dialog bekommt den `auto_transcribe`-Toggle; Settings-Controller-DTO wurde in Task 1 erweitert, API-Aufruf analog)
- Modify: Episoden-Kontextmenü/-Detail (Ort: `rg -n "download" ui/src/components/PodcastDetailItem*` bzw. wo Episoden-Aktionen liegen): Aktion „Transkribieren" + Status-Badge
- Modify: SocketIO-Handling (`rg -n "podcastRefreshed|socket" ui/src` → zentrale Stelle der Event-Handler): Event `transcriptionStatus` → Query-Invalidierung + Badge-Update
- Modify: `ui/src/store/CommonSlice.ts` o.ä. für das `transcriptionEnabled`-Flag aus `/sys/config`
- Modify: alle 7 Sprachdateien (`transcribe`, `transcription-pending`, `transcription-running`, `transcription-failed`, `auto-transcribe`)

- [ ] **Step 1: `transcriptionEnabled` aus sys/config in den Store**; Toggle + Aktion nur rendern, wenn `true`.
- [ ] **Step 2: Aktion „Transkribieren"**: POST auf `/podcasts/episodes/{id}/transcribe`; 409 → Toast „läuft bereits"; Status-Badge aus `GET /podcasts/episodes/{id}/transcripts` (existiert ein `generated`-Eintrag bzw. Job-Fehler → `error`-Tooltip).
- [ ] **Step 3: `auto_transcribe`-Toggle** im Podcast-Settings-Dialog (exakt neben `auto_download`, gleiche Update-Mutation).
- [ ] **Step 4: SocketIO-Event verdrahten** (Invalidate der Transcript-Query der betroffenen Episode).
- [ ] **Step 5:** `cd ui && npm run build` — Expected: PASS
- [ ] **Step 6: Commit** — `feat(transcripts): add transcribe action, status and auto-transcribe setting to ui`

---

### Task 17: Doku + Gesamtverifikation

**Files:**
- Create: `docs/src/transcripts.md` (Feature-Beschreibung: Feed-Transkripte, Suche, Whisper-Setup mit speaches/faster-whisper-Beispiel `docker compose`-Snippet, Env-Vars-Tabelle)
- Modify: `docs/src/SUMMARY.md` (Eintrag „Transcripts"), `README.md` (Feature-Liste um Transkripte + Volltextsuche ergänzen)

- [ ] **Step 1: Doku schreiben** (Env-Vars exakt wie Task 9; Beispiel: `TRANSCRIPTION_API_BASE_URL=http://speaches:8000`).
- [ ] **Step 2: Gesamtverifikation:**

Run: `cargo test --workspace` — Expected: PASS
Run: `cargo clippy --workspace -- -D warnings` — Expected: keine neuen Warnungen
Run: `cd ui && npm run build` — Expected: PASS
End-to-End-Smoke (verify-Skill): Server lokal starten, Podcast mit Transkript-Feed hinzufügen (z.B. „Podcasting 2.0"-Testfeed oder lokaler Fixture-Feed), Episode herunterladen → Transkript-Tab zeigt Segmente, Suche findet Text, RSS-Feed enthält das Tag.
- [ ] **Step 3: Commit** — `docs: document transcript support` — **NICHT pushen.**

---

## Selbstreview-Protokoll (bereits eingearbeitet)

- Spec-Abdeckung: Flows 1-4 → Tasks 7, 6/8, 10/11, 6 (`reparse_all`) + Endpoint Task 12; Präferenzregeln → Task 6; FTS beide Backends → Tasks 1/4; RSS-Export → Task 13; UI-Punkte → Tasks 14-16; Doku → Task 17. Abweichung von der Spec (handgeschriebene VTT/SRT-Parser statt Crate) ist im Header deklariert und muss beim Merge in der Spec nachgezogen werden.
- Abhängigkeiten: Tasks sind in Nummernreihenfolge ausführbar (Task 8 nutzt nur Task 6; alles Config-abhängige liegt in 9-12).
