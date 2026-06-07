# Download Options: NFO Files, `{episodeNumber}` Token & Jellyfin Polish — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add configurable Jellyfin/Kodi `.nfo` sidecar generation, an `{episodeNumber}` filename token, an `.m4a`/`.mp4` episode-numbering fix, and a configurable podcast-cover filename to PodFetch.

**Architecture:** A dedicated `services/nfo` module holds **pure** XML builders (domain types in → XML string out, `quick_xml::Writer`) plus a thin, **non-fatal** writer that resolves the format from settings and places files. It's invoked at three sites: the download path, the per-podcast settings-reapply path, and a new `regenerate_nfo` rescan option. Two new settings columns (`nfo_format`, `cover_filename`) are threaded through the existing global + per-podcast settings stack. Numbering logic shared by the MP3/MP4 tag writers is extracted into one pure helper.

**Tech Stack:** Rust, Diesel (sqlite + postgres via MultiBackend), Axum, `quick_xml` (already a dep), React/TypeScript + `openapi-fetch` (`schema.d.ts` is hand-edited).

**Spec:** `docs/superpowers/specs/2026-06-06-download-options-nfo-design.md`

**Conventions (from repo memory — MUST follow):**
- Repo methods in `podfetch-persistence` return `PersistenceError`, not `CustomError`.
- DB tests in `podfetch-persistence` MUST serialize via the crate-wide `crate::db::test_db::setup()` lock (runs migrations + holds a process-wide mutex). Never add a module-local lock.
- **No `cargo fmt`** (there is no fmt gate; running it churns unrelated files).
- `schema.d.ts` (ui + mobile) is hand-edited — keep keys ordered to match the surrounding file.
- Migrations target **sqlite + postgres only** (MySQL is not maintained). Simple `ADD COLUMN`/`DROP COLUMN` need no `metadata.toml` (that's only for table-rebuild migrations).
- Per-crate `-p` builds and clippy need `--features sqlite` (or `--features postgresql`).

---

## Task 1: Data layer — `nfo_format` + `cover_filename` columns end-to-end

Adds two columns to `settings` and `podcast_settings`, threaded through migrations, the Diesel schema, persistence entities, domain structs, and web DTOs. These are coupled by exhaustive `From` impls, so they land together to reach a compiling state.

**Files:**
- Create: `migrations/sqlite/2026-06-06-130000_nfo_options/up.sql`
- Create: `migrations/sqlite/2026-06-06-130000_nfo_options/down.sql`
- Create: `migrations/postgres/2026-06-06-130000_nfo_options/up.sql`
- Create: `migrations/postgres/2026-06-06-130000_nfo_options/down.sql`
- Modify: `crates/podfetch-persistence/src/settings.rs`
- Modify: `crates/podfetch-persistence/src/podcast_settings.rs`
- Modify: `crates/podfetch-domain/src/settings.rs`
- Modify: `crates/podfetch-domain/src/podcast_settings.rs`
- Modify: `crates/podfetch-web/src/settings.rs`
- Modify: `crates/podfetch-web/src/podcast_settings.rs`

- [ ] **Step 1: Write the SQLite migration**

`migrations/sqlite/2026-06-06-130000_nfo_options/up.sql`:
```sql
ALTER TABLE settings ADD COLUMN nfo_format TEXT NOT NULL DEFAULT 'off';
ALTER TABLE settings ADD COLUMN cover_filename TEXT NOT NULL DEFAULT 'image';
ALTER TABLE podcast_settings ADD COLUMN nfo_format TEXT NOT NULL DEFAULT 'off';
ALTER TABLE podcast_settings ADD COLUMN cover_filename TEXT NOT NULL DEFAULT 'image';
```

`migrations/sqlite/2026-06-06-130000_nfo_options/down.sql`:
```sql
ALTER TABLE settings DROP COLUMN nfo_format;
ALTER TABLE settings DROP COLUMN cover_filename;
ALTER TABLE podcast_settings DROP COLUMN nfo_format;
ALTER TABLE podcast_settings DROP COLUMN cover_filename;
```

- [ ] **Step 2: Write the Postgres migration** (identical SQL)

Create `migrations/postgres/2026-06-06-130000_nfo_options/up.sql` and `down.sql` with the **same** four `ADD COLUMN` / `DROP COLUMN` statements as Step 1.

- [ ] **Step 3: Add columns to the `settings` Diesel schema + entity** (`crates/podfetch-persistence/src/settings.rs`)

In the `diesel::table! { settings ... }` block, append after `sponsorblock_enabled -> Bool,`:
```rust
        nfo_format -> Text,
        cover_filename -> Text,
```
In `struct SettingEntity`, append after `sponsorblock_enabled: bool,`:
```rust
    nfo_format: String,
    cover_filename: String,
```
In `impl From<SettingEntity> for Setting`, append after `sponsorblock_enabled: value.sponsorblock_enabled,`:
```rust
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
```
In `impl From<Setting> for SettingEntity`, append after `sponsorblock_enabled: value.sponsorblock_enabled,`:
```rust
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
```
In `insert_default_settings`, append to the `.values((...))` tuple after `sponsorblock_enabled.eq(true),`:
```rust
                nfo_format.eq("off"),
                cover_filename.eq("image"),
```

- [ ] **Step 4: Add columns to the `podcast_settings` Diesel schema + entity** (`crates/podfetch-persistence/src/podcast_settings.rs`)

In the `diesel::table! { podcast_settings ... }` block, append after `use_one_cover_for_all_episodes -> Bool,`:
```rust
        nfo_format -> Text,
        cover_filename -> Text,
```
In `struct PodcastSettingEntity`, append after `use_one_cover_for_all_episodes: bool,`:
```rust
    nfo_format: String,
    cover_filename: String,
```
In **both** `From` impls (`From<PodcastSettingEntity> for PodcastSetting` and `From<PodcastSetting> for PodcastSettingEntity`), append after `use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,`:
```rust
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
```

- [ ] **Step 5: Add fields to the domain structs**

`crates/podfetch-domain/src/settings.rs` — in `struct Setting`, append after `pub sponsorblock_enabled: bool,`:
```rust
    /// Jellyfin/Kodi NFO format: "off" | "tvshow" | "album".
    pub nfo_format: String,
    /// Base name of the podcast cover file (default "image").
    pub cover_filename: String,
```
`crates/podfetch-domain/src/podcast_settings.rs` — in `struct PodcastSetting`, append after `pub use_one_cover_for_all_episodes: bool,`:
```rust
    pub nfo_format: String,
    pub cover_filename: String,
```

- [ ] **Step 6: Add fields to the web DTOs with serde defaults**

`crates/podfetch-web/src/settings.rs` — in `struct Setting`, append after the `sponsorblock_enabled` field:
```rust
    /// Defaulted on deserialize so older clients that omit it keep working.
    #[serde(default = "default_nfo_format")]
    pub nfo_format: String,
    /// Defaulted on deserialize so older clients that omit it keep working.
    #[serde(default = "default_cover_filename")]
    pub cover_filename: String,
```
Add the default fns next to `default_sponsorblock_enabled`:
```rust
fn default_nfo_format() -> String {
    "off".to_string()
}

fn default_cover_filename() -> String {
    "image".to_string()
}
```
In **both** `From` impls in this file (`From<domain::Setting> for Setting` and `From<Setting> for domain::Setting`), append after the `sponsorblock_enabled: value.sponsorblock_enabled,` line:
```rust
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
```

`crates/podfetch-web/src/podcast_settings.rs` — in `struct PodcastSetting`, append after `pub use_one_cover_for_all_episodes: bool,`:
```rust
    #[serde(default = "crate::settings::default_nfo_format")]
    pub nfo_format: String,
    #[serde(default = "crate::settings::default_cover_filename")]
    pub cover_filename: String,
```
Make the two default fns in `crates/podfetch-web/src/settings.rs` `pub(crate)` (change `fn default_nfo_format` → `pub(crate) fn default_nfo_format`, same for `default_cover_filename`) so `podcast_settings.rs` can reference them.
In **both** `From` impls in `podcast_settings.rs`, append after `use_one_cover_for_all_episodes: value.use_one_cover_for_all_episodes,`:
```rust
            nfo_format: value.nfo_format,
            cover_filename: value.cover_filename,
```

- [ ] **Step 7: Write the persistence round-trip test** (`crates/podfetch-persistence/src/settings.rs`)

Append at the end of the file:
```rust
#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::db::{database, test_db::setup};

    #[test]
    fn nfo_format_and_cover_filename_round_trip() {
        let _guard = setup();
        let repo = DieselSettingsRepository::new(database());
        if repo.get_settings().expect("get").is_none() {
            repo.insert_default_settings().expect("insert default");
        }

        let mut s = repo.get_settings().expect("get").expect("present");
        s.nfo_format = "album".to_string();
        s.cover_filename = "cover".to_string();
        let updated = repo.update_settings(s).expect("update");
        assert_eq!(updated.nfo_format, "album");
        assert_eq!(updated.cover_filename, "cover");

        let reread = repo.get_settings().expect("get").expect("present");
        assert_eq!(reread.nfo_format, "album");
        assert_eq!(reread.cover_filename, "cover");
    }
}
```

- [ ] **Step 8: Build to verify migrations + schema compile, then run the test**

Run: `cargo build -p podfetch-persistence --features sqlite`
Expected: PASS (the embedded migrations are validated against the schema at build time).

Run: `cargo test -p podfetch-persistence --features sqlite nfo_format_and_cover_filename_round_trip -- --nocapture`
Expected: PASS.

Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS (proves every `From` impl across domain/persistence/web maps the new fields). If the compiler reports a missing field at another struct-literal/`From` site, add `nfo_format`/`cover_filename` there with `"off"`/`"image"` (or pass-through).

- [ ] **Step 9: Commit**

```bash
git add migrations crates/podfetch-persistence/src/settings.rs crates/podfetch-persistence/src/podcast_settings.rs crates/podfetch-domain/src/settings.rs crates/podfetch-domain/src/podcast_settings.rs crates/podfetch-web/src/settings.rs crates/podfetch-web/src/podcast_settings.rs
git commit -m "feat(download-options): add nfo_format + cover_filename settings columns (#315)"
```

---

## Task 2: `NfoFormat` enum + module skeleton

**Files:**
- Create: `crates/podfetch-web/src/services/nfo/mod.rs`
- Modify: `crates/podfetch-web/src/services/mod.rs` (register `pub mod nfo;`)

- [ ] **Step 1: Write the failing test** in `crates/podfetch-web/src/services/nfo/mod.rs`

```rust
use std::str::FromStr;

pub mod builders;
pub mod service;

/// Which NFO layout to emit for a podcast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NfoFormat {
    #[default]
    Off,
    Tvshow,
    Album,
}

impl FromStr for NfoFormat {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "tvshow" => Ok(NfoFormat::Tvshow),
            "album" => Ok(NfoFormat::Album),
            // "off", "", and any unknown value disable NFO generation.
            _ => Ok(NfoFormat::Off),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_and_unknown_values() {
        assert_eq!(NfoFormat::from_str("tvshow"), Ok(NfoFormat::Tvshow));
        assert_eq!(NfoFormat::from_str("album"), Ok(NfoFormat::Album));
        assert_eq!(NfoFormat::from_str("off"), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::from_str(""), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::from_str("garbage"), Ok(NfoFormat::Off));
        assert_eq!(NfoFormat::default(), NfoFormat::Off);
    }
}
```

- [ ] **Step 2: Register the module** in `crates/podfetch-web/src/services/mod.rs`

Add (in alphabetical position among the existing `pub mod` lines):
```rust
pub mod nfo;
```

> Note: `builders.rs` and `service.rs` are created in Tasks 3 and 4. Until then this won't compile, so create empty stub files now to keep the tree building:
> - `crates/podfetch-web/src/services/nfo/builders.rs` containing only a doc comment line `//! NFO XML builders (filled in Task 3).`
> - `crates/podfetch-web/src/services/nfo/service.rs` containing only `//! NFO writer service (filled in Task 4).`

- [ ] **Step 3: Run the test**

Run: `cargo test -p podfetch-web --features sqlite nfo::tests::parses_known_and_unknown_values`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/nfo crates/podfetch-web/src/services/mod.rs
git commit -m "feat(nfo): add NfoFormat enum + module skeleton (#315)"
```

---

## Task 3: Pure NFO builders

**Files:**
- Modify: `crates/podfetch-web/src/services/nfo/builders.rs`

- [ ] **Step 1: Write the builders + helpers**

Replace the stub content of `crates/podfetch-web/src/services/nfo/builders.rs` with:
```rust
//! Pure NFO XML builders. Domain types in, XML string out. No DB, no FS.

use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Cursor;

type XmlWriter = Writer<Cursor<Vec<u8>>>;

fn new_writer() -> XmlWriter {
    Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)
}

fn finish(writer: XmlWriter) -> String {
    String::from_utf8(writer.into_inner().into_inner()).expect("xml is valid utf-8")
}

fn write_decl(w: &mut XmlWriter) {
    w.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), Some("yes"))))
        .expect("write decl");
}

/// Write `<name>text</name>`. No-op when `text` is None or empty. `quick_xml`
/// escapes the text content (`&`, `<`, `>`, quotes) automatically.
fn write_text_el(w: &mut XmlWriter, name: &str, text: Option<&str>) {
    let Some(text) = text.filter(|t| !t.is_empty()) else {
        return;
    };
    w.write_event(Event::Start(BytesStart::new(name)))
        .expect("start element");
    w.write_event(Event::Text(BytesText::new(text)))
        .expect("text");
    w.write_event(Event::End(BytesEnd::new(name)))
        .expect("end element");
}

/// Write `<uniqueid type="podfetch">id</uniqueid>` when `id` is non-empty.
fn write_uniqueid(w: &mut XmlWriter, id: Option<&str>) {
    let Some(id) = id.filter(|s| !s.is_empty()) else {
        return;
    };
    let mut el = BytesStart::new("uniqueid");
    el.push_attribute(("type", "podfetch"));
    w.write_event(Event::Start(el)).expect("start uniqueid");
    w.write_event(Event::Text(BytesText::new(id)))
        .expect("uniqueid text");
    w.write_event(Event::End(BytesEnd::new("uniqueid")))
        .expect("end uniqueid");
}

/// Kodi `<runtime>` is expressed in whole minutes.
fn runtime_minutes(total_time_secs: i32) -> i64 {
    ((total_time_secs.max(0) as f64) / 60.0).round() as i64
}

/// `date_of_recording` is typically ISO-8601 ("2023-09-07T13:09:00"). Take the
/// `YYYY-MM-DD` prefix (dates are ASCII so byte slicing is char-safe).
fn aired_date(date_of_recording: &str) -> Option<String> {
    date_of_recording.get(..10).map(str::to_string)
}

/// `tvshow.nfo` at the podcast root.
pub fn build_tvshow_nfo(podcast: &Podcast) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("tvshow")))
        .expect("start tvshow");
    write_text_el(&mut w, "title", Some(&podcast.name));
    write_text_el(&mut w, "plot", podcast.summary.as_deref());
    write_text_el(&mut w, "studio", podcast.author.as_deref());
    write_text_el(&mut w, "genre", podcast.keywords.as_deref());
    write_uniqueid(&mut w, podcast.guid.as_deref());
    w.write_event(Event::End(BytesEnd::new("tvshow")))
        .expect("end tvshow");
    finish(w)
}

/// Per-episode `<basename>.nfo`.
pub fn build_episodedetails_nfo(
    podcast: &Podcast,
    episode: &PodcastEpisode,
    position: i64,
) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("episodedetails")))
        .expect("start episodedetails");
    write_text_el(&mut w, "title", Some(&episode.name));
    write_text_el(&mut w, "showtitle", Some(&podcast.name));
    write_text_el(&mut w, "season", Some("1"));
    write_text_el(&mut w, "episode", Some(&position.to_string()));
    write_text_el(&mut w, "plot", Some(&episode.description));
    write_text_el(&mut w, "aired", aired_date(&episode.date_of_recording).as_deref());
    write_text_el(&mut w, "runtime", Some(&runtime_minutes(episode.total_time).to_string()));
    if let Some(author) = podcast.author.as_deref().filter(|a| !a.is_empty()) {
        w.write_event(Event::Start(BytesStart::new("actor")))
            .expect("start actor");
        write_text_el(&mut w, "name", Some(author));
        w.write_event(Event::End(BytesEnd::new("actor")))
            .expect("end actor");
    }
    write_uniqueid(&mut w, Some(&episode.guid));
    w.write_event(Event::End(BytesEnd::new("episodedetails")))
        .expect("end episodedetails");
    finish(w)
}

/// Single `album.nfo` at the podcast root. `tracks` are `(episode, position)`
/// ordered by date.
pub fn build_album_nfo(podcast: &Podcast, tracks: &[(PodcastEpisode, i64)]) -> String {
    let mut w = new_writer();
    write_decl(&mut w);
    w.write_event(Event::Start(BytesStart::new("album")))
        .expect("start album");
    write_text_el(&mut w, "title", Some(&podcast.name));
    write_text_el(&mut w, "artist", podcast.author.as_deref());
    write_text_el(&mut w, "genre", podcast.keywords.as_deref());
    write_text_el(&mut w, "review", podcast.summary.as_deref());
    for (episode, position) in tracks {
        w.write_event(Event::Start(BytesStart::new("track")))
            .expect("start track");
        write_text_el(&mut w, "position", Some(&position.to_string()));
        write_text_el(&mut w, "title", Some(&episode.name));
        write_text_el(&mut w, "duration", Some(&episode.total_time.to_string()));
        w.write_event(Event::End(BytesEnd::new("track")))
            .expect("end track");
    }
    w.write_event(Event::End(BytesEnd::new("album")))
        .expect("end album");
    finish(w)
}
```

- [ ] **Step 2: Write the tests** (append to `builders.rs`)

These assert structure/escaping/omission rather than exact whitespace (indentation is a `quick_xml` impl detail), so they're deterministic.
```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn podcast() -> Podcast {
        Podcast {
            name: "My Podcast".to_string(),
            summary: Some("A show about things & stuff".to_string()),
            author: Some("Jane Host".to_string()),
            keywords: Some("tech, news".to_string()),
            guid: Some("podcast-guid-1".to_string()),
            ..Default::default()
        }
    }

    fn episode() -> PodcastEpisode {
        PodcastEpisode {
            name: "Episode <One>".to_string(),
            description: "Plot & details".to_string(),
            date_of_recording: "2023-09-07T13:09:00".to_string(),
            total_time: 2520, // 42 minutes
            guid: "episode-guid-1".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn tvshow_has_expected_fields_and_escapes() {
        let xml = build_tvshow_nfo(&podcast());
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>"));
        assert!(xml.contains("<title>My Podcast</title>"));
        assert!(xml.contains("<plot>A show about things &amp; stuff</plot>"));
        assert!(xml.contains("<studio>Jane Host</studio>"));
        assert!(xml.contains("<genre>tech, news</genre>"));
        assert!(xml.contains("<uniqueid type=\"podfetch\">podcast-guid-1</uniqueid>"));
    }

    #[test]
    fn tvshow_omits_absent_optional_fields() {
        let p = Podcast {
            name: "Bare".to_string(),
            ..Default::default()
        };
        let xml = build_tvshow_nfo(&p);
        assert!(xml.contains("<title>Bare</title>"));
        assert!(!xml.contains("<studio>"));
        assert!(!xml.contains("<plot>"));
        assert!(!xml.contains("<genre>"));
        assert!(!xml.contains("<uniqueid"));
    }

    #[test]
    fn episodedetails_maps_runtime_minutes_position_and_actor() {
        let xml = build_episodedetails_nfo(&podcast(), &episode(), 7);
        assert!(xml.contains("<title>Episode &lt;One&gt;</title>"));
        assert!(xml.contains("<showtitle>My Podcast</showtitle>"));
        assert!(xml.contains("<season>1</season>"));
        assert!(xml.contains("<episode>7</episode>"));
        assert!(xml.contains("<plot>Plot &amp; details</plot>"));
        assert!(xml.contains("<aired>2023-09-07</aired>"));
        assert!(xml.contains("<runtime>42</runtime>"));
        assert!(xml.contains("<actor>"));
        assert!(xml.contains("<name>Jane Host</name>"));
        assert!(xml.contains("<uniqueid type=\"podfetch\">episode-guid-1</uniqueid>"));
    }

    #[test]
    fn episodedetails_omits_actor_without_author() {
        let p = Podcast {
            name: "P".to_string(),
            ..Default::default()
        };
        let xml = build_episodedetails_nfo(&p, &episode(), 1);
        assert!(!xml.contains("<actor>"));
    }

    #[test]
    fn album_lists_tracks_in_order() {
        let mut e2 = episode();
        e2.name = "Episode Two".to_string();
        e2.total_time = 60;
        let tracks = vec![(episode(), 1), (e2, 2)];
        let xml = build_album_nfo(&podcast(), &tracks);
        assert!(xml.contains("<artist>Jane Host</artist>"));
        assert!(xml.contains("<review>A show about things &amp; stuff</review>"));
        let p1 = xml.find("<position>1</position>").expect("track 1");
        let p2 = xml.find("<position>2</position>").expect("track 2");
        assert!(p1 < p2, "tracks must be ordered");
        assert!(xml.contains("<duration>2520</duration>"));
        assert!(xml.contains("<duration>60</duration>"));
    }
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p podfetch-web --features sqlite nfo::builders`
Expected: PASS (all 5 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/nfo/builders.rs
git commit -m "feat(nfo): pure tvshow/episodedetails/album XML builders (#315)"
```

---

## Task 4: NFO writer service (resolution, path derivation, file writing)

**Files:**
- Modify: `crates/podfetch-web/src/services/nfo/service.rs`

- [ ] **Step 1: Write the service**

Replace the stub content of `crates/podfetch-web/src/services/nfo/service.rs` with:
```rust
//! Resolves the NFO format from settings and writes NFO files. All writes are
//! best-effort: failures are logged, never propagated (must not fail a
//! download or a settings save).

use super::NfoFormat;
use super::builders;
use crate::services::file::service::FileService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast::Podcast;
use podfetch_domain::podcast_episode::PodcastEpisode;
use podfetch_persistence::podcast::PodcastEntity;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity;
use podfetch_storage::{FileHandleWrapper, FileRequest};
use std::str::FromStr;
use uuid::Uuid;

const COVER_CANDIDATES: &[&str] = &["image", "cover", "folder", "poster"];

/// Resolve the effective NFO format: per-podcast override (when activated)
/// falls back to the global setting.
pub fn resolve_nfo_format(podcast_id: Uuid) -> NfoFormat {
    let global = SettingsService::shared()
        .get_settings()
        .ok()
        .flatten()
        .map(|s| s.nfo_format)
        .unwrap_or_default();
    let raw = match PodcastSettingsService::get_settings_for_podcast(podcast_id) {
        Ok(Some(ps)) if ps.activated => ps.nfo_format,
        _ => global,
    };
    NfoFormat::from_str(&raw).unwrap_or_default()
}

/// Resolve the effective cover base name (empty → "image").
pub fn resolve_cover_filename(podcast_id: Uuid) -> String {
    let global = SettingsService::shared()
        .get_settings()
        .ok()
        .flatten()
        .map(|s| s.cover_filename)
        .unwrap_or_default();
    let raw = match PodcastSettingsService::get_settings_for_podcast(podcast_id) {
        Ok(Some(ps)) if ps.activated => ps.cover_filename,
        _ => global,
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "image".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Derive the sidecar `.nfo` path from a media file path by swapping its
/// extension. If the path has no extension, append `.nfo`.
pub fn nfo_path_for(audio_path: &str) -> String {
    let last_sep = audio_path.rfind(['/', '\\']);
    match audio_path.rfind('.') {
        Some(dot) if last_sep.map_or(true, |sep| dot > sep) => {
            format!("{}.nfo", &audio_path[..dot])
        }
        _ => format!("{audio_path}.nfo"),
    }
}

fn write_nfo_file(path: &str, xml: &str) {
    let mut bytes = xml.as_bytes().to_vec();
    if let Err(err) = FileHandleWrapper::write_file(
        path,
        bytes.as_mut_slice(),
        &ENVIRONMENT_SERVICE.default_file_handler,
    ) {
        tracing::warn!("Failed to write NFO file {path}: {err}");
    }
}

/// Generate NFO for one episode (and refresh the podcast-level NFO). Non-fatal.
/// `audio_path` is the FINAL media path (post-transcode), so the per-episode
/// `.nfo` basename matches the audio file.
pub fn regenerate_for_episode(
    podcast_entity: &PodcastEntity,
    episode_entity: &PodcastEpisodeEntity,
    audio_path: &str,
) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    match resolve_nfo_format(podcast_uuid) {
        NfoFormat::Off => {}
        NfoFormat::Tvshow => {
            let podcast = Podcast::from(podcast_entity.clone());
            let episode = PodcastEpisode::from(episode_entity.clone());
            let position = PodcastEpisodeService::get_position_of_episode(
                &episode.date_of_recording,
                podcast_uuid,
            )
            .unwrap_or(0) as i64;
            write_nfo_file(
                &nfo_path_for(audio_path),
                &builders::build_episodedetails_nfo(&podcast, &episode, position),
            );
            write_nfo_file(
                &format!("{}/tvshow.nfo", podcast_entity.directory_name),
                &builders::build_tvshow_nfo(&podcast),
            );
        }
        NfoFormat::Album => write_album_nfo(podcast_entity),
    }
}

/// Rewrite `album.nfo` from the podcast's full downloaded-episode set. Non-fatal.
fn write_album_nfo(podcast_entity: &PodcastEntity) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    let podcast = Podcast::from(podcast_entity.clone());
    let mut episodes = match PodcastEpisodeService::get_episodes_by_podcast_id(podcast_uuid) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!("album.nfo: could not load episodes: {err}");
            return;
        }
    };
    episodes.retain(|e| e.download_time.is_some());
    episodes.sort_by(|a, b| a.date_of_recording.cmp(&b.date_of_recording));
    let tracks: Vec<(PodcastEpisode, i64)> = episodes
        .into_iter()
        .enumerate()
        .map(|(i, e)| (PodcastEpisode::from(e), (i as i64) + 1))
        .collect();
    write_nfo_file(
        &format!("{}/album.nfo", podcast_entity.directory_name),
        &builders::build_album_nfo(&podcast, &tracks),
    );
}

/// Rename the on-disk podcast cover to the configured base name when needed.
/// Best-effort, Local backend only (rename is unsupported on S3).
pub fn ensure_cover_filename(podcast_entity: &PodcastEntity) {
    let Ok(podcast_uuid) = Uuid::parse_str(&podcast_entity.id) else {
        return;
    };
    let target = resolve_cover_filename(podcast_uuid);
    let dir = &podcast_entity.directory_name;
    if dir.is_empty() {
        return;
    }
    let Some(ext) = std::path::Path::new(&podcast_entity.image_url)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
    else {
        return;
    };

    let fh = &ENVIRONMENT_SERVICE.default_file_handler;
    let target_path = format!("{dir}/{target}.{ext}");
    if FileHandleWrapper::path_exists(&target_path, FileRequest::File, fh) {
        return; // already correct
    }
    for cand in COVER_CANDIDATES {
        if *cand == target {
            continue;
        }
        let cand_path = format!("{dir}/{cand}.{ext}");
        if !FileHandleWrapper::path_exists(&cand_path, FileRequest::File, fh) {
            continue;
        }
        match FileHandleWrapper::rename_file(&cand_path, &target_path, fh) {
            Ok(()) => {
                tracing::info!("Renamed cover {cand_path} -> {target_path}");
                let new_url = format!(
                    "{}/{}.{}",
                    PodcastEpisodeService::map_to_local_url(dir),
                    target,
                    ext
                );
                if let Err(err) =
                    PodcastEpisodeService::update_podcast_image(&podcast_entity.id, &new_url)
                {
                    tracing::warn!("Cover renamed but DB image_url update failed: {err}");
                }
            }
            Err(err) => tracing::warn!("Cover rename {cand_path} -> {target_path} failed: {err}"),
        }
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::nfo_path_for;

    #[test]
    fn nfo_path_swaps_extension_or_appends() {
        assert_eq!(nfo_path_for("podcasts/x/audio.mp3"), "podcasts/x/audio.nfo");
        assert_eq!(
            nfo_path_for("podcasts/My Show/2023-09-07 - Ep.mp3"),
            "podcasts/My Show/2023-09-07 - Ep.nfo"
        );
        // dot only in a directory component → treat as no extension
        assert_eq!(
            nfo_path_for("podcasts/My.Show/episode"),
            "podcasts/My.Show/episode.nfo"
        );
        // windows separators
        assert_eq!(nfo_path_for("podcasts\\x\\audio.opus"), "podcasts\\x\\audio.nfo");
    }
}
```

> **Note on `FileService` import:** it is imported above only if a later step needs it; if `cargo` warns it is unused, remove the `use crate::services::file::service::FileService;` line. (It is listed defensively because the cover URL/local-path helpers live near there.) Prefer removing unused imports over `#[allow]`.

- [ ] **Step 2: Run the test**

Run: `cargo test -p podfetch-web --features sqlite nfo::service::tests::nfo_path_swaps_extension_or_appends`
Expected: PASS.

Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS. (Confirms `get_episodes_by_podcast_id`, `get_position_of_episode`, `map_to_local_url`, `update_podcast_image` signatures match. If any differ, adjust the call — these are existing `PodcastEpisodeUseCase` methods.)

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/services/nfo/service.rs
git commit -m "feat(nfo): writer service (resolve format, path derivation, cover rename) (#315)"
```

---

## Task 5: Configurable cover filename for new downloads

**Files:**
- Modify: `crates/podfetch-storage/src/path.rs`
- Modify: `crates/podfetch-web/src/services/file/service.rs`

- [ ] **Step 1: Update the failing test + signature** in `crates/podfetch-storage/src/path.rs`

Change `build_podcast_image_paths` to take a `cover_filename`:
```rust
pub fn build_podcast_image_paths(
    directory: &str,
    suffix: &str,
    cover_filename: &str,
    map_to_local_url: impl FnOnce(&str) -> String,
) -> (String, String) {
    let file_path = format!("{directory}/{cover_filename}.{suffix}");
    let url_path = format!("{}/{}.{}", map_to_local_url(directory), cover_filename, suffix);
    (file_path, url_path)
}
```
Update the existing test `builds_prefixed_podcast_image_paths` to pass the new arg and assert the configured name:
```rust
    #[test]
    fn builds_prefixed_podcast_image_paths() {
        let result = build_podcast_image_paths("podcasts/test", "png", "cover", |directory| {
            format!("/api/files/{directory}")
        });

        assert_eq!(
            result,
            (
                "podcasts/test/cover.png".to_string(),
                "/api/files/podcasts/test/cover.png".to_string()
            )
        );
    }
```

- [ ] **Step 2: Run the storage test (expect FAIL until callers compile, then PASS)**

Run: `cargo test -p podfetch-storage --features sqlite build_podcast_image_paths`
Expected: compile error in `podfetch-web` callers is fine at the crate level; run the storage crate test in isolation — it should PASS (storage crate has no caller of the new signature besides its own test).

- [ ] **Step 3: Update the caller** in `crates/podfetch-web/src/services/file/service.rs`

In `download_podcast_image`, resolve the cover name from settings and pass it. Replace the `build_podcast_image_paths(...)` call (currently around line 135) with:
```rust
        let cover_filename = Uuid::parse_str(podcast_id)
            .map(crate::services::nfo::service::resolve_cover_filename)
            .unwrap_or_else(|_| "image".to_string());
        let file_path = build_podcast_image_paths(
            podcast_path,
            &image_suffix.0,
            &cover_filename,
            PodcastEpisodeService::map_to_local_url,
        );
```
If `uuid::Uuid` is not already imported in this file, add `use uuid::Uuid;` at the top.

- [ ] **Step 4: Find and update any other callers**

Run: `rg -n "build_podcast_image_paths\(" crates --glob '!**/path.rs'`
For each match, insert the cover-name argument (resolve via `crate::services::nfo::service::resolve_cover_filename(podcast_uuid)` where a podcast UUID is in scope, else `"image"`). Show the change matches the Step 3 shape.

- [ ] **Step 5: Build + test**

Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS.
Run: `cargo test -p podfetch-storage --features sqlite build_podcast_image_paths`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-storage/src/path.rs crates/podfetch-web/src/services/file/service.rs
git commit -m "feat(download-options): configurable cover filename for new downloads (#315)"
```

---

## Task 6: `{episodeNumber}` filename token

**Files:**
- Modify: `crates/podfetch-web/src/services/file/service.rs`
- Modify: `crates/podfetch-web/src/controllers/podcast_episode_controller.rs`
- Modify: `crates/podfetch-web/src/services/settings/service.rs`

- [ ] **Step 1: Write a failing test** in `crates/podfetch-web/src/services/file/service.rs` (the existing `#[cfg(test)] mod tests`)

```rust
    #[test]
    fn episode_format_supports_episode_number_token() {
        let mut settings = test_settings(); // existing helper used by sibling tests
        settings.episode_format = "{episodeNumber:0>3} - {episodeTitle}".to_string();
        let mut episode = test_episode(); // existing helper
        episode.name = "Hello".to_string();
        let result =
            perform_episode_variable_replacement(settings, episode, None, 7).unwrap();
        assert_eq!(result, "007 - Hello");
    }
```
> If the sibling tests construct `Setting`/`PodcastEpisode` inline rather than via helpers, mirror their exact construction here instead of `test_settings()`/`test_episode()`.

- [ ] **Step 2: Add the parameter + token** to `perform_episode_variable_replacement`

Change the signature (line 340) to add a trailing param:
```rust
pub fn perform_episode_variable_replacement(
    retrieved_settings: Setting,
    podcast_episode: PodcastEpisode,
    podcast_settings: Option<PodcastSetting>,
    episode_number: i64,
) -> Result<String, CustomError> {
```
After `let total_time = podcast_episode.total_time.to_string();` add:
```rust
    let episode_number_str = episode_number.to_string();
```
After the `vars.insert("episodeDuration".to_string(), &total_time);` line add:
```rust
    vars.insert("episodeNumber".to_string(), &episode_number_str);
```
(No alias mapping is needed — users write `{episodeNumber}` directly, and `strfmt` format specs like `{episodeNumber:0>3}` pass through the existing `.replace(...)` chain unchanged.)

- [ ] **Step 3: Update the production call site** in `prepare_podcast_episode_title_to_directory` (same file, ~line 333)

Replace the body that parses the podcast id + calls the replacement with:
```rust
    let podcast_uuid = uuid::Uuid::parse_str(&podcast_episode.podcast_id)
        .map_err(|_| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;
    let podcast_settings = PodcastSettingsService::get_settings_for_podcast(podcast_uuid)?;
    let episode_number = crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_position_of_episode(
        &podcast_episode.date_of_recording,
        podcast_uuid,
    )? as i64;
    perform_episode_variable_replacement(
        retrieved_settings.into(),
        podcast_episode,
        podcast_settings,
        episode_number,
    )
```

- [ ] **Step 4: Update the remaining call sites** (preview/sample → position `1`)

In `crates/podfetch-web/src/controllers/podcast_episode_controller.rs:558`, change:
```rust
    let result = perform_episode_variable_replacement(settings.into(), episode, None)?;
```
to:
```rust
    let result = perform_episode_variable_replacement(settings.into(), episode, None, 1)?;
```
In `crates/podfetch-web/src/services/settings/service.rs:89`, change:
```rust
        perform_episode_variable_replacement(transient_setting, sample_episode, None)?;
```
to:
```rust
        perform_episode_variable_replacement(transient_setting, sample_episode, None, 1)?;
```
In `crates/podfetch-web/src/services/file/service.rs`, update the three test call sites (currently lines ~634, ~681, ~728) to pass a final `1` argument.

- [ ] **Step 5: Run tests + build**

Run: `cargo test -p podfetch-web --features sqlite file::service`
Expected: PASS (including the new `episode_format_supports_episode_number_token`).
Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/services/file/service.rs crates/podfetch-web/src/controllers/podcast_episode_controller.rs crates/podfetch-web/src/services/settings/service.rs
git commit -m "feat(download-options): add {episodeNumber} filename token (#315)"
```

---

## Task 7: Shared numbering helper + MP4 numbering fix

**Files:**
- Modify: `crates/podfetch-web/src/services/download/service.rs`

- [ ] **Step 1: Write a failing test** in the `#[cfg(test)] mod` of `download/service.rs`

```rust
    #[test]
    fn resolve_episode_title_handles_all_numbering_states() {
        // numbering on, not yet processed → prefix + mark processed
        assert_eq!(
            super::DownloadService::resolve_episode_title("Ep", false, true, 5),
            Some(("5 - Ep".to_string(), true))
        );
        // numbering on, already processed → leave as-is
        assert_eq!(
            super::DownloadService::resolve_episode_title("Ep", true, true, 5),
            None
        );
        // numbering off → plain title, mark not-processed
        assert_eq!(
            super::DownloadService::resolve_episode_title("Ep", false, false, 5),
            Some(("Ep".to_string(), false))
        );
        assert_eq!(
            super::DownloadService::resolve_episode_title("Ep", true, false, 5),
            Some(("Ep".to_string(), false))
        );
    }
```

- [ ] **Step 2: Add the pure helper** as an associated fn on `DownloadService` (in `impl DownloadService`)

```rust
    /// Decide the title to embed and whether to (re)write the
    /// `episode_numbering_processed` flag. Returns `None` when numbering is on
    /// and the episode was already processed (leave the existing title alone).
    pub(crate) fn resolve_episode_title(
        name: &str,
        episode_numbering_processed: bool,
        numbering_enabled: bool,
        index: usize,
    ) -> Option<(String, bool)> {
        if numbering_enabled {
            if episode_numbering_processed {
                None
            } else {
                Some((format!("{index} - {name}"), true))
            }
        } else {
            Some((name.to_string(), false))
        }
    }
```

- [ ] **Step 3: Refactor `update_meta_data_mp3` to use the helper**

Replace the numbering block (currently lines ~482-507, from `let settings_for_podcast =` through the trailing `else { ... }`) with:
```rust
        let settings_for_podcast =
            PodcastSettingsService::get_settings_for_podcast(parse_id(&podcast.id)?)?;
        let numbering_enabled = settings_for_podcast
            .as_ref()
            .map(|s| s.episode_numbering)
            .unwrap_or(false);
        if let Some((title, processed)) = Self::resolve_episode_title(
            &podcast_episode.name,
            podcast_episode.episode_numbering_processed,
            numbering_enabled,
            index,
        ) {
            tag.set_title(title);
            PodcastEpisodeService::update_episode_numbering_processed(
                processed,
                &podcast_episode.episode_id,
            )?;
        }
```
(`index` is already computed just above via `get_position_of_episode`.)

- [ ] **Step 4: Apply numbering in `update_meta_data_mp4` (the bug fix)**

In `update_meta_data_mp4`, replace `tag.set_title(&podcast_episode.name);` with:
```rust
                let index = PodcastEpisodeService::get_position_of_episode(
                    &podcast_episode.date_of_recording,
                    parse_id(&podcast_episode.podcast_id)?,
                )?;
                let settings_for_podcast =
                    PodcastSettingsService::get_settings_for_podcast(parse_id(&podcast.id)?)?;
                let numbering_enabled = settings_for_podcast
                    .as_ref()
                    .map(|s| s.episode_numbering)
                    .unwrap_or(false);
                if let Some((title, processed)) = Self::resolve_episode_title(
                    &podcast_episode.name,
                    podcast_episode.episode_numbering_processed,
                    numbering_enabled,
                    index,
                ) {
                    tag.set_title(&title);
                    PodcastEpisodeService::update_episode_numbering_processed(
                        processed,
                        &podcast_episode.episode_id,
                    )?;
                }
```

- [ ] **Step 5: Run tests + build**

Run: `cargo test -p podfetch-web --features sqlite resolve_episode_title_handles_all_numbering_states`
Expected: PASS.
Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/services/download/service.rs
git commit -m "fix(download-options): apply episode numbering to mp4 via shared helper (#315)"
```

---

## Task 8: Wire NFO into the download path

**Files:**
- Modify: `crates/podfetch-web/src/services/download/service.rs`

- [ ] **Step 1: Add the NFO call** in `download_podcast_episode`

Insert immediately **after** `final_episode_path` is computed (the `let final_episode_path = ...` block, ~line 339-349) and **before** `PodcastEpisodeService::update_local_paths(...)` (~line 351):
```rust
        // Jellyfin/Kodi NFO sidecar files (non-fatal). Uses the FINAL media
        // path so the per-episode .nfo basename matches the audio file even
        // after Opus transcoding.
        crate::services::nfo::service::regenerate_for_episode(
            podcast,
            &podcast_episode,
            &final_episode_path,
        );
```

- [ ] **Step 2: Build**

Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS. (`podcast` is `&PodcastEntity`, `podcast_episode` is `PodcastEpisodeEntity`, matching `regenerate_for_episode`.)

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/services/download/service.rs
git commit -m "feat(nfo): generate NFO on episode download (#315)"
```

---

## Task 9: Wire NFO + cover rename into the settings-reapply path

**Files:**
- Modify: `crates/podfetch-web/src/services/podcast_settings/service.rs`

- [ ] **Step 1: Regenerate NFO per episode inside the reapply loop**

In `update_settings`, inside the `for episode in available_episodes` loop, after the `match DownloadService::handle_metadata_insertion(...) { ... }` block and before the loop ends, add:
```rust
            if let Some(audio_path) = episode
                .file_episode_path
                .as_deref()
                .filter(|p| !p.is_empty())
            {
                crate::services::nfo::service::regenerate_for_episode(&podcast, &episode, audio_path);
            }
```

- [ ] **Step 2: Rename the cover once after the loop**

After the `for episode in available_episodes { ... }` loop closes and before `Ok(updated_setting.into())`, add:
```rust
        crate::services::nfo::service::ensure_cover_filename(&podcast);
```

- [ ] **Step 3: Build**

Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS. (`podcast` and `episode` here are persistence entities, matching the NFO service.)

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/podcast_settings/service.rs
git commit -m "feat(nfo): regenerate NFO + rename cover when podcast settings change (#315)"
```

---

## Task 10: Backfill rescan option (`regenerateNfo`)

**Files:**
- Modify: `crates/podfetch-web/src/services/episode_rescan/service.rs`
- Modify: `ui/schema.d.ts` (RescanOptions)
- Modify: `mobile/schema.d.ts` (RescanOptions)

- [ ] **Step 1: Add the option field + toggle**

In `struct RescanOptions`, append after `pub refetch_sponsorblock: bool,`:
```rust
    /// (Re)write NFO files for the episode (and rename the cover) using the
    /// current settings, without re-downloading audio.
    pub regenerate_nfo: bool,
```
In `impl RescanOptions::any_enabled`, add to the boolean chain:
```rust
            || self.regenerate_nfo
```

- [ ] **Step 2: Act on the option inside the per-episode loop**

Near the existing `if opts.refetch_sponsorblock { ... }` block (~line 347), add:
```rust
        if opts.regenerate_nfo {
            crate::services::nfo::service::regenerate_for_episode(
                &podcast,
                episode,
                &final_audio_path,
            );
            crate::services::nfo::service::ensure_cover_filename(&podcast);
        }
```
> `episode`, `podcast`, and `final_audio_path` are the loop-local bindings already used by the cover-consolidation/sponsorblock blocks. If `episode` is a reference there, pass it as-is; if owned, pass `&episode`. Match the existing `refetch_sponsorblock` block's usage (it uses `&episode.clone()` / `episode` — mirror it).

- [ ] **Step 3: Add the unit assertion** to the existing rescan test module

In `rescan_options_any_enabled_reflects_individual_toggles`, add:
```rust
        assert!(
            RescanOptions {
                regenerate_nfo: true,
                ..RescanOptions::default()
            }
            .any_enabled()
        );
```

- [ ] **Step 4: Add `regenerateNfo` to `RescanOptions` in both schema files**

In `ui/schema.d.ts`, inside the `RescanOptions` object (the block near line 1824 that documents `apply_covers`), add after the existing `refetchSponsorblock` property (matching the surrounding `@default false` doc style):
```ts
            /**
             * @description (Re)write NFO files for the episode (and rename the cover)
             *     using the current settings, without re-downloading audio.
             * @default false
             */
            regenerateNfo?: boolean;
```
Apply the **same** addition to `mobile/schema.d.ts` in its `RescanOptions` block.

- [ ] **Step 5: Build + test**

Run: `cargo test -p podfetch-web --features sqlite rescan_options_any_enabled`
Expected: PASS.
Run: `cargo build -p podfetch-web --features sqlite`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/services/episode_rescan/service.rs ui/schema.d.ts mobile/schema.d.ts
git commit -m "feat(nfo): add regenerateNfo backfill rescan option (#315)"
```

---

## Task 11: Frontend — expose `nfoFormat` + `coverFilename`

Mirror the existing `useOneCoverForAllEpisodes` field everywhere it appears. Open each file and copy that field's wiring for the two new fields.

**Files:**
- Modify: `ui/schema.d.ts`
- Modify: `mobile/schema.d.ts`
- Modify: `ui/src/models/Setting.tsx`
- Modify: `ui/src/models/PodcastSetting.ts`
- Modify: `ui/src/models/PodcastDefaultSettings.tsx`
- Modify: `ui/src/components/SettingsNaming.tsx`
- Modify: `ui/src/components/PodcastSettingsModal.tsx`

- [ ] **Step 1: Add types to `ui/schema.d.ts`**

In the `Setting` schema object (the block where `useOneCoverForAllEpisodes: boolean;` appears, ~line 1798), add after it:
```ts
            nfoFormat: string;
            coverFilename: string;
```
Do the same in the per-podcast settings schema object (the second `useOneCoverForAllEpisodes` occurrence, ~line 1871).

- [ ] **Step 2: Add the same two lines to `mobile/schema.d.ts`**

Run: `rg -n "useOneCoverForAllEpisodes" mobile/schema.d.ts`
For each occurrence inside a settings object, add `nfoFormat: string;` and `coverFilename: string;` after it (same as Step 1).

- [ ] **Step 3: Update the TS models**

In `ui/src/models/Setting.tsx`, `ui/src/models/PodcastSetting.ts`, and `ui/src/models/PodcastDefaultSettings.tsx`: locate the `useOneCoverForAllEpisodes` field (type + any default/initializer) and add `nfoFormat` (string, default `"off"`) and `coverFilename` (string, default `"image"`) right beside it, following the exact shape of that field in each file.

- [ ] **Step 4: Add the form controls**

In `ui/src/components/SettingsNaming.tsx` (global) and `ui/src/components/PodcastSettingsModal.tsx` (per-podcast): near the `useOneCoverForAllEpisodes` control, add:
- A `<select>` bound to `nfoFormat` with options `off` / `tvshow` / `album` (labels e.g. "No NFO files" / "Jellyfin TV-show" / "Music album").
- A text `<input>` bound to `coverFilename` (placeholder `image`).

Wire both into the same change/submit handler the existing fields use (copy how `useOneCoverForAllEpisodes` is read and written in the component's state + payload).

- [ ] **Step 5: Type-check + build the frontend**

Run: `cd ui && npm run build`
Expected: PASS (tsc + vite). Fix any type errors surfaced by the new fields.

- [ ] **Step 6: Commit**

```bash
git add ui/schema.d.ts mobile/schema.d.ts ui/src/models/Setting.tsx ui/src/models/PodcastSetting.ts ui/src/models/PodcastDefaultSettings.tsx ui/src/components/SettingsNaming.tsx ui/src/components/PodcastSettingsModal.tsx
git commit -m "feat(download-options): NFO format + cover filename UI (#315)"
```

---

## Task 12: Full verification gate

**Files:** none (verification only).

- [ ] **Step 1: Clippy on all three feature configurations**

Run: `cargo clippy --all-targets --features sqlite -- -D warnings`
Run: `cargo clippy --all-targets --features postgresql -- -D warnings`
Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings on any. (Do NOT run `cargo fmt`.)

- [ ] **Step 2: Tests (sqlite locally; postgres runs in CI)**

Run: `cargo test --features sqlite`
Expected: PASS. (Postgres-backed tests run in the CI `Build postgres` job — they need a testcontainer DB not assumed locally.)

- [ ] **Step 3: Frontend**

Run: `cd ui && npm run build`
Expected: PASS.

- [ ] **Step 4: Final commit (only if Step 1-3 required fixes)**

```bash
git add -A
git commit -m "chore(download-options): clippy/test fixes for NFO feature (#315)"
```

- [ ] **Step 5: Push + open PR** (per finishing-a-development-branch, option 2)

```bash
git push -u origin feat/download-options-315
gh pr create --title "feat: NFO files, {episodeNumber} token & Jellyfin polish (#315)" --body "$(cat <<'EOF'
## Summary
- Configurable Jellyfin/Kodi NFO generation (`nfo_format` = off | tvshow | album), global + per-podcast.
- `{episodeNumber}` filename token (1-based date position; zero-pad via `{episodeNumber:0>3}`).
- Fix episode numbering for `.m4a`/`.mp4` (shared `resolve_episode_title` helper).
- Configurable podcast cover filename (`cover_filename`, default `image`); renamed on change.
- NFO written on download, regenerated on podcast settings change, and backfillable via the `regenerateNfo` rescan option.

Closes #315.

## Test Plan
- [ ] `cargo clippy --all-targets --features sqlite -- -D warnings` (and `--features postgresql`, and default)
- [ ] `cargo test --features sqlite`
- [ ] `cd ui && npm run build`
- [ ] Manual: set a podcast to `tvshow`, download an episode, confirm `tvshow.nfo` + `<episode>.nfo`; set to `album`, confirm `album.nfo`; set `cover_filename=cover`, run rescan with `regenerateNfo`, confirm `image.jpg`→`cover.jpg`.
EOF
)"
```

---

## Self-Review

**Spec coverage:**
- §2 data model (nfo_format, cover_filename, both tables, resolution rule, sqlite+postgres) → Task 1. ✓
- §3 NFO module (NfoFormat, builders, quick_xml, field mapping) → Tasks 2-3. ✓
- §3 dispatch/writer + non-fatal → Task 4. ✓
- §4 wiring: download → Task 8; reapply → Task 9; backfill rescan → Task 10. ✓
- §4 cover filename: new downloads → Task 5; rename-on-change → Task 4 (`ensure_cover_filename`) + Tasks 9/10. ✓
- §5 `{episodeNumber}` token → Task 6; MP4 numbering fix via shared helper → Task 7. ✓
- §6 tests: builders golden/structural (Task 3), `NfoFormat::from_str` (Task 2), `{episodeNumber}` (Task 6), `resolve_episode_title` (Task 7), `.nfo` path (Task 4), settings round-trip (Task 1); DTO serde defaults (Task 1, `#[serde(default)]`). ✓
- §6 verification gate → Task 12. ✓
- Frontend exposure (implied by "global + per-podcast" + schema.d.ts convention) → Task 11. ✓

**Placeholder scan:** No TBD/TODO. Every code step shows complete code or an exact transformation anchored to a named existing field/line. Frontend Task 11 references the concrete existing `useOneCoverForAllEpisodes` field as the copy template (not vague).

**Type consistency:** `resolve_nfo_format`/`resolve_cover_filename`/`regenerate_for_episode`/`ensure_cover_filename`/`nfo_path_for` (Task 4) are used with matching signatures in Tasks 5/8/9/10. `resolve_episode_title(name, processed, numbering_enabled, index)` defined and called identically in Task 7 (mp3 + mp4) and tested in Task 7. `perform_episode_variable_replacement(..., episode_number: i64)` defined in Task 6 and all call sites updated in the same task. Builders take domain `Podcast`/`PodcastEpisode`; call sites convert from entities via existing `From` impls.
