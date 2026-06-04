# SponsorBlock Integration — Backend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the server-side half of SponsorBlock support to PodFetch — fetch SponsorBlock segments at download time for YouTube-sourced episodes, store them non-destructively, expose them (plus per-user skip preferences) over the API, and allow re-fetching via rescan.

**Architecture:** Approach B from the design spec. A YouTube video ID is captured at feed-ingest into a new `podcast_episodes.youtube_video_id` column. After an episode downloads, a new `services/sponsorblock` service queries SponsorBlock's privacy-preserving (hash-prefix) API via the existing async reqwest client and stores all returned segments in a new `episode_sponsor_segments` table. Per-user category preferences live in a new `sponsorblock_user_settings` table. New Axum endpoints serve segments + prefs and read/write prefs. All failures are non-fatal to downloads.

**Tech Stack:** Rust, Axum, Diesel (SQLite + Postgres), utoipa, the `sha256` + `reqwest` + `rss` + `serde_json` workspace crates.

**Companion plans (separate, written after this one):** web client (skip + settings UI), mobile client (skip).

**Spec:** `docs/superpowers/specs/2026-06-04-sponsorblock-design.md`

---

## File Structure

**Created:**
- `migrations/sqlite/2026-06-05-120000_sponsorblock/{up,down}.sql`
- `migrations/postgres/2026-06-05-120000_sponsorblock/{up,down}.sql`
- `crates/podfetch-persistence/src/sponsorblock.rs` — Diesel entities + repository for segments and user settings.
- `crates/podfetch-web/src/services/sponsorblock/mod.rs`
- `crates/podfetch-web/src/services/sponsorblock/video_id.rs` — pure YouTube-ID extraction.
- `crates/podfetch-web/src/services/sponsorblock/client.rs` — SponsorBlock HTTP client + JSON parsing.
- `crates/podfetch-web/src/services/sponsorblock/service.rs` — `fetch_and_store` orchestration + duration guard.
- `crates/podfetch-web/src/controllers/sponsorblock_controller.rs` — API endpoints + DTOs.

**Modified:**
- `crates/podfetch-persistence/src/schema.rs` — new `table!` blocks + the new `youtube_video_id`/`sponsorblock_enabled` columns + joinable/allow_tables.
- `crates/podfetch-persistence/src/lib.rs` — register `pub mod sponsorblock;`.
- `crates/podfetch-persistence/src/podcast_episode.rs` — `youtube_video_id` on entity structs + conversions.
- `crates/podfetch-persistence/src/settings.rs` — `sponsorblock_enabled` on table!/entity/From/insert_default.
- `crates/podfetch-domain/src/podcast_episode.rs` — `youtube_video_id` on domain structs.
- `crates/podfetch-domain/src/settings.rs` — `sponsorblock_enabled` on `Setting`.
- `crates/podfetch-web/src/settings.rs` — `sponsorblock_enabled` on web DTO (+ serde default) + conversions; `refetchSponsorblock` is added to `RescanOptions` (which lives in the rescan service, see below).
- `crates/podfetch-web/src/services/settings/service.rs` — `sponsorblock_enabled` in `build_name_only_setting`.
- `crates/podfetch-web/src/services/mod.rs` — register `pub mod sponsorblock;`.
- `crates/podfetch-web/src/usecases/podcast_episode/mod.rs` — call video-ID extraction in `insert_podcast_episode`; add `youtube_video_id` to `NewPodcastEpisode` construction.
- `crates/podfetch-web/src/services/download/service.rs` — call `fetch_and_store` after metadata insertion.
- `crates/podfetch-web/src/services/episode_rescan/service.rs` — `refetch_sponsorblock` field on `RescanOptions` + handling in `apply_to_episode`/`any_enabled`.
- `crates/podfetch-web/src/controllers/mod.rs` — register `pub mod sponsorblock_controller;`.
- `crates/podfetch-web/src/startup.rs` — merge the new sponsorblock router into the private API.
- `ui/schema.d.ts` and `mobile/schema.d.ts` — hand-add the new paths + component schemas.

---

## Task 1: Database migration + Diesel schema

**Files:**
- Create: `migrations/sqlite/2026-06-05-120000_sponsorblock/up.sql`
- Create: `migrations/sqlite/2026-06-05-120000_sponsorblock/down.sql`
- Create: `migrations/postgres/2026-06-05-120000_sponsorblock/up.sql`
- Create: `migrations/postgres/2026-06-05-120000_sponsorblock/down.sql`
- Modify: `crates/podfetch-persistence/src/schema.rs`

> MySQL migrations are not maintained (the last several migrations — `uuid_primary_keys`, `settings_max_parallel_downloads`, `devices_base_url`, `episode_triages` — have no `migrations/mysql` variant). Do **not** create a mysql migration. The `2026-06-05-120000` timestamp sorts after the current latest (`2026-06-05-000000_episode_triages`), so it runs last.

- [ ] **Step 1: Write the SQLite `up.sql`**

Create `migrations/sqlite/2026-06-05-120000_sponsorblock/up.sql`:

```sql
ALTER TABLE podcast_episodes ADD COLUMN youtube_video_id TEXT;
ALTER TABLE settings ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT 1;

CREATE TABLE episode_sponsor_segments (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    uuid TEXT NOT NULL,
    category TEXT NOT NULL,
    action_type TEXT NOT NULL,
    start_ms BIGINT NOT NULL,
    end_ms BIGINT NOT NULL,
    votes INTEGER NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT 0,
    duration_mismatch BOOLEAN NOT NULL DEFAULT 0,
    fetched_at TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX uq_episode_sponsor_segments_episode_uuid
    ON episode_sponsor_segments (episode_id, uuid);

CREATE TABLE sponsorblock_user_settings (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    skip_sponsor BOOLEAN NOT NULL DEFAULT 1,
    skip_selfpromo BOOLEAN NOT NULL DEFAULT 1,
    skip_interaction BOOLEAN NOT NULL DEFAULT 0,
    skip_intro BOOLEAN NOT NULL DEFAULT 0,
    skip_outro BOOLEAN NOT NULL DEFAULT 0,
    skip_preview BOOLEAN NOT NULL DEFAULT 0,
    skip_filler BOOLEAN NOT NULL DEFAULT 0,
    skip_music_offtopic BOOLEAN NOT NULL DEFAULT 0
);
```

- [ ] **Step 2: Write the SQLite `down.sql`**

Create `migrations/sqlite/2026-06-05-120000_sponsorblock/down.sql`:

```sql
DROP TABLE sponsorblock_user_settings;
DROP TABLE episode_sponsor_segments;
ALTER TABLE settings DROP COLUMN sponsorblock_enabled;
ALTER TABLE podcast_episodes DROP COLUMN youtube_video_id;
```

- [ ] **Step 3: Write the Postgres `up.sql`**

Create `migrations/postgres/2026-06-05-120000_sponsorblock/up.sql`:

```sql
ALTER TABLE podcast_episodes ADD COLUMN youtube_video_id TEXT;
ALTER TABLE settings ADD COLUMN sponsorblock_enabled BOOLEAN NOT NULL DEFAULT true;

CREATE TABLE episode_sponsor_segments (
    id TEXT PRIMARY KEY NOT NULL,
    episode_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE,
    uuid TEXT NOT NULL,
    category TEXT NOT NULL,
    action_type TEXT NOT NULL,
    start_ms BIGINT NOT NULL,
    end_ms BIGINT NOT NULL,
    votes INTEGER NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT false,
    duration_mismatch BOOLEAN NOT NULL DEFAULT false,
    fetched_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE UNIQUE INDEX uq_episode_sponsor_segments_episode_uuid
    ON episode_sponsor_segments (episode_id, uuid);

CREATE TABLE sponsorblock_user_settings (
    user_id TEXT PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT true,
    skip_sponsor BOOLEAN NOT NULL DEFAULT true,
    skip_selfpromo BOOLEAN NOT NULL DEFAULT true,
    skip_interaction BOOLEAN NOT NULL DEFAULT false,
    skip_intro BOOLEAN NOT NULL DEFAULT false,
    skip_outro BOOLEAN NOT NULL DEFAULT false,
    skip_preview BOOLEAN NOT NULL DEFAULT false,
    skip_filler BOOLEAN NOT NULL DEFAULT false,
    skip_music_offtopic BOOLEAN NOT NULL DEFAULT false
);
```

- [ ] **Step 4: Write the Postgres `down.sql`**

Create `migrations/postgres/2026-06-05-120000_sponsorblock/down.sql` — identical content to the SQLite `down.sql` from Step 2.

- [ ] **Step 5: Update `schema.rs` — add columns to existing tables**

In `crates/podfetch-persistence/src/schema.rs`, in the `podcast_episodes (id)` block, add `youtube_video_id` as the last column (after `download_location -> Nullable<Text>,`):

```rust
        download_location -> Nullable<Text>,
        youtube_video_id -> Nullable<Text>,
    }
}
```

In the `settings (id)` block, add after `max_parallel_downloads -> Integer,`:

```rust
        max_parallel_downloads -> Integer,
        sponsorblock_enabled -> Bool,
    }
}
```

- [ ] **Step 6: Update `schema.rs` — add the two new `table!` blocks**

Add these two blocks to `crates/podfetch-persistence/src/schema.rs` (anywhere among the other `diesel::table!` blocks; alphabetical placement near `episodes` / `settings` is fine):

```rust
diesel::table! {
    episode_sponsor_segments (id) {
        id -> Text,
        episode_id -> Text,
        uuid -> Text,
        category -> Text,
        action_type -> Text,
        start_ms -> BigInt,
        end_ms -> BigInt,
        votes -> Integer,
        locked -> Bool,
        duration_mismatch -> Bool,
        fetched_at -> Timestamp,
    }
}

diesel::table! {
    sponsorblock_user_settings (user_id) {
        user_id -> Text,
        enabled -> Bool,
        skip_sponsor -> Bool,
        skip_selfpromo -> Bool,
        skip_interaction -> Bool,
        skip_intro -> Bool,
        skip_outro -> Bool,
        skip_preview -> Bool,
        skip_filler -> Bool,
        skip_music_offtopic -> Bool,
    }
}
```

Add the joinable + register both tables in `allow_tables_to_appear_in_same_query!`:

```rust
diesel::joinable!(episode_sponsor_segments -> podcast_episodes (episode_id));
```

and add `episode_sponsor_segments,` and `sponsorblock_user_settings,` to the `diesel::allow_tables_to_appear_in_same_query!(...)` list (keep it alphabetical-ish; placement doesn't affect correctness).

- [ ] **Step 7: Verify the persistence crate still compiles**

Run: `cargo build -p podfetch-persistence`
Expected: PASS. (The new `table!` blocks generate modules; nothing references them yet.)

- [ ] **Step 8: Commit**

```bash
git add migrations/sqlite/2026-06-05-120000_sponsorblock migrations/postgres/2026-06-05-120000_sponsorblock crates/podfetch-persistence/src/schema.rs
git commit -m "feat(sponsorblock): add migration + diesel schema for segments, user settings, episode video id"
```

---

## Task 2: Add `youtube_video_id` to the episode entity & domain

**Files:**
- Modify: `crates/podfetch-domain/src/podcast_episode.rs`
- Modify: `crates/podfetch-persistence/src/podcast_episode.rs`

- [ ] **Step 1: Add the field to the domain structs**

In `crates/podfetch-domain/src/podcast_episode.rs`, add `pub youtube_video_id: Option<String>,` as the last field of BOTH `PodcastEpisode` (after `download_location: Option<String>,`) and `NewPodcastEpisode` (after `guid: String,`):

```rust
    // ...in PodcastEpisode, after download_location:
    pub download_location: Option<String>,
    pub youtube_video_id: Option<String>,
}
```

```rust
    // ...in NewPodcastEpisode, after guid:
    pub guid: String,
    pub youtube_video_id: Option<String>,
}
```

- [ ] **Step 2: Add the field to the persistence entity structs**

In `crates/podfetch-persistence/src/podcast_episode.rs`:

`PodcastEpisodeEntity` (after `download_location: Option<String>,`):
```rust
    pub download_location: Option<String>,
    pub youtube_video_id: Option<String>,
}
```

`NewPodcastEpisodeEntity` (after `guid: String,`):
```rust
    guid: String,
    youtube_video_id: Option<String>,
}
```

- [ ] **Step 3: Update the three `From` conversions**

In `impl From<PodcastEpisodeEntity> for PodcastEpisode` add (after `download_location: entity.download_location,`):
```rust
            download_location: entity.download_location,
            youtube_video_id: entity.youtube_video_id,
        }
    }
}
```

In `impl From<NewPodcastEpisode> for NewPodcastEpisodeEntity` add (after `guid: episode.guid,`):
```rust
            guid: episode.guid,
            youtube_video_id: episode.youtube_video_id,
        }
    }
}
```

> If any OTHER constructor of `PodcastEpisode`/`NewPodcastEpisode` exists (e.g. `..Default::default()` is used in tests so they're unaffected; explicit literal constructions are not). Search the workspace for `NewPodcastEpisode {` and `PodcastEpisode {` and add `youtube_video_id: None,` to any **exhaustive** struct literal the compiler flags. Default derives already cover `PodcastEpisodeEntity::default()` and `PodcastEpisode::default()`.

- [ ] **Step 4: Compile to find every struct literal that needs the new field**

Run: `cargo build -p podfetch-persistence -p podfetch-domain`
Expected: FAIL initially if any exhaustive literal is missing the field; the compiler lists exact files/lines. Add `youtube_video_id: None,` to each until it PASSES.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-domain/src/podcast_episode.rs crates/podfetch-persistence/src/podcast_episode.rs
git commit -m "feat(sponsorblock): thread youtube_video_id through episode entity + domain"
```

---

## Task 3: Add the `sponsorblock_enabled` global setting

**Files:**
- Modify: `crates/podfetch-domain/src/settings.rs`
- Modify: `crates/podfetch-persistence/src/settings.rs`
- Modify: `crates/podfetch-web/src/settings.rs`
- Modify: `crates/podfetch-web/src/services/settings/service.rs`
- Test: `crates/podfetch-web/src/controllers/settings_controller.rs` (new `#[tokio::test]`)

> This mirrors exactly how `max_parallel_downloads` was added.

- [ ] **Step 1: Write the failing test (round-trip + serde default)**

In `crates/podfetch-web/src/controllers/settings_controller.rs`, inside the existing `#[cfg(test)] mod tests { ... }`, add (it mirrors `test_max_parallel_downloads_defaults_persists_and_tolerates_omission`):

```rust
    #[tokio::test]
    #[serial]
    async fn test_sponsorblock_enabled_defaults_persists_and_tolerates_omission() {
        let server = handle_test_startup().await;

        // Seeded default is true.
        let default_get = server.test_server.get("/api/v1/settings").await;
        assert_eq!(default_get.status_code(), 200);
        assert!(default_get.json::<Setting>().sponsorblock_enabled);

        // Updating to false round-trips.
        let updated = server
            .test_server
            .put("/api/v1/settings")
            .json(&json!({
                "id": uuid::Uuid::nil().to_string(),
                "autoDownload": true,
                "autoUpdate": true,
                "autoCleanup": true,
                "autoCleanupDays": 30,
                "podcastPrefill": 5,
                "replaceInvalidCharacters": false,
                "useExistingFilename": false,
                "replacementStrategy": "replace-with-dash",
                "episodeFormat": "{episodeTitle}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": false,
                "autoTranscodeOpus": false,
                "useOneCoverForAllEpisodes": false,
                "maxParallelDownloads": 3,
                "sponsorblockEnabled": false
            }))
            .await;
        assert_eq!(updated.status_code(), 200);
        assert!(!updated.json::<Setting>().sponsorblock_enabled);

        // Backward compatibility: omitting the field defaults to true.
        let legacy = server
            .test_server
            .put("/api/v1/settings")
            .json(&json!({
                "id": uuid::Uuid::nil().to_string(),
                "autoDownload": true,
                "autoUpdate": true,
                "autoCleanup": true,
                "autoCleanupDays": 30,
                "podcastPrefill": 5,
                "replaceInvalidCharacters": false,
                "useExistingFilename": false,
                "replacementStrategy": "replace-with-dash",
                "episodeFormat": "{episodeTitle}",
                "podcastFormat": "{podcastTitle}",
                "directPaths": false,
                "autoTranscodeOpus": false,
                "useOneCoverForAllEpisodes": false,
                "maxParallelDownloads": 3
            }))
            .await;
        assert_eq!(legacy.status_code(), 200);
        assert!(legacy.json::<Setting>().sponsorblock_enabled);
    }
```

- [ ] **Step 2: Run it — verify it fails to compile**

Run: `cargo test -p podfetch-web sponsorblock_enabled_defaults -- --nocapture`
Expected: FAIL — `no field 'sponsorblock_enabled' on type 'Setting'`.

- [ ] **Step 3: Add the field across all layers**

`crates/podfetch-domain/src/settings.rs` — add `pub sponsorblock_enabled: bool,` as the last field of `Setting` (after `max_parallel_downloads: i32,`).

`crates/podfetch-persistence/src/settings.rs`:
- In the in-file `diesel::table! { settings ... }` block add `sponsorblock_enabled -> Bool,` after `max_parallel_downloads -> Integer,`.
- In `struct SettingEntity` add `sponsorblock_enabled: bool,` after `max_parallel_downloads: i32,`.
- In `impl From<SettingEntity> for Setting` add `sponsorblock_enabled: value.sponsorblock_enabled,` after the `max_parallel_downloads` line.
- In `impl From<Setting> for SettingEntity` add `sponsorblock_enabled: value.sponsorblock_enabled,` after the `max_parallel_downloads` line.
- In `insert_default_settings`, add `sponsorblock_enabled.eq(true),` after `max_parallel_downloads.eq(3),`.

`crates/podfetch-web/src/settings.rs` — add the field with a serde default, and a default fn, after the `max_parallel_downloads` field of the web `Setting`:
```rust
    /// Enable SponsorBlock for this instance. Defaulted on deserialize so older
    /// clients that omit it keep working.
    #[serde(default = "default_sponsorblock_enabled")]
    pub sponsorblock_enabled: bool,
}

fn default_sponsorblock_enabled() -> bool {
    true
}
```
Then add `sponsorblock_enabled: value.sponsorblock_enabled,` to BOTH `From` impls in that file (after each `max_parallel_downloads` line).

`crates/podfetch-web/src/services/settings/service.rs` — in `build_name_only_setting` add `sponsorblock_enabled: true,` after `max_parallel_downloads: 3,`.

- [ ] **Step 4: Run the test — verify it passes**

Run: `cargo test -p podfetch-web sponsorblock_enabled_defaults -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-domain/src/settings.rs crates/podfetch-persistence/src/settings.rs crates/podfetch-web/src/settings.rs crates/podfetch-web/src/services/settings/service.rs crates/podfetch-web/src/controllers/settings_controller.rs
git commit -m "feat(sponsorblock): add global sponsorblock_enabled setting"
```

---

## Task 4: YouTube video-ID extraction (pure function, TDD)

**Files:**
- Create: `crates/podfetch-web/src/services/sponsorblock/mod.rs`
- Create: `crates/podfetch-web/src/services/sponsorblock/video_id.rs`
- Modify: `crates/podfetch-web/src/services/mod.rs`

- [ ] **Step 1: Register the module**

In `crates/podfetch-web/src/services/mod.rs`, add `pub mod sponsorblock;` alongside the other `pub mod` lines.

Create `crates/podfetch-web/src/services/sponsorblock/mod.rs`:
```rust
pub mod client;
pub mod service;
pub mod video_id;
```

> `client` and `service` modules are created in later tasks; if you implement strictly task-by-task, temporarily comment out the `pub mod client;` and `pub mod service;` lines until Tasks 6 and 8, or create empty stub files. Re-enable as you add them.

- [ ] **Step 2: Write the failing test**

Create `crates/podfetch-web/src/services/sponsorblock/video_id.rs`:

```rust
//! Extracts a YouTube video ID from an RSS item. SponsorBlock data is keyed by
//! YouTube video ID, so this gates the whole feature: no ID -> not a YouTube
//! episode -> no SponsorBlock lookup.

/// Extract an 11-character YouTube video ID from the most reliable source on the
/// item, in priority order: `<link>`, then `<guid>`, then the enclosure URL.
pub fn extract_youtube_video_id(
    link: Option<&str>,
    guid: Option<&str>,
    enclosure_url: Option<&str>,
) -> Option<String> {
    for candidate in [link, guid, enclosure_url].into_iter().flatten() {
        if let Some(id) = parse_video_id(candidate) {
            return Some(id);
        }
    }
    None
}

/// Parse a single string for a YouTube video ID. Handles watch URLs, youtu.be,
/// /embed/, music.youtube.com, the `yt:video:ID` guid form, and a bare ID.
fn parse_video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // yt:video:VIDEOID  (YouTube channel/uploads feed guid form)
    if let Some(rest) = trimmed.strip_prefix("yt:video:") {
        return valid_id(rest);
    }

    // URL forms.
    if let Ok(url) = url::Url::parse(trimmed) {
        let host = url.host_str().unwrap_or("").trim_start_matches("www.");
        match host {
            "youtu.be" => {
                if let Some(seg) = url.path_segments().and_then(|mut s| s.next()) {
                    return valid_id(seg);
                }
            }
            "youtube.com" | "m.youtube.com" | "music.youtube.com" => {
                // /watch?v=ID
                if let Some((_, v)) = url.query_pairs().find(|(k, _)| k == "v") {
                    return valid_id(&v);
                }
                // /embed/ID  or  /shorts/ID  or  /v/ID
                let segs: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();
                if let [kind, id, ..] = segs.as_slice()
                    && matches!(*kind, "embed" | "shorts" | "v")
                {
                    return valid_id(id);
                }
            }
            _ => {}
        }
        return None;
    }

    // Bare 11-char id (last resort).
    valid_id(trimmed)
}

/// A YouTube video ID is exactly 11 chars of [A-Za-z0-9_-].
fn valid_id(candidate: &str) -> Option<String> {
    let id = candidate.trim();
    if id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
        Some(id.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_url_in_link() {
        assert_eq!(
            extract_youtube_video_id(Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn youtu_be_short_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://youtu.be/dQw4w9WgXcQ?t=10"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn embed_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://www.youtube.com/embed/dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn music_youtube_url() {
        assert_eq!(
            extract_youtube_video_id(Some("https://music.youtube.com/watch?v=dQw4w9WgXcQ"), None, None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn yt_video_guid_form() {
        assert_eq!(
            extract_youtube_video_id(None, Some("yt:video:dQw4w9WgXcQ"), None),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn falls_back_to_enclosure_when_link_and_guid_have_no_id() {
        assert_eq!(
            extract_youtube_video_id(
                Some("https://example.com/post/123"),
                Some("urn:uuid:abc"),
                Some("https://youtu.be/dQw4w9WgXcQ")
            ),
            Some("dQw4w9WgXcQ".to_string())
        );
    }

    #[test]
    fn non_youtube_returns_none() {
        assert_eq!(
            extract_youtube_video_id(
                Some("https://anchor.fm/s/123/podcast/play/456/episode.mp3"),
                Some("9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"),
                Some("https://anchor.fm/s/123/audio.mp3")
            ),
            None
        );
    }

    #[test]
    fn malformed_id_returns_none() {
        // Too short.
        assert_eq!(extract_youtube_video_id(Some("https://youtu.be/abc"), None, None), None);
        // Wrong characters.
        assert_eq!(extract_youtube_video_id(None, Some("yt:video:dQw4w9WgX!Q"), None), None);
    }
}
```

- [ ] **Step 3: Run the tests — verify they fail then pass**

Run: `cargo test -p podfetch-web services::sponsorblock::video_id -- --nocapture`
Expected: First run after creating the file should PASS (the impl is included above). If you wrote tests first with an empty impl, expect FAIL with "not found", then add the impl and re-run to PASS.

> Note: `url` is already a dependency of `podfetch-web` (used in `services/download/service.rs`). If `cargo` reports it missing, add `url = { workspace = true }` to `crates/podfetch-web/Cargo.toml`.

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/sponsorblock/ crates/podfetch-web/src/services/mod.rs
git commit -m "feat(sponsorblock): add youtube video id extraction"
```

---

## Task 5: Wire video-ID extraction into episode ingest

**Files:**
- Modify: `crates/podfetch-web/src/usecases/podcast_episode/mod.rs`

- [ ] **Step 1: Use the extractor in `insert_podcast_episode`**

In `crates/podfetch-web/src/usecases/podcast_episode/mod.rs`, in `insert_podcast_episode`, compute the video ID from the item and add it to the `NewPodcastEpisode` literal. Replace the `NewPodcastEpisode { ... }` construction so it reads:

```rust
        let youtube_video_id = crate::services::sponsorblock::video_id::extract_youtube_video_id(
            item.link.as_deref(),
            item.guid.as_ref().map(|g| g.value.as_str()),
            item.enclosure.as_ref().map(|e| e.url.as_str()),
        );

        Self::repo()
            .create(NewPodcastEpisode {
                podcast_id: Self::parse_id(&podcast.id)?,
                episode_id: uuid::Uuid::new_v4().to_string(),
                name: item
                    .title
                    .clone()
                    .unwrap_or_else(|| "No title given".to_string()),
                url: item.enclosure.clone().unwrap().url,
                date_of_recording: Self::parse_recording_date(item),
                image_url: episode_image_url.to_string(),
                total_time: duration,
                description: opt_or_empty_string(item.clone().description),
                guid: item.guid.clone().unwrap_or(guid_to_insert).value,
                youtube_video_id,
            })
            .map(Into::into)
            .map_err(Into::into)
```

- [ ] **Step 2: Compile**

Run: `cargo build -p podfetch-web`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/usecases/podcast_episode/mod.rs
git commit -m "feat(sponsorblock): capture youtube video id when ingesting episodes"
```

---

## Task 6: SponsorBlock HTTP client + JSON parsing (TDD on the parser)

**Files:**
- Create: `crates/podfetch-web/src/services/sponsorblock/client.rs`

SponsorBlock categories we store + request: `sponsor`, `selfpromo`, `interaction`, `intro`, `outro`, `preview`, `filler`, `music_offtopic`. We use the privacy-preserving hash-prefix endpoint `GET {base}/api/skipSegments/{sha256_prefix}?categories=[...]&actionTypes=["skip"]`, which returns an array of `{ videoID, hash, segments: [...] }`; we keep only entries whose `videoID` matches ours.

- [ ] **Step 1: Write the client with a pure parser + failing parser test**

Create `crates/podfetch-web/src/services/sponsorblock/client.rs`:

```rust
//! Thin SponsorBlock API client built on the project's existing async reqwest
//! client. Uses the privacy-preserving hash-prefix endpoint so the exact video
//! ID never leaves this server.

use common_infrastructure::error::{map_reqwest_error, CustomError};
use common_infrastructure::http::{get_async_sync_client, COMMON_USER_AGENT};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use reqwest::header::USER_AGENT;
use serde::Deserialize;

/// Categories PodFetch knows how to skip. Anything else returned by the API is
/// ignored at parse time.
pub const SUPPORTED_CATEGORIES: [&str; 8] = [
    "sponsor",
    "selfpromo",
    "interaction",
    "intro",
    "outro",
    "preview",
    "filler",
    "music_offtopic",
];

/// A single segment as PodFetch stores/uses it (milliseconds).
#[derive(Debug, Clone, PartialEq)]
pub struct FetchedSegment {
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    /// The video duration SponsorBlock recorded (seconds); 0.0 if unknown.
    pub video_duration_secs: f64,
}

// ---- Raw API shapes (hash-prefix endpoint) ----

#[derive(Debug, Deserialize)]
struct RawVideo {
    #[serde(rename = "videoID")]
    video_id: String,
    segments: Vec<RawSegment>,
}

#[derive(Debug, Deserialize)]
struct RawSegment {
    #[serde(rename = "UUID")]
    uuid: String,
    category: String,
    #[serde(rename = "actionType")]
    action_type: String,
    /// [start, end] in floating-point seconds.
    segment: [f64; 2],
    #[serde(default)]
    votes: i32,
    #[serde(default)]
    locked: i32,
    #[serde(rename = "videoDuration", default)]
    video_duration: f64,
}

/// Parse the hash-prefix endpoint response, keeping only segments for `video_id`
/// whose category is supported. Pure function — unit tested without network.
pub fn parse_hash_response(body: &str, video_id: &str) -> Result<Vec<FetchedSegment>, CustomError> {
    let videos: Vec<RawVideo> = serde_json::from_str(body).map_err(|e| {
        common_infrastructure::error::CustomErrorInner::Conflict(
            format!("Failed to parse SponsorBlock response: {e}"),
            common_infrastructure::error::ErrorSeverity::Warning,
        )
    })?;

    let mut out = Vec::new();
    for video in videos.into_iter().filter(|v| v.video_id == video_id) {
        for seg in video.segments {
            if !SUPPORTED_CATEGORIES.contains(&seg.category.as_str()) {
                continue;
            }
            let start_ms = (seg.segment[0] * 1000.0).round() as i64;
            let end_ms = (seg.segment[1] * 1000.0).round() as i64;
            if end_ms <= start_ms {
                continue; // drop zero/negative-length segments
            }
            out.push(FetchedSegment {
                uuid: seg.uuid,
                category: seg.category,
                action_type: seg.action_type,
                start_ms,
                end_ms,
                votes: seg.votes,
                locked: seg.locked != 0,
                video_duration_secs: seg.video_duration,
            });
        }
    }
    Ok(out)
}

/// Base URL, overridable via `SPONSORBLOCK_API_URL` for self-hosted mirrors.
fn base_url() -> String {
    std::env::var("SPONSORBLOCK_API_URL")
        .unwrap_or_else(|_| "https://sponsor.ajay.app".to_string())
}

/// Query SponsorBlock for one video. Returns an empty Vec on 404 (no data).
pub async fn fetch_segments(video_id: &str) -> Result<Vec<FetchedSegment>, CustomError> {
    let hash = sha256::digest(video_id);
    let prefix = &hash[..4];
    let categories = serde_json::to_string(&SUPPORTED_CATEGORIES).unwrap();
    let url = format!("{}/api/skipSegments/{}", base_url(), prefix);

    let client = get_async_sync_client(&ENVIRONMENT_SERVICE)
        .build()
        .map_err(map_reqwest_error)?;

    let resp = client
        .get(&url)
        .header(USER_AGENT, COMMON_USER_AGENT)
        .query(&[("categories", categories.as_str()), ("actionTypes", "[\"skip\"]")])
        .send()
        .await
        .map_err(map_reqwest_error)?;

    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }
    if !resp.status().is_success() {
        return Err(common_infrastructure::error::CustomErrorInner::Conflict(
            format!("SponsorBlock returned status {}", resp.status()),
            common_infrastructure::error::ErrorSeverity::Warning,
        )
        .into());
    }

    let body = resp.text().await.map_err(map_reqwest_error)?;
    parse_hash_response(&body, video_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"[
      {"videoID":"dQw4w9WgXcQ","hash":"e0c4...","segments":[
        {"UUID":"seg-1","category":"sponsor","actionType":"skip","segment":[30.5,45.0],"votes":12,"locked":1,"videoDuration":600.0},
        {"UUID":"seg-2","category":"music_offtopic","actionType":"skip","segment":[0.0,0.0],"votes":0,"locked":0,"videoDuration":600.0},
        {"UUID":"seg-3","category":"chapter","actionType":"chapter","segment":[100.0,120.0],"votes":3,"locked":0,"videoDuration":600.0}
      ]},
      {"videoID":"other00video","hash":"e0c4...","segments":[
        {"UUID":"x","category":"sponsor","actionType":"skip","segment":[1.0,2.0],"votes":0,"locked":0,"videoDuration":10.0}
      ]}
    ]"#;

    #[test]
    fn parses_and_filters_to_target_video() {
        let segs = parse_hash_response(SAMPLE, "dQw4w9WgXcQ").unwrap();
        // seg-1 kept; seg-2 dropped (zero length); seg-3 dropped (unsupported
        // category); the "other00video" entry dropped (wrong video).
        assert_eq!(segs.len(), 1);
        let s = &segs[0];
        assert_eq!(s.uuid, "seg-1");
        assert_eq!(s.category, "sponsor");
        assert_eq!(s.start_ms, 30500);
        assert_eq!(s.end_ms, 45000);
        assert_eq!(s.votes, 12);
        assert!(s.locked);
        assert_eq!(s.video_duration_secs, 600.0);
    }

    #[test]
    fn unknown_video_yields_empty() {
        assert!(parse_hash_response(SAMPLE, "no-such-id").unwrap().is_empty());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse_hash_response("not json", "x").is_err());
    }
}
```

- [ ] **Step 2: Run the parser tests**

Run: `cargo test -p podfetch-web services::sponsorblock::client -- --nocapture`
Expected: PASS (3 tests).

> If `serde_json` is not a direct dependency of `podfetch-web`, add `serde_json = { workspace = true }` to `crates/podfetch-web/Cargo.toml`. Confirm `common_infrastructure::http` exposes `get_async_sync_client` and `COMMON_USER_AGENT` (used already by `services/download/service.rs`); if the import paths differ, match those used there.

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/services/sponsorblock/client.rs
git commit -m "feat(sponsorblock): add API client with hash-prefix privacy lookup"
```

---

## Task 7: Persistence repository for segments + user settings

**Files:**
- Create: `crates/podfetch-persistence/src/sponsorblock.rs`
- Modify: `crates/podfetch-persistence/src/lib.rs` (add `pub mod sponsorblock;`)

This uses the inherent-method + `get_connection()` style (the free function used by `services/download/service.rs`), returning `Result<_, CustomError>` (a `From<diesel::result::Error> for CustomError` already exists — it's used throughout the persistence repos via `.map_err(Into::into)`).

- [ ] **Step 1: Create the module**

Create `crates/podfetch-persistence/src/sponsorblock.rs`:

```rust
use crate::db::get_connection;
use crate::schema::{episode_sponsor_segments, sponsorblock_user_settings};
use common_infrastructure::error::CustomError;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, PartialEq)]
#[diesel(table_name = episode_sponsor_segments)]
pub struct SponsorSegmentEntity {
    pub id: String,
    pub episode_id: String,
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    pub duration_mismatch: bool,
    pub fetched_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, PartialEq)]
#[diesel(table_name = sponsorblock_user_settings)]
pub struct SponsorblockUserSettingsEntity {
    pub user_id: String,
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

pub struct SponsorblockRepository;

impl SponsorblockRepository {
    /// Idempotently replace ALL stored segments for an episode with `segments`.
    /// Done in a transaction so a concurrent reader never sees a half-written set.
    pub fn replace_segments_for_episode(
        episode_id_value: &str,
        segments: Vec<SponsorSegmentEntity>,
    ) -> Result<(), CustomError> {
        use self::episode_sponsor_segments::dsl as s;
        let mut conn = get_connection();
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            diesel::delete(s::episode_sponsor_segments.filter(s::episode_id.eq(episode_id_value)))
                .execute(conn)?;
            if !segments.is_empty() {
                diesel::insert_into(s::episode_sponsor_segments)
                    .values(&segments)
                    .execute(conn)?;
            }
            Ok(())
        })
        .map_err(Into::into)
    }

    pub fn get_segments_for_episode(
        episode_id_value: &str,
    ) -> Result<Vec<SponsorSegmentEntity>, CustomError> {
        use self::episode_sponsor_segments::dsl as s;
        s::episode_sponsor_segments
            .filter(s::episode_id.eq(episode_id_value))
            .order(s::start_ms.asc())
            .load::<SponsorSegmentEntity>(&mut get_connection())
            .map_err(Into::into)
    }

    pub fn get_user_settings(
        user_id_value: &str,
    ) -> Result<Option<SponsorblockUserSettingsEntity>, CustomError> {
        use self::sponsorblock_user_settings::dsl as u;
        u::sponsorblock_user_settings
            .filter(u::user_id.eq(user_id_value))
            .first::<SponsorblockUserSettingsEntity>(&mut get_connection())
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_user_settings(
        settings: SponsorblockUserSettingsEntity,
    ) -> Result<(), CustomError> {
        use self::sponsorblock_user_settings::dsl as u;
        let mut conn = get_connection();
        let existing = u::sponsorblock_user_settings
            .filter(u::user_id.eq(&settings.user_id))
            .first::<SponsorblockUserSettingsEntity>(&mut conn)
            .optional()?;
        match existing {
            Some(_) => {
                diesel::update(
                    u::sponsorblock_user_settings.filter(u::user_id.eq(&settings.user_id)),
                )
                .set(&settings)
                .execute(&mut conn)?;
            }
            None => {
                diesel::insert_into(u::sponsorblock_user_settings)
                    .values(&settings)
                    .execute(&mut conn)?;
            }
        }
        Ok(())
    }
}
```

In `crates/podfetch-persistence/src/lib.rs`, add `pub mod sponsorblock;`.

- [ ] **Step 2: Compile**

Run: `cargo build -p podfetch-persistence`
Expected: PASS.

> If `chrono::NaiveDateTime` import errors, mirror `podcast_episode.rs`'s import (`use chrono::NaiveDateTime;`). If `AsChangeset` complains about the primary key, add `#[diesel(treat_none_as_null = false)]` is not needed; instead derive `AsChangeset` only where used — `SponsorSegmentEntity` does not need `AsChangeset` (it's insert/delete only), so you may remove `AsChangeset` from its derive list if the compiler objects to the `id`/composite key.

- [ ] **Step 3: Write a DB round-trip test**

Add to the bottom of `crates/podfetch-persistence/src/sponsorblock.rs` (this exercises replace-idempotency and ordering). Use the project's existing test-DB harness — search the crate for how other persistence tests obtain a connection/migrated DB (e.g. an existing `#[cfg(test)]` module in `filter.rs` or a `test_utils` helper) and mirror it. The assertions to implement:

```rust
#[cfg(test)]
mod tests {
    // Mirror the existing persistence test setup in this crate to get a
    // migrated connection (see filter.rs / podcast_episode.rs test modules).
    //
    // Test 1 (replace_is_idempotent): insert an episode row, call
    // replace_segments_for_episode twice with the SAME two segments; assert
    // get_segments_for_episode returns exactly 2 (no duplicates) and ordered by
    // start_ms ascending.
    //
    // Test 2 (replace_removes_stale): replace with [A,B], then replace with [A];
    // assert only A remains.
    //
    // Test 3 (user_settings_upsert): upsert settings for a user, read back, flip
    // enabled, upsert again, assert the row updated (still exactly one row).
}
```

Implement the three tests using the crate's existing migrated-connection test pattern.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p podfetch-persistence sponsorblock -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-persistence/src/sponsorblock.rs crates/podfetch-persistence/src/lib.rs
git commit -m "feat(sponsorblock): add persistence repo for segments and user settings"
```

---

## Task 8: `fetch_and_store` orchestration + duration guard (TDD on the guard)

**Files:**
- Create: `crates/podfetch-web/src/services/sponsorblock/service.rs`

- [ ] **Step 1: Write the service with a pure duration-guard helper + tests**

Create `crates/podfetch-web/src/services/sponsorblock/service.rs`:

```rust
//! Orchestrates fetching SponsorBlock data for a downloaded episode and storing
//! it. All failures are the caller's to swallow — see download/service.rs.

use crate::services::sponsorblock::client::{self, FetchedSegment};
use common_infrastructure::error::CustomError;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_persistence::sponsorblock::{SponsorblockRepository, SponsorSegmentEntity};

/// Tolerance for declaring a duration mismatch: the larger of 2 seconds or 1%.
fn durations_mismatch(episode_secs: i64, sb_secs: f64) -> bool {
    // If either side is unknown (<= 0) we cannot verify alignment, so we do NOT
    // flag — better to skip than to disable the feature for feeds without a
    // duration. Only a confident, sizeable divergence flags as a mismatch.
    if episode_secs <= 0 || sb_secs <= 0.0 {
        return false;
    }
    let diff = (episode_secs as f64 - sb_secs).abs();
    let tolerance = (sb_secs * 0.01).max(2.0);
    diff > tolerance
}

/// Fetch + store SponsorBlock segments for one episode. Returns the number of
/// segments stored. Caller is responsible for non-fatal error handling.
pub async fn fetch_and_store(episode: &PodcastEpisode) -> Result<usize, CustomError> {
    // Guard 1: global toggle.
    let settings = crate::services::settings::service::SettingsService::shared().get_settings()?;
    let enabled = settings.map(|s| s.sponsorblock_enabled).unwrap_or(true);
    if !enabled {
        return Ok(0);
    }

    // Guard 2: must be a YouTube episode.
    let Some(video_id) = episode.youtube_video_id.clone() else {
        return Ok(0);
    };

    let fetched: Vec<FetchedSegment> = client::fetch_segments(&video_id).await?;

    let now = chrono::Utc::now().naive_utc();
    let episode_secs = episode.total_time as i64;

    let rows: Vec<SponsorSegmentEntity> = fetched
        .into_iter()
        .map(|seg| SponsorSegmentEntity {
            id: uuid::Uuid::new_v4().to_string(),
            episode_id: episode.id.clone(),
            uuid: seg.uuid,
            category: seg.category,
            action_type: seg.action_type,
            start_ms: seg.start_ms,
            end_ms: seg.end_ms,
            votes: seg.votes,
            locked: seg.locked,
            duration_mismatch: durations_mismatch(episode_secs, seg.video_duration_secs),
            fetched_at: now,
        })
        .collect();

    let count = rows.len();
    SponsorblockRepository::replace_segments_for_episode(&episode.id, rows)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::durations_mismatch;

    #[test]
    fn unknown_durations_never_mismatch() {
        assert!(!durations_mismatch(0, 600.0));
        assert!(!durations_mismatch(600, 0.0));
    }

    #[test]
    fn close_durations_do_not_mismatch() {
        // 600s vs 601s -> diff 1s, tolerance max(6, 2)=6 -> ok.
        assert!(!durations_mismatch(600, 601.0));
    }

    #[test]
    fn large_divergence_flags_mismatch() {
        // 600s vs 540s -> diff 60s, tolerance 6 -> mismatch.
        assert!(durations_mismatch(600, 540.0));
    }

    #[test]
    fn small_videos_use_two_second_floor() {
        // 100s vs 103s -> diff 3s, tolerance max(1.03, 2)=2 -> mismatch.
        assert!(durations_mismatch(100, 103.0));
        // 100s vs 101s -> diff 1s -> ok.
        assert!(!durations_mismatch(100, 101.0));
    }
}
```

- [ ] **Step 2: Run the guard tests**

Run: `cargo test -p podfetch-web services::sponsorblock::service -- --nocapture`
Expected: PASS (4 tests).

> Confirm `SettingsService::shared().get_settings()` returns `Option<Setting>` where `Setting` carries `sponsorblock_enabled` (added in Task 3). Match the exact path used in `services/download/service.rs` (`crate::services::settings::service::SettingsService::shared().get_settings()`).

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/services/sponsorblock/service.rs
git commit -m "feat(sponsorblock): add fetch_and_store with duration-mismatch guard"
```

---

## Task 9: Hook into download + rescan

**Files:**
- Modify: `crates/podfetch-web/src/services/download/service.rs`
- Modify: `crates/podfetch-web/src/services/episode_rescan/service.rs`

- [ ] **Step 1: Call `fetch_and_store` after metadata insertion in the downloader**

In `crates/podfetch-web/src/services/download/service.rs`, `download_podcast_episode` is a **sync** fn. After the chapter-saving block and before the transcode block (i.e. right after the `if let Err(err) = result { ... }` that logs metadata errors, around line 323), add a non-fatal SponsorBlock fetch. Because the surrounding fn is sync, drive the async call on a fresh runtime:

```rust
        // SponsorBlock: fetch + store segments for YouTube-sourced episodes.
        // Non-fatal — a failure here must never fail the download.
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            let sb_result = rt.block_on(
                crate::services::sponsorblock::service::fetch_and_store(&podcast_episode),
            );
            match sb_result {
                Ok(n) if n > 0 => {
                    tracing::info!("Stored {n} SponsorBlock segments for episode {}", podcast_episode.id)
                }
                Ok(_) => {}
                Err(err) => tracing::warn!(
                    "SponsorBlock fetch failed for episode {}: {err}",
                    podcast_episode.id
                ),
            }
        }
```

> If `download_podcast_episode` already runs inside a tokio runtime context, `Runtime::new()` inside it will fail (`Cannot start a runtime from within a runtime`). It is invoked from a blocking download thread (see how downloads are spawned), so a fresh runtime is correct. If a future refactor makes the fn async, replace this block with a direct `.await`. Verify which by checking the caller of `download_podcast_episode`; if it is `spawn_blocking`/a plain thread, this block is correct.

- [ ] **Step 2: Add the `refetch_sponsorblock` rescan option**

In `crates/podfetch-web/src/services/episode_rescan/service.rs`, add to `RescanOptions` (after `apply_metadata: bool,`):

```rust
    /// Re-query SponsorBlock and replace stored segments for the episode.
    pub refetch_sponsorblock: bool,
}
```

Update `any_enabled`:

```rust
    pub fn any_enabled(&self) -> bool {
        self.apply_filenames
            || self.apply_transcode
            || self.apply_covers
            || self.apply_metadata
            || self.refetch_sponsorblock
    }
```

Add handling at the end of `apply_to_episode`, just before the final `Ok(())`. Episodes may pre-date the feature, so backfill the video ID first if it is null:

```rust
        // Step 5: re-fetch SponsorBlock segments if requested. Backfill the
        // youtube_video_id from the episode URL when it's missing (older rows).
        if opts.refetch_sponsorblock {
            let episode_for_fetch = if episode.youtube_video_id.is_none() {
                let backfilled = crate::services::sponsorblock::video_id::extract_youtube_video_id(
                    None,
                    Some(&episode.guid),
                    Some(&episode.url),
                );
                if let Some(ref vid) = backfilled {
                    if let Err(err) = PodcastEpisodeService::update_youtube_video_id(
                        &episode.episode_id,
                        vid,
                    ) {
                        tracing::warn!(
                            "Could not persist backfilled youtube_video_id for {}: {err}",
                            episode.episode_id
                        );
                    }
                }
                let mut e = episode.clone();
                e.youtube_video_id = backfilled;
                e
            } else {
                episode.clone()
            };

            if let Ok(rt) = tokio::runtime::Runtime::new() {
                match rt.block_on(crate::services::sponsorblock::service::fetch_and_store(
                    &episode_for_fetch,
                )) {
                    Ok(n) => stats.metadata_refreshed += n.min(1), // count episodes touched
                    Err(err) => {
                        tracing::warn!(
                            "SponsorBlock refetch failed for {}: {err}",
                            episode.episode_id
                        );
                        stats.errors += 1;
                    }
                }
            }
        }
```

> `PodcastEpisodeService::update_youtube_video_id` does not exist yet — add a small method to the episode usecase + persistence repo that runs `UPDATE podcast_episodes SET youtube_video_id = ? WHERE episode_id = ?`, mirroring the existing `update_local_paths`/`update_guid` methods in `crates/podfetch-web/src/usecases/podcast_episode/mod.rs` and the persistence repo. If you prefer to keep this task small, omit the backfill-persist (drop the `update_youtube_video_id` call and just use the in-memory `backfilled` value for this run); the segments still get fetched and stored. Persisting the id is the better behaviour but is optional for v1.

- [ ] **Step 3: Compile**

Run: `cargo build -p podfetch-web`
Expected: PASS.

- [ ] **Step 4: Verify the existing download regression test still passes**

Run: `cargo test -p podfetch-web download -- --nocapture`
Expected: PASS — the existing `handle_metadata_insertion_returns_err_when_audio_file_is_missing` and other download tests are unaffected (SponsorBlock runs after metadata and is non-fatal).

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/services/download/service.rs crates/podfetch-web/src/services/episode_rescan/service.rs crates/podfetch-web/src/usecases/podcast_episode/mod.rs
git commit -m "feat(sponsorblock): fetch on download + add refetch rescan option"
```

---

## Task 10: API endpoints (segments + per-user prefs)

**Files:**
- Create: `crates/podfetch-web/src/controllers/sponsorblock_controller.rs`
- Modify: `crates/podfetch-web/src/controllers/mod.rs` (add `pub mod sponsorblock_controller;`)
- Modify: `crates/podfetch-web/src/startup.rs` (merge the router)

Endpoints (under `/api/v1`):
- `GET /podcasts/episodes/{id}/sponsorblock` → `{ segments: [...], preferences: {...} }`
- `GET /settings/sponsorblock` → current user's prefs (defaults applied if absent)
- `PUT /settings/sponsorblock` → upsert current user's prefs

- [ ] **Step 1: Create the controller with DTOs, handlers, and a router**

Create `crates/podfetch-web/src/controllers/sponsorblock_controller.rs`:

```rust
use crate::app_state::AppState;
use crate::controllers::id_resolver::{parse_resolved_id, ResolvedId};
use crate::services::podcast::service::PodcastService;
use axum::extract::Path;
use axum::{Extension, Json};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_domain::user::User;
use podfetch_persistence::sponsorblock::{
    SponsorblockRepository, SponsorblockUserSettingsEntity,
};
use serde::{Deserialize, Serialize};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorSegmentDto {
    pub uuid: String,
    pub category: String,
    pub action_type: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub votes: i32,
    pub locked: bool,
    pub duration_mismatch: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorblockUserSettingsDto {
    pub enabled: bool,
    pub skip_sponsor: bool,
    pub skip_selfpromo: bool,
    pub skip_interaction: bool,
    pub skip_intro: bool,
    pub skip_outro: bool,
    pub skip_preview: bool,
    pub skip_filler: bool,
    pub skip_music_offtopic: bool,
}

impl SponsorblockUserSettingsDto {
    /// Defaults for a user with no stored row.
    fn default_for(user_id: &str) -> Self {
        let _ = user_id;
        Self {
            enabled: true,
            skip_sponsor: true,
            skip_selfpromo: true,
            skip_interaction: false,
            skip_intro: false,
            skip_outro: false,
            skip_preview: false,
            skip_filler: false,
            skip_music_offtopic: false,
        }
    }

    fn into_entity(self, user_id: String) -> SponsorblockUserSettingsEntity {
        SponsorblockUserSettingsEntity {
            user_id,
            enabled: self.enabled,
            skip_sponsor: self.skip_sponsor,
            skip_selfpromo: self.skip_selfpromo,
            skip_interaction: self.skip_interaction,
            skip_intro: self.skip_intro,
            skip_outro: self.skip_outro,
            skip_preview: self.skip_preview,
            skip_filler: self.skip_filler,
            skip_music_offtopic: self.skip_music_offtopic,
        }
    }
}

impl From<SponsorblockUserSettingsEntity> for SponsorblockUserSettingsDto {
    fn from(e: SponsorblockUserSettingsEntity) -> Self {
        Self {
            enabled: e.enabled,
            skip_sponsor: e.skip_sponsor,
            skip_selfpromo: e.skip_selfpromo,
            skip_interaction: e.skip_interaction,
            skip_intro: e.skip_intro,
            skip_outro: e.skip_outro,
            skip_preview: e.skip_preview,
            skip_filler: e.skip_filler,
            skip_music_offtopic: e.skip_music_offtopic,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SponsorblockEpisodeResponse {
    pub segments: Vec<SponsorSegmentDto>,
    pub preferences: SponsorblockUserSettingsDto,
}

/// Resolve an episode `{id}` path segment (UUID or legacy integer) to the
/// canonical episode `Uuid` string used as `episode_sponsor_segments.episode_id`.
fn resolve_episode_uuid(id: &str) -> Result<String, CustomError> {
    match parse_resolved_id(id)? {
        ResolvedId::Uuid(uuid) => Ok(uuid.to_string()),
        ResolvedId::Legacy(legacy) => {
            let ep = PodcastService::get_episode_by_legacy_id(legacy)?;
            Ok(ep.id)
        }
    }
}

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/sponsorblock",
    responses(
        (status = 200, description = "SponsorBlock segments + caller preferences", body = SponsorblockEpisodeResponse)
    ),
    tag = "sponsorblock"
)]
pub async fn get_episode_sponsorblock(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<SponsorblockEpisodeResponse>, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?;

    let segments = SponsorblockRepository::get_segments_for_episode(&episode_id)?
        .into_iter()
        .map(|e| SponsorSegmentDto {
            uuid: e.uuid,
            category: e.category,
            action_type: e.action_type,
            start_ms: e.start_ms,
            end_ms: e.end_ms,
            votes: e.votes,
            locked: e.locked,
            duration_mismatch: e.duration_mismatch,
        })
        .collect();

    let prefs = match SponsorblockRepository::get_user_settings(&requester.id.to_string())? {
        Some(e) => e.into(),
        None => SponsorblockUserSettingsDto::default_for(&requester.id.to_string()),
    };

    Ok(Json(SponsorblockEpisodeResponse {
        segments,
        preferences: prefs,
    }))
}

#[utoipa::path(
    get,
    path = "/settings/sponsorblock",
    responses(
        (status = 200, description = "Current user's SponsorBlock preferences", body = SponsorblockUserSettingsDto)
    ),
    tag = "sponsorblock"
)]
pub async fn get_sponsorblock_settings(
    Extension(requester): Extension<User>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    let prefs = match SponsorblockRepository::get_user_settings(&requester.id.to_string())? {
        Some(e) => e.into(),
        None => SponsorblockUserSettingsDto::default_for(&requester.id.to_string()),
    };
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/settings/sponsorblock",
    request_body = SponsorblockUserSettingsDto,
    responses(
        (status = 200, description = "Updated SponsorBlock preferences", body = SponsorblockUserSettingsDto)
    ),
    tag = "sponsorblock"
)]
pub async fn update_sponsorblock_settings(
    Extension(requester): Extension<User>,
    Json(body): Json<SponsorblockUserSettingsDto>,
) -> Result<Json<SponsorblockUserSettingsDto>, CustomError> {
    let user_id = requester.id.to_string();
    SponsorblockRepository::upsert_user_settings(body.clone().into_entity(user_id))?;
    Ok(Json(body))
}

pub fn get_sponsorblock_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_episode_sponsorblock))
        .routes(routes!(get_sponsorblock_settings))
        .routes(routes!(update_sponsorblock_settings))
}
```

> Verify the exact import path for `parse_resolved_id` / `ResolvedId` — it's used in `podcast_episode_controller.rs` as `crate::controllers::id_resolver::{ResolvedId, parse_resolved_id}`. Verify a `PodcastService::get_episode_by_legacy_id` (or equivalent) exists; if the only legacy resolver is `get_podcast_by_legacy_id`, then for episodes use whatever the existing `resolve_episode_uuid` in `podcast_episode_controller.rs` uses and copy it verbatim (it already resolves episode ids — prefer re-using that exact helper if it is `pub`).

- [ ] **Step 2: Register the module + router**

In `crates/podfetch-web/src/controllers/mod.rs`, add `pub mod sponsorblock_controller;`.

In `crates/podfetch-web/src/startup.rs`, inside `get_private_api`, add the merge (alongside the other `.merge(...)` lines):

```rust
        .merge(crate::controllers::sponsorblock_controller::get_sponsorblock_router().with_state(state.clone()))
```

- [ ] **Step 3: Write an API round-trip test**

In a `#[cfg(test)] mod tests` at the bottom of `sponsorblock_controller.rs`, mirror the settings-controller test harness (`handle_test_startup`, `server.test_server`, `#[serial]`):

```rust
#[cfg(test)]
mod tests {
    use super::SponsorblockUserSettingsDto;
    use serde_json::json;
    use serial_test::serial;
    // import handle_test_startup the same way settings_controller tests do

    #[tokio::test]
    #[serial]
    async fn sponsorblock_settings_default_then_roundtrip() {
        let server = handle_test_startup().await;

        // Default for a fresh user: enabled true, sponsor + selfpromo on.
        let got = server.test_server.get("/api/v1/settings/sponsorblock").await;
        assert_eq!(got.status_code(), 200);
        let prefs = got.json::<SponsorblockUserSettingsDto>();
        assert!(prefs.enabled);
        assert!(prefs.skip_sponsor);
        assert!(prefs.skip_selfpromo);
        assert!(!prefs.skip_interaction);

        // Update: disable sponsor, enable interaction.
        let put = server
            .test_server
            .put("/api/v1/settings/sponsorblock")
            .json(&json!({
                "enabled": true,
                "skipSponsor": false,
                "skipSelfpromo": true,
                "skipInteraction": true,
                "skipIntro": false,
                "skipOutro": false,
                "skipPreview": false,
                "skipFiller": false,
                "skipMusicOfftopic": false
            }))
            .await;
        assert_eq!(put.status_code(), 200);

        // Read back persists.
        let after = server
            .test_server
            .get("/api/v1/settings/sponsorblock")
            .await
            .json::<SponsorblockUserSettingsDto>();
        assert!(!after.skip_sponsor);
        assert!(after.skip_interaction);
    }
}
```

> Copy the precise `handle_test_startup` import and any auth/user seeding the settings-controller tests use, so the `Extension<User>` is populated for the test requests.

- [ ] **Step 4: Run the test**

Run: `cargo test -p podfetch-web sponsorblock_settings_default_then_roundtrip -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/controllers/sponsorblock_controller.rs crates/podfetch-web/src/controllers/mod.rs crates/podfetch-web/src/startup.rs
git commit -m "feat(sponsorblock): add segments + user-prefs API endpoints"
```

---

## Task 11: Update generated API types (`schema.d.ts`)

**Files:**
- Modify: `ui/schema.d.ts`
- Modify: `mobile/schema.d.ts`

These are hand-maintained in this repo. The web/mobile client plans depend on these types existing.

- [ ] **Step 1: Determine the canonical OpenAPI shape**

Run the server's OpenAPI dump if available, or read the utoipa-generated spec. Easiest: start the server and GET the OpenAPI JSON (the project serves it for Swagger). Otherwise hand-write the additions to match the DTOs from Task 10.

Run (if a generation script exists): check `ui/package.json` for an `openapi-typescript` script; if present, run it against the running server to regenerate `schema.d.ts`, then copy the result to `mobile/schema.d.ts`.

- [ ] **Step 2: Hand-add the three paths**

If no generator, add to the `paths` interface in `ui/schema.d.ts` (and mirror in `mobile/schema.d.ts`):
- `"/api/v1/podcasts/episodes/{id}/sponsorblock"` with a `get` returning `SponsorblockEpisodeResponse`.
- `"/api/v1/settings/sponsorblock"` with a `get` returning `SponsorblockUserSettingsDto` and a `put` taking/returning `SponsorblockUserSettingsDto`.

Add to `components.schemas`: `SponsorSegmentDto`, `SponsorblockUserSettingsDto`, `SponsorblockEpisodeResponse` with the camelCase field names exactly as serialized (`actionType`, `startMs`, `endMs`, `durationMismatch`, `skipSponsor`, etc.). Follow the formatting of an existing entry (e.g. `PodcastChapterDto`) precisely.

- [ ] **Step 3: Verify the UI still type-checks**

Run (from `ui/`): the project's typecheck/build — check `ui/package.json` for the script name (e.g. `npm run build` which runs `tsc`).
Expected: PASS (no type errors introduced; nothing consumes the new types yet).

- [ ] **Step 4: Commit**

```bash
git add ui/schema.d.ts mobile/schema.d.ts
git commit -m "feat(sponsorblock): add API types to schema.d.ts (ui + mobile)"
```

---

## Task 12: Full verification gate

**Files:** none (verification only)

- [ ] **Step 1: Clippy with warnings-as-errors (matches CI)**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 2: Backend tests on SQLite**

Run the workspace test suite against SQLite (the default). Use the project's standard test invocation (check CI config / `Makefile` / `justfile` for the exact command and any required env).
Expected: PASS, including all new SponsorBlock tests.

- [ ] **Step 3: Backend tests on Postgres**

Run the test suite against Postgres per the project's CI method (it tests both backends). Use the documented Postgres test command/env.
Expected: PASS.

- [ ] **Step 4: Frontend typecheck/build**

Run the `ui` typecheck/build (`tsc` + build) per `ui/package.json`.
Expected: PASS.

- [ ] **Step 5: Do NOT run `cargo fmt`**

Per repo conventions there is no fmt gate; do not reformat.

- [ ] **Step 6: Final commit (if any verification fixups were needed)**

```bash
git add -A
git commit -m "chore(sponsorblock): verification fixups"
```

---

## Notes carried forward to the client plans

- The web/mobile players consume `GET /api/v1/podcasts/episodes/{id}/sponsorblock`, which returns BOTH the segments and the caller's preferences in one payload.
- Active skip ranges = segments where `actionType === "skip"`, `category` enabled in prefs, `durationMismatch === false`, and `preferences.enabled === true`. Merge overlaps; ignore zero/negative ranges (the backend already drops those).
- Per-user prefs are read/written via `GET`/`PUT /api/v1/settings/sponsorblock`.
- The admin global toggle is the `sponsorblockEnabled` field already present on the existing `GET`/`PUT /api/v1/settings` payload.
