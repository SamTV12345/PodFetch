# Remove ENVIRONMENT_SERVICE.server_url Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate `ENVIRONMENT_SERVICE.server_url` and the `UrlRewriter`; all client-visible URLs are built at request time from `X-Forwarded-Host` / `Host` headers; background jobs persist only remote refs (no internal URLs).

**Architecture:** Thread `server_url: &str` (resolved from request headers via the existing `resolve_server_url_from_headers()`) explicitly through every URL-building function. Add a `resolve_image_url()` helper that handles empty / absolute / relative cases. Migrate existing DB rows that store internal default URLs to empty strings. Drop the `UrlRewriter` post-hoc machinery once it has no callers.

**Tech Stack:** Rust, Axum, Diesel (SQLite + PostgreSQL), tokio for async tests.

**Spec:** `docs/superpowers/specs/2026-05-22-remove-server-url-from-env-service-design.md`

---

## File Structure

**New files:**
- `migrations/sqlite/2026-05-22-120000_strip_internal_urls/up.sql`
- `migrations/sqlite/2026-05-22-120000_strip_internal_urls/down.sql`
- `migrations/postgres/2026-05-22-120000_strip_internal_urls/up.sql`
- `migrations/postgres/2026-05-22-120000_strip_internal_urls/down.sql`
- `migrations/mysql/2026-05-22-120000_strip_internal_urls/up.sql`
- `migrations/mysql/2026-05-22-120000_strip_internal_urls/down.sql`

**Modified files:**
- `crates/podfetch-web/src/url_rewriting.rs` — add `resolve_image_url`; later remove `UrlRewriter`
- `crates/podfetch-web/src/podcast_episode_dto.rs` — helpers + From impls take `server_url`
- `crates/podfetch-web/src/podcast.rs` — `map_podcast_to_dto*` + `build_podfetch_feed` take `server_url`; remove `rewrite_urls` / `with_rewritten_urls`
- `crates/podfetch-web/src/controllers/websocket_controller.rs` — use new API
- `crates/podfetch-web/src/controllers/podcast_controller.rs` — use new API
- `crates/podfetch-web/src/controllers/podcast_episode_controller.rs` — use new API
- `crates/podfetch-web/src/controllers/watch_time_controller.rs` — use new API
- `crates/podfetch-web/src/controllers/settings_controller.rs` — use new API
- `crates/podfetch-web/src/controllers/controller_utils.rs` — helpers take `server_url`
- `crates/podfetch-web/src/audiobookshelf_api/controllers/podcasts.rs:131` — use header-resolved URL
- `crates/podfetch-web/src/api_file_access.rs:25` — take `server_url` parameter
- `crates/podfetch-web/src/usecases/podcast_episode/mod.rs:535,588` — stop persisting internal URLs
- `crates/podfetch-web/src/services/download/service.rs:45-47` — detect empty string as "default image"
- `crates/common-infrastructure/src/config.rs` — remove `INTERNAL_SERVER_URL`, `server_url`, `get_server_url`, `build_url_to_rss_feed`; `ConfigModel.rss_feed`/`server_url` populated from header-resolved value at `get_config` call site

---

## Pre-flight

Run the existing test suite to establish a green baseline before any changes:

```bash
cargo test -p podfetch-web --lib --features sqlite 2>&1 | tail -5
```

Expected: `test result: ok. 395 passed; 0 failed`

If any tests are red here, stop and investigate before starting the plan.

---

## Phase 1: Foundation

### Task 1.1: DB migration that strips internal URLs

**Files:**
- Create: `migrations/sqlite/2026-05-22-120000_strip_internal_urls/up.sql`
- Create: `migrations/sqlite/2026-05-22-120000_strip_internal_urls/down.sql`
- Create: `migrations/postgres/2026-05-22-120000_strip_internal_urls/up.sql`
- Create: `migrations/postgres/2026-05-22-120000_strip_internal_urls/down.sql`
- Create: `migrations/mysql/2026-05-22-120000_strip_internal_urls/up.sql`
- Create: `migrations/mysql/2026-05-22-120000_strip_internal_urls/down.sql`

- [ ] **Step 1: Create up.sql (same content for all three backends)**

Content for `migrations/{sqlite,postgres,mysql}/2026-05-22-120000_strip_internal_urls/up.sql`:

```sql
UPDATE podcasts
SET image_url = ''
WHERE image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);

UPDATE podcasts
SET original_image_url = ''
WHERE original_image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);

UPDATE podcast_episodes
SET image_url = ''
WHERE image_url IN (
    'http://localhost:8000/ui/default.jpg',
    'http://localhost:8080/ui/default.jpg'
);
```

- [ ] **Step 2: Create down.sql (no-op for all three)**

Content for each `down.sql`:

```sql
-- No-op: stripping a hardcoded fallback to empty cannot be meaningfully reversed
-- because the original empty/non-default state is now indistinguishable.
SELECT 1;
```

(SQLite/Postgres tolerate `SELECT 1;` as a valid statement; MySQL likewise. Diesel just needs the file to exist with at least one valid statement.)

- [ ] **Step 3: Verify migration applies on a fresh SQLite DB**

```bash
cd C:/Users/samue/RustroverProjects/PodFetch
cargo test -p podfetch-web --lib --features sqlite controllers::websocket_controller::tests::test_get_rss_feed_for_existing_podcast_returns_xml_or_forbidden 2>&1 | tail -5
```

Expected: `test result: ok` — the test bootstraps a fresh DB and applies all migrations; if the new migration's SQL is malformed the bootstrap fails.

- [ ] **Step 4: Commit**

```bash
git add migrations/sqlite/2026-05-22-120000_strip_internal_urls migrations/postgres/2026-05-22-120000_strip_internal_urls migrations/mysql/2026-05-22-120000_strip_internal_urls
git commit -m "db: migration to strip internal default-image URLs"
```

---

### Task 1.2: Add `resolve_image_url` helper

**Files:**
- Modify: `crates/podfetch-web/src/url_rewriting.rs` (append the helper near top, with tests in the existing `#[cfg(test)] mod tests` block)

- [ ] **Step 1: Write failing tests for the helper**

Insert at the end of the `#[cfg(test)] mod tests { ... }` block in `crates/podfetch-web/src/url_rewriting.rs`, just before the closing `}` of the module:

```rust
    #[test]
    fn resolve_image_url_empty_stored_returns_default() {
        let result = resolve_image_url("", "https://example.com/");
        assert_eq!(result, "https://example.com/ui/default.jpg");
    }

    #[test]
    fn resolve_image_url_absolute_http_passes_through() {
        let result = resolve_image_url("http://remote.example/cover.jpg", "https://example.com/");
        assert_eq!(result, "http://remote.example/cover.jpg");
    }

    #[test]
    fn resolve_image_url_absolute_https_passes_through() {
        let result = resolve_image_url("https://remote.example/cover.jpg", "https://example.com/");
        assert_eq!(result, "https://remote.example/cover.jpg");
    }

    #[test]
    fn resolve_image_url_relative_path_gets_prefixed() {
        let result = resolve_image_url("podcasts/foo/image.jpg", "https://example.com/");
        assert_eq!(result, "https://example.com/podcasts/foo/image.jpg");
    }

    #[test]
    fn resolve_image_url_leading_slash_stripped_before_prefix() {
        let result = resolve_image_url("/podcasts/foo/image.jpg", "https://example.com/");
        assert_eq!(result, "https://example.com/podcasts/foo/image.jpg");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p podfetch-web --lib --features sqlite url_rewriting::tests::resolve_image_url 2>&1 | tail -10
```

Expected: 5 failures with "cannot find function `resolve_image_url` in this scope".

- [ ] **Step 3: Implement `resolve_image_url`**

Add to `crates/podfetch-web/src/url_rewriting.rs` (just above the `#[cfg(test)] mod tests` block), and import the constant at the top of the file:

At the top of the file, change the existing `use common_infrastructure::runtime::ENVIRONMENT_SERVICE;` line to:

```rust
use common_infrastructure::runtime::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
```

Then add the function:

```rust
/// Resolve a stored image URL into a client-facing URL.
///
/// - Empty string → default image at `<server_url>/ui/default.jpg`.
/// - Absolute http/https → passthrough (external remote URL).
/// - Relative path → `<server_url>/<path>` (leading slash stripped before joining).
pub fn resolve_image_url(stored: &str, server_url: &str) -> String {
    let base = normalize_server_url(server_url);
    if stored.is_empty() {
        return format!("{base}{DEFAULT_IMAGE_URL}");
    }
    if stored.starts_with("http://") || stored.starts_with("https://") {
        return stored.to_string();
    }
    format!("{base}{}", stored.trim_start_matches('/'))
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p podfetch-web --lib --features sqlite url_rewriting::tests::resolve_image_url 2>&1 | tail -10
```

Expected: `test result: ok. 5 passed`

- [ ] **Step 5: Run the full url_rewriting tests to confirm no regressions**

```bash
cargo test -p podfetch-web --lib --features sqlite url_rewriting 2>&1 | tail -5
```

Expected: all url_rewriting tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/url_rewriting.rs
git commit -m "feat(url): add resolve_image_url helper for empty/absolute/relative cases"
```

---

## Phase 2: Refactor URL-building helpers to take `server_url`

### Task 2.1: `podcast_episode_dto.rs` helpers take `server_url`

The file currently has these private helpers that read `ENVIRONMENT_SERVICE.server_url`:
- `map_file_url(url, remote_url, user)` — line ~206
- `map_local_file_url_with_api_key(url, remote_url, api_key)` — line ~141
- `map_url(episode, local_url, remote_url, user, type)` — line ~169

And the two `From` impls (lines 39-119) which call those helpers. We refactor the helpers first.

**Files:**
- Modify: `crates/podfetch-web/src/podcast_episode_dto.rs`

- [ ] **Step 1: Change `map_file_url` signature to take `server_url: &str`**

Replace the function (currently at line ~206):

```rust
pub fn map_file_url(
    url: &Option<String>,
    remote_url: &str,
    user: &Option<User>,
    server_url: &str,
) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            url_encoded = format!("{server_url}{url_encoded}");

            match ENVIRONMENT_SERVICE.any_auth_enabled {
                true => match &user {
                    None => url_encoded,
                    Some(user) => match &user.api_key {
                        None => url_encoded,
                        Some(key) => format!("{}{}{}", url_encoded, "?apiKey=", key),
                    },
                },
                false => url_encoded,
            }
        }
        None => remote_url.to_string(),
    }
}
```

- [ ] **Step 2: Change `map_local_file_url_with_api_key` signature**

Replace (currently at line ~141):

```rust
pub fn map_local_file_url_with_api_key(
    url: &Option<String>,
    remote_url: &str,
    api_key: &Option<String>,
    server_url: &str,
) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            let urlencoded = url_encoded.clone();
            url_encoded = server_url.to_owned();
            url_encoded.push_str(&urlencoded);

            match ENVIRONMENT_SERVICE.any_auth_enabled {
                true => match &api_key {
                    None => url_encoded,
                    Some(api_key) => format!("{}{}{}", url_encoded, "?apiKey=", api_key),
                },
                false => url_encoded,
            }
        }
        None => remote_url.to_string(),
    }
}
```

- [ ] **Step 3: Change `map_url` signature**

Replace (currently at line ~169):

```rust
fn map_url(
    episode: &PodcastEpisode,
    local_url: &Option<String>,
    remote_url: &str,
    user: &Option<User>,
    r#type: FileType,
    server_url: &str,
) -> String {
    match &episode.download_location {
        Some(location) => {
            let handle = FileHandlerType::from(location.as_str());
            match handle {
                FileHandlerType::Local => map_file_url(local_url, remote_url, user, server_url),
                FileHandlerType::S3 => map_s3_url(local_url, remote_url),
            }
        }
        None => match r#type {
            FileType::Image => remote_url.to_string(),
            FileType::Episode => {
                let mut url = url::Url::from_str(&format!("{server_url}proxy/podcast"))
                    .unwrap();
                if ENVIRONMENT_SERVICE.any_auth_enabled
                    && let Some(user) = user
                    && let Some(key) = &user.api_key
                {
                    url.query_pairs_mut().append_pair("apiKey", key);
                }
                url.query_pairs_mut()
                    .append_pair("episodeId", &episode.episode_id);
                url.to_string()
            }
        },
    }
}
```

- [ ] **Step 4: Change `map_file_url_with_api_key` (intermediate dispatcher) signature**

Replace (currently at line ~121):

```rust
fn map_file_url_with_api_key(
    podcast_episode: &PodcastEpisode,
    local_url: &Option<String>,
    remote_url: &str,
    api_key: &Option<String>,
    server_url: &str,
) -> String {
    match &podcast_episode.download_location {
        Some(location) => {
            let handle = FileHandlerType::from(location.as_str());
            match handle {
                FileHandlerType::Local => {
                    map_local_file_url_with_api_key(local_url, remote_url, api_key, server_url)
                }
                FileHandlerType::S3 => map_s3_url(local_url, remote_url),
            }
        }
        None => remote_url.to_string(),
    }
}
```

- [ ] **Step 5: Replace `From` impls with named constructors**

Delete both `From` impl blocks (lines ~39-119) and replace with:

```rust
impl PodcastEpisodeDto {
    pub fn from_episode_with_user(
        episode: PodcastEpisode,
        user: Option<User>,
        favorite: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self {
        let local_url = map_url(
            &episode,
            &episode.file_episode_path,
            &episode.url,
            &user,
            FileType::Episode,
            server_url,
        );
        let local_image_url = resolve_image_url(
            &map_url(
                &episode,
                &episode.file_image_path,
                &episode.image_url,
                &user,
                FileType::Image,
                server_url,
            ),
            server_url,
        );
        PodcastEpisodeDto {
            id: episode.id,
            podcast_id: episode.podcast_id,
            episode_id: episode.episode_id.to_string(),
            name: episode.name.to_string(),
            url: episode.url.clone(),
            date_of_recording: episode.date_of_recording.to_string(),
            image_url: episode.image_url.clone(),
            total_time: episode.total_time,
            local_url,
            local_image_url,
            description: episode.description.to_string(),
            download_time: episode.download_time,
            guid: episode.guid.to_string(),
            deleted: episode.deleted,
            episode_numbering_processed: episode.episode_numbering_processed,
            favored: favorite.map(|f| f.favorite),
            status: episode.is_downloaded(),
        }
    }

    pub fn from_episode_with_api_key(
        episode: PodcastEpisode,
        api_key: Option<String>,
        favorite: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self {
        let local_url = map_file_url_with_api_key(
            &episode,
            &episode.file_episode_path,
            &episode.url,
            &api_key,
            server_url,
        );
        let local_image_url = resolve_image_url(
            &map_file_url_with_api_key(
                &episode,
                &episode.file_image_path,
                &episode.image_url,
                &api_key,
                server_url,
            ),
            server_url,
        );
        PodcastEpisodeDto {
            id: episode.id,
            podcast_id: episode.podcast_id,
            episode_id: episode.episode_id.to_string(),
            name: episode.name.to_string(),
            url: episode.url.clone(),
            date_of_recording: episode.date_of_recording.to_string(),
            image_url: episode.image_url.clone(),
            total_time: episode.total_time,
            local_url,
            local_image_url,
            description: episode.description.to_string(),
            status: episode.is_downloaded(),
            download_time: episode.download_time,
            guid: episode.guid,
            deleted: episode.deleted,
            episode_numbering_processed: episode.episode_numbering_processed,
            favored: favorite.map(|f| f.favorite),
        }
    }
}
```

Add this import to the top of the file:

```rust
use crate::url_rewriting::resolve_image_url;
```

And remove (top of file):

```rust
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
```

(Keep `FileHandlerType` import; remove only the `ENVIRONMENT_SERVICE` import.)

Wait — `ENVIRONMENT_SERVICE.any_auth_enabled` is still used inside `map_url`, `map_file_url`, `map_local_file_url_with_api_key`. Keep the import. Only the `ENVIRONMENT_SERVICE.server_url` reads are gone.

- [ ] **Step 6: Build the crate to find all callers of the old `From` impls**

```bash
cargo build -p podfetch-web --features sqlite 2>&1 | tail -40
```

Expected: compile errors at every site that uses `(episode, user, fav).into()` or `(episode, api_key, fav).into()`. Record each error site — these are the call sites updated in Phase 3 and Phase 4.

(Do NOT fix yet; this build failure is expected and informational.)

- [ ] **Step 7: Commit (with broken build acknowledged in the commit message)**

```bash
git add crates/podfetch-web/src/podcast_episode_dto.rs
git commit -m "refactor(dto): PodcastEpisodeDto takes server_url via named constructors

Build is intentionally broken at this commit; callers are updated in
follow-up commits."
```

---

### Task 2.2: `podcast.rs` mapping functions take `server_url`

**Files:**
- Modify: `crates/podfetch-web/src/podcast.rs`

- [ ] **Step 1: Refactor `map_podcast_to_dto` to take `server_url: &str`**

Replace the function (currently at line ~151):

```rust
pub fn map_podcast_to_dto(value: Podcast, server_url: &str) -> PodcastDto {
    let image_url = resolve_image_url(&value.image_url, server_url);
    let keywords = dedupe_keywords(value.keywords.clone());
    let podfetch_rss_feed = build_podfetch_feed(value.id, None, server_url);

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        language: value.language.clone(),
        keywords,
        podfetch_feed: podfetch_rss_feed,
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags: vec![],
        favorites: false,
    }
}
```

- [ ] **Step 2: Refactor `map_podcast_with_context_to_dto` to take `server_url: &str`**

Replace (currently at line ~181):

```rust
pub fn map_podcast_with_context_to_dto(
    value: Podcast,
    favorite: Option<bool>,
    tags: Vec<Tag>,
    user: &User,
    server_url: &str,
) -> PodcastDto {
    let image_url = match resolve_file_handler_type(value.download_location.clone()) {
        FileHandlerType::Local => resolve_image_url(&value.image_url, server_url),
        FileHandlerType::S3 => {
            format!(
                "{}/{}",
                ENVIRONMENT_SERVICE.s3_config.endpoint.clone(),
                &value.image_url
            )
        }
    };

    PodcastDto {
        id: value.id,
        name: value.name.clone(),
        directory_id: value.directory_id.clone(),
        rssfeed: value.rssfeed.clone(),
        image_url,
        podfetch_feed: build_podfetch_feed(value.id, user.api_key.as_deref(), server_url),
        language: value.language.clone(),
        keywords: dedupe_keywords(value.keywords.clone()),
        summary: value.summary.clone(),
        explicit: value.explicit.clone(),
        last_build_date: value.last_build_date.clone(),
        author: value.author.clone(),
        active: value.active,
        original_image_url: value.original_image_url.clone(),
        directory_name: value.directory_name.clone(),
        tags,
        favorites: favorite.unwrap_or(false),
    }
}
```

- [ ] **Step 3: Refactor `build_podfetch_feed` to take `server_url: &str`**

Replace (currently at line ~243):

```rust
fn build_podfetch_feed(podcast_id: i32, api_key: Option<&str>, server_url: &str) -> String {
    let base = crate::url_rewriting::normalize_server_url(server_url);
    let mut url = url::Url::parse(&format!("{base}rss"))
        .expect("server_url must be a valid base URL");
    url.path_segments_mut()
        .expect("rss feed base URL must be a hierarchical scheme")
        .push(&podcast_id.to_string());
    if let Some(api_key) = api_key {
        url.query_pairs_mut().append_pair("apiKey", api_key);
    }
    url.to_string()
}
```

- [ ] **Step 4: Delete `PodcastDto::rewrite_urls` and `with_rewritten_urls` methods**

Delete the `impl PodcastDto { ... }` block at lines ~134-149:

```rust
impl PodcastDto {
    pub fn rewrite_urls(&mut self, rewriter: &UrlRewriter) { ... }
    pub fn with_rewritten_urls(mut self, rewriter: &UrlRewriter) -> Self { ... }
}
```

Also remove the `use crate::url_rewriting::UrlRewriter;` import line at the top.

Add the import for the new helper:

```rust
use crate::url_rewriting::resolve_image_url;
```

The existing build_podfetch_feed test (added during issue #2079) needs updating. Replace the existing tests block at the bottom of `podcast.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::build_podfetch_feed;

    #[test]
    fn build_podfetch_feed_appends_podcast_id_to_rss_path() {
        let url = build_podfetch_feed(42, None, "http://localhost:8000/");
        assert!(
            url.ends_with("/rss/42"),
            "expected per-podcast feed URL to end with /rss/42, got: {url}"
        );
    }

    #[test]
    fn build_podfetch_feed_appends_api_key_query_param() {
        let url = build_podfetch_feed(42, Some("secret-key"), "http://localhost:8000/");
        assert!(
            url.contains("/rss/42"),
            "expected per-podcast feed URL to contain /rss/42, got: {url}"
        );
        assert!(
            url.contains("apiKey=secret-key"),
            "expected url to contain apiKey query param, got: {url}"
        );
    }

    #[test]
    fn build_podfetch_feed_uses_provided_server_url() {
        let url = build_podfetch_feed(7, None, "https://podfetch.example.com/");
        assert_eq!(url, "https://podfetch.example.com/rss/7");
    }
}
```

- [ ] **Step 5: Remove the `get_server_url()` call at line 154 and 191**

In `map_podcast_to_dto` (now refactored), the `image_url` line at the old line 152-156 used `ENVIRONMENT_SERVICE.get_server_url()` for the format!. This is replaced by `resolve_image_url(&value.image_url, server_url)` per step 1 above. Verify no `ENVIRONMENT_SERVICE.get_server_url()` call remains in this file:

```bash
grep -n "ENVIRONMENT_SERVICE" crates/podfetch-web/src/podcast.rs
```

Expected output: only references to `ENVIRONMENT_SERVICE.s3_config.endpoint` (kept) inside `map_podcast_with_context_to_dto`. No `server_url`/`get_server_url`/`build_url_to_rss_feed` references.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/podcast.rs
git commit -m "refactor(podcast): map_podcast_to_dto* + build_podfetch_feed take server_url

Build is still broken; callers updated in follow-up commits."
```

---

## Phase 3: Update controllers to pass `server_url`

Each controller below resolves `server_url = resolve_server_url_from_headers(&headers)` once at the top of each handler and threads `&server_url` into every DTO/mapping call. All `rewriter.rewrite_in_place(...)` and `create_url_rewriter(...)` calls in these files are deleted.

### Task 3.1: `controllers/websocket_controller.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/websocket_controller.rs`

- [ ] **Step 1: Replace the episode mapping in `get_rss_feed`**

Find the block (added during #2081 fix):

```rust
let api_key = api_key.and_then(|c| c.api_key);
let rewriter = create_url_rewriter(&headers);

let downloaded_episodes: Vec<PodcastEpisodeDto> = downloaded_episodes
    .into_iter()
    .map(|c| {
        let mut dto: PodcastEpisodeDto =
            (c, api_key.clone(), None::<FavoritePodcastEpisode>).into();
        rewriter.rewrite_in_place(&mut dto.local_url);
        rewriter.rewrite_in_place(&mut dto.local_image_url);
        dto
    })
    .collect();
```

Replace with:

```rust
let api_key = api_key.and_then(|c| c.api_key);

let downloaded_episodes: Vec<PodcastEpisodeDto> = downloaded_episodes
    .into_iter()
    .map(|c| {
        PodcastEpisodeDto::from_episode_with_api_key(
            c,
            api_key.clone(),
            None::<FavoritePodcastEpisode>,
            &server_url,
        )
    })
    .collect();
```

- [ ] **Step 2: Replace the episode mapping in `get_rss_feed_for_podcast`**

Same shape, just below `let podcast = PodcastService::get_podcast(id)?;`. Replace the block with the rewriter and `.into()`:

```rust
let api_key = api_key.and_then(|c| c.api_key);
let podcast = PodcastService::get_podcast(id)?;

let downloaded_episodes: Vec<PodcastEpisodeDto> =
    PodcastEpisodeService::find_all_downloaded_podcast_episodes_by_podcast_id(id)?
        .into_iter()
        .map(|c| {
            PodcastEpisodeDto::from_episode_with_api_key(
                c,
                api_key.clone(),
                None::<FavoritePodcastEpisode>,
                &server_url,
            )
        })
        .collect();
```

- [ ] **Step 3: Update the import**

At the top of the file, change:

```rust
use crate::url_rewriting::{create_url_rewriter, resolve_server_url_from_headers};
```

back to:

```rust
use crate::url_rewriting::resolve_server_url_from_headers;
```

- [ ] **Step 4: Run the file's tests**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::websocket_controller 2>&1 | tail -10
```

Expected: all tests in `websocket_controller::tests` pass — including `test_get_rss_feed_for_podcast_rewrites_episode_urls_using_forwarded_headers` from the #2081 fix.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/controllers/websocket_controller.rs
git commit -m "refactor(rss): use named PodcastEpisodeDto constructor, drop rewriter"
```

---

### Task 3.2: `controllers/podcast_episode_controller.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/podcast_episode_controller.rs`

This file has three call sites (lines ~103, ~153, ~217) that currently look like:

```rust
let mut mapped_podcast_episode: PodcastEpisodeDto =
    (podcast_inner, Some(requester), None).into();
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
```

- [ ] **Step 1: Replace site at ~line 103 (`get_podcast_episode_by_id`)**

Replace the block:

```rust
let mut mapped_podcast_episode: PodcastEpisodeDto =
    (podcast_inner, Some(requester), None).into();
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
mapped_podcast_episode
```

with:

```rust
PodcastEpisodeDto::from_episode_with_user(podcast_inner, Some(requester), None, &server_url)
```

And replace `let rewriter = create_url_rewriter(&headers);` at the top of `get_podcast_episode_by_id` with:

```rust
let server_url = resolve_server_url_from_headers(&headers);
```

- [ ] **Step 2: Replace site at ~line 153 (`find_all_podcast_episodes_of_podcast`)**

Replace `let rewriter = create_url_rewriter(&headers);` with:

```rust
let server_url = resolve_server_url_from_headers(&headers);
```

Replace the per-episode mapping block:

```rust
let mut mapped_podcast_episode: PodcastEpisodeDto =
    (podcast_inner.0, Some(user.clone()), podcast_inner.2).into();
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
```

with:

```rust
let mapped_podcast_episode = PodcastEpisodeDto::from_episode_with_user(
    podcast_inner.0,
    Some(user.clone()),
    podcast_inner.2,
    &server_url,
);
```

- [ ] **Step 3: Replace site at ~line 217 (the `get_timeline` handler)**

This site is structurally different from Steps 1-2: at line 217, both `mapped_podcast_episode: PodcastEpisodeDto` and `podcast_extracted: PodcastDto` are **already DTOs**, built upstream by `TimelineItem::get_timeline(requester, favored_only)`. The current code rewrites them post-hoc:

```rust
let mut mapped_podcast_episode = podcast_episode;
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_url);
rewriter.rewrite_in_place(&mut mapped_podcast_episode.local_image_url);
let mapped_podcast = podcast_extracted.with_rewritten_urls(&rewriter);
```

We push `server_url` upstream into `TimelineItem::get_timeline` instead:

1. Open the file that defines `TimelineItem::get_timeline` (search: `grep -rn "fn get_timeline" crates/`). Add a `server_url: &str` parameter and thread it into the DTO construction (which will use the new `from_episode_with_user` constructor and `map_podcast_with_context_to_dto(..., server_url)`).
2. Back in this handler at line 208, change:

```rust
let res = TimelineItem::get_timeline(requester, favored_only)?;
let rewriter = create_url_rewriter(&headers);
```

to:

```rust
let server_url = resolve_server_url_from_headers(&headers);
let res = TimelineItem::get_timeline(requester, favored_only, &server_url)?;
```

3. Replace the closure body (lines 214-228) with:

```rust
.map(|podcast_episode| {
    let (mapped_podcast_episode, mapped_podcast, history, favorite) =
        podcast_episode.clone();
    TimeLinePodcastEpisode {
        podcast_episode: mapped_podcast_episode,
        podcast: mapped_podcast,
        history,
        favorite,
    }
})
```

(No more `rewrite_in_place` / `with_rewritten_urls` — the upstream now builds with the right server_url.)

- [ ] **Step 4: Update imports**

Remove `create_url_rewriter` from the import list; ensure `resolve_server_url_from_headers` is present.

- [ ] **Step 5: Run the file's tests**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::podcast_episode_controller 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/controllers/podcast_episode_controller.rs
git commit -m "refactor(episodes): use named PodcastEpisodeDto constructor, drop rewriter"
```

---

### Task 3.3: `controllers/podcast_controller.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/podcast_controller.rs`

This file has one `rewrite_in_place` site (~line 444) for episodes, plus calls to `map_podcast_to_dto` / `map_podcast_with_context_to_dto`, plus the default-image fallback at lines 623, 625.

- [ ] **Step 1: At every handler that builds a `PodcastDto` or `PodcastEpisodeDto`, resolve `server_url` once and pass it**

For each handler that takes `headers: HeaderMap`:

```rust
let server_url = resolve_server_url_from_headers(&headers);
```

Then for each call site:

- `map_podcast_to_dto(podcast)` → `map_podcast_to_dto(podcast, &server_url)`
- `map_podcast_with_context_to_dto(podcast, fav, tags, &user)` → `... &server_url)`
- `(episode, user, fav).into()` → `PodcastEpisodeDto::from_episode_with_user(episode, user, fav, &server_url)`

Run `cargo build -p podfetch-web --features sqlite 2>&1 | grep -E "podcast_controller.rs:[0-9]+" | sort -u` to enumerate the exact call-site lines after Task 2.x; update each.

For each `rewriter.rewrite_in_place(...)` line in this file: delete it (the new constructors already write the correct URL).

For `let rewriter = create_url_rewriter(&headers);`: delete it (replaced by `let server_url = resolve_server_url_from_headers(&headers);`).

- [ ] **Step 2: Fix the default-image fallback at lines 623, 625**

Find the block (around line 615-630):

```rust
ENVIRONMENT_SERVICE.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
```

and

```rust
ENVIRONMENT_SERVICE.server_url.clone().to_owned() + "ui/default.jpg"
```

Replace both with:

```rust
resolve_image_url("", &server_url)
```

Add `use crate::url_rewriting::resolve_image_url;` to the imports if not already present.

If the surrounding handler doesn't already have `let server_url = resolve_server_url_from_headers(&headers);` and `headers: HeaderMap` in its signature, add both. (This handler emits image URLs to clients, so it must take headers.)

- [ ] **Step 3: Update imports**

```rust
use crate::url_rewriting::{resolve_image_url, resolve_server_url_from_headers};
```

Drop `create_url_rewriter` and `UrlRewriter` if imported.

- [ ] **Step 4: Run the file's tests**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::podcast_controller 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/controllers/podcast_controller.rs
git commit -m "refactor(podcasts): use server_url parameter, drop rewriter and env fallback"
```

---

### Task 3.4: `controllers/watch_time_controller.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/watch_time_controller.rs`

Same pattern as 3.2. The file has one rewrite site at lines ~65-66.

- [ ] **Step 1: Replace `create_url_rewriter` with `resolve_server_url_from_headers`**

Replace:

```rust
let rewriter = create_url_rewriter(&headers);
```

with:

```rust
let server_url = resolve_server_url_from_headers(&headers);
```

- [ ] **Step 2: Replace the closure body**

The current `get_last_watched` closure (lines 60-70) is:

```rust
let rewriter = create_url_rewriter(&headers);
let episodes = watchtime::get_last_watched(state.watchtime_service.as_ref(), &requester)
    .map_err(map_watchtime_error)?
    .into_iter()
    .map(|mut item| {
        rewriter.rewrite_in_place(&mut item.podcast_episode.local_url);
        rewriter.rewrite_in_place(&mut item.podcast_episode.local_image_url);
        item.podcast.rewrite_urls(&rewriter);
        item
    })
    .collect();
```

`item.podcast_episode` is a `PodcastEpisodeDto` and `item.podcast` is a `PodcastDto`, both built upstream by `watchtime::get_last_watched`. Push `server_url` upstream:

1. Find `fn get_last_watched` (search: `grep -rn "fn get_last_watched" crates/`). Add `server_url: &str` parameter and thread it into the DTO constructors there (use `PodcastEpisodeDto::from_episode_with_user(..., server_url)` and `map_podcast_with_context_to_dto(..., server_url)`).
2. Back in this controller, replace the block with:

```rust
let server_url = resolve_server_url_from_headers(&headers);
let episodes = watchtime::get_last_watched(
    state.watchtime_service.as_ref(),
    &requester,
    &server_url,
)
.map_err(map_watchtime_error)?
.into_iter()
.collect();
```

- [ ] **Step 3: Update imports**

Remove `create_url_rewriter` / `UrlRewriter`; ensure `resolve_server_url_from_headers` is imported.

- [ ] **Step 4: Run the file's tests**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::watch_time_controller 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/controllers/watch_time_controller.rs
git commit -m "refactor(watchtime): use server_url parameter, drop rewriter"
```

---

### Task 3.5: `controllers/settings_controller.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/settings_controller.rs`

This file imports `resolve_server_url_from_headers` (line 170) so likely uses it already. Verify no remaining `ENVIRONMENT_SERVICE.server_url` references; if any, replace with the local `server_url` variable.

- [ ] **Step 1: Inspect the file**

```bash
grep -n "ENVIRONMENT_SERVICE.server_url\|server_url\|create_url_rewriter\|UrlRewriter" crates/podfetch-web/src/controllers/settings_controller.rs
```

- [ ] **Step 2: For each `ENVIRONMENT_SERVICE.server_url` reference**

Replace with the locally-resolved `server_url` (already present in handlers that use it for OPML export at line 170). If a handler doesn't currently resolve it, add `let server_url = resolve_server_url_from_headers(&headers);` (and ensure `headers: HeaderMap` is in the signature).

- [ ] **Step 3: Run the file's tests**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::settings_controller 2>&1 | tail -10
```

- [ ] **Step 4: Commit if changes were made**

```bash
git add crates/podfetch-web/src/controllers/settings_controller.rs
git commit -m "refactor(settings): use header-resolved server_url everywhere"
```

(Skip the commit if no changes were needed — file was already clean.)

---

### Task 3.6: `controllers/controller_utils.rs`

**Files:**
- Modify: `crates/podfetch-web/src/controllers/controller_utils.rs`

- [ ] **Step 1: Refactor `unwrap_string_audio` to take `server_url: &str`**

Replace the file contents:

```rust
use common_infrastructure::runtime::DEFAULT_IMAGE_URL;
use serde_json::Value;

use crate::url_rewriting::normalize_server_url;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value, server_url: &str) -> String {
    match value.to_string().is_empty() {
        true => format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url)),
        false => value.to_string().replace('\"', ""),
    }
}

pub fn get_default_image(server_url: &str) -> String {
    format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url))
}
```

- [ ] **Step 2: Find all callers of these two functions and update**

```bash
grep -rn "unwrap_string_audio\|get_default_image" crates/podfetch-web/src/ | grep -v controller_utils.rs
```

For each caller, ensure `server_url` is available (from `resolve_server_url_from_headers(&headers)`) and pass it through.

- [ ] **Step 3: Build to confirm**

```bash
cargo build -p podfetch-web --features sqlite 2>&1 | grep -E "error\[" | head -20
```

Expected: no errors (or only errors in files we haven't refactored yet — those get fixed in later tasks).

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/controllers/controller_utils.rs <other-files-touched>
git commit -m "refactor(utils): controller_utils helpers take server_url"
```

---

### Task 3.7: `audiobookshelf_api/controllers/podcasts.rs:131`

**Files:**
- Modify: `crates/podfetch-web/src/audiobookshelf_api/controllers/podcasts.rs`

- [ ] **Step 1: Replace the default-image fallback**

Find lines ~130-133:

```rust
.unwrap_or_else(|| {
    ENVIRONMENT_SERVICE.server_url.clone()
        + common_infrastructure::runtime::DEFAULT_IMAGE_URL
});
```

Replace with:

```rust
.unwrap_or_else(|| crate::url_rewriting::resolve_image_url("", &server_url));
```

Confirm the enclosing handler has `let server_url = resolve_server_url_from_headers(&headers);` available. If not, add it (and ensure `headers: HeaderMap` is in the handler signature).

- [ ] **Step 2: Run the audiobookshelf tests**

```bash
cargo test -p podfetch-web --lib --features sqlite audiobookshelf_api 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/audiobookshelf_api/controllers/podcasts.rs
git commit -m "refactor(audiobookshelf): use resolve_image_url for default fallback"
```

---

### Task 3.8: `api_file_access.rs:25` — resolve from request headers

**Files:**
- Modify: `crates/podfetch-web/src/api_file_access.rs`

`check_permissions_for_files` is axum middleware. It currently passes `&ENVIRONMENT_SERVICE.server_url` into `check_file_access`. Since the middleware receives the full `Request`, headers are reachable via `req.headers()`.

- [ ] **Step 1: Resolve server_url from the request headers**

Replace the function body (lines ~19-37):

```rust
pub async fn check_permissions_for_files(
    State(state): State<AppState>,
    OptionalQuery(query): OptionalQuery<RSSAPiKey>,
    req: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let request = query.and_then(|rss_api_key| rss_api_key.api_key);
    let server_url = crate::url_rewriting::resolve_server_url_from_headers(req.headers());
    check_file_access(
        req.uri().path(),
        request,
        ENVIRONMENT_SERVICE.any_auth_enabled,
        &server_url,
        |api_key| state.user_auth_service.is_api_key_valid(api_key),
        |path| {
            PodcastEpisodeService::get_podcast_episodes_by_url(path)
                .map(|episode| episode.and_then(|e| e.file_image_path))
        },
        |encoded_path| {
            PodcastService::find_podcast_by_image_path(encoded_path)
                .map(|podcast| podcast.is_some())
        },
    )
    .map_err(map_file_access_error)?;
    Ok(next.run(req).await)
}
```

- [ ] **Step 2: Build**

```bash
cargo build -p podfetch-web --features sqlite 2>&1 | grep -E "error\[" | head -10
```

Expected: no errors in this file.

- [ ] **Step 3: Run tests**

```bash
cargo test -p podfetch-web --lib --features sqlite api_file_access 2>&1 | tail -5
```

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/api_file_access.rs
git commit -m "refactor(file_access): middleware resolves server_url from request headers"
```

---

## Phase 4: Background-job cleanup

### Task 4.1: Stop persisting internal URLs in episode insertion

**Files:**
- Modify: `crates/podfetch-web/src/usecases/podcast_episode/mod.rs`

- [ ] **Step 1: Update line ~535 (episode-image fallback)**

Find:

```rust
let mut image_url = if !podcast.original_image_url.is_empty() {
    podcast.original_image_url.clone()
} else {
    format!("{}{}", ENVIRONMENT_SERVICE.server_url, DEFAULT_IMAGE_URL)
};
```

Replace with:

```rust
let mut image_url = if !podcast.original_image_url.is_empty() {
    podcast.original_image_url.clone()
} else {
    // No image available; persist empty so the response layer substitutes
    // the default at read time from request headers.
    String::new()
};
```

- [ ] **Step 2: Update `handle_podcast_image_insert` (around line 588)**

Find:

```rust
None => {
    let url = ENVIRONMENT_SERVICE.server_url.clone().to_owned() + DEFAULT_IMAGE_URL;
    crate::services::podcast::service::PodcastService::update_original_image_url(
        &url, podcast.id,
    )?;
}
```

Replace with:

```rust
None => {
    // No channel image; leave original_image_url empty so the response
    // layer substitutes the default at read time.
    crate::services::podcast::service::PodcastService::update_original_image_url(
        "", podcast.id,
    )?;
}
```

- [ ] **Step 3: Remove the `DEFAULT_IMAGE_URL` and `ENVIRONMENT_SERVICE` imports if unused**

```bash
grep -n "DEFAULT_IMAGE_URL\|ENVIRONMENT_SERVICE" crates/podfetch-web/src/usecases/podcast_episode/mod.rs
```

If these names are no longer referenced inside this file, drop them from the `use` statement at line 23.

- [ ] **Step 4: Run the usecase tests**

```bash
cargo test -p podfetch-web --lib --features sqlite usecases::podcast_episode 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/usecases/podcast_episode/mod.rs
git commit -m "refactor(usecase): background jobs persist empty image_url, not internal URL"
```

---

### Task 4.2: Update `services/download/service.rs` default-image detection

**Files:**
- Modify: `crates/podfetch-web/src/services/download/service.rs`

This file at lines 45-47 detects "is this image_url the default placeholder?". After Task 4.1, the new default marker is the empty string. Existing DB rows (post-migration) also have empty strings.

- [ ] **Step 1: Inspect the current detection**

```bash
sed -n '40,55p' crates/podfetch-web/src/services/download/service.rs
```

Expected current shape:

```rust
trimmed == DEFAULT_IMAGE_URL
    || trimmed == format!("/{DEFAULT_IMAGE_URL}")
    || trimmed.ends_with(&format!("/{DEFAULT_IMAGE_URL}"))
```

- [ ] **Step 2: Add `is_empty` to the check**

Replace with:

```rust
trimmed.is_empty()
    || trimmed == DEFAULT_IMAGE_URL
    || trimmed == format!("/{DEFAULT_IMAGE_URL}")
    || trimmed.ends_with(&format!("/{DEFAULT_IMAGE_URL}"))
```

The original three branches stay so that any backup-restored rows that didn't get the migration treatment (e.g. an installation that ran an old version after the migration) are still detected.

- [ ] **Step 3: Add a unit test that empty stored value is treated as default**

Find or create a test for this predicate in the same file. Inside `#[cfg(test)] mod tests { ... }` add:

```rust
#[test]
fn empty_stored_value_is_treated_as_default_image() {
    assert!(super::is_default_image_url("")); // adjust function name to match the file
}
```

(Adjust the function name `is_default_image_url` to whatever the surrounding code calls this check — inspect lines 40-55 and use the actual function name.)

- [ ] **Step 4: Run tests**

```bash
cargo test -p podfetch-web --lib --features sqlite services::download 2>&1 | tail -10
```

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-web/src/services/download/service.rs
git commit -m "fix(download): treat empty image_url as default placeholder"
```

---

## Phase 5: Delete obsolete API

By now the build should be green again. All callers use the new API. The old API surfaces are unreferenced and ready to delete.

### Task 5.1: Delete `UrlRewriter` and helpers from `url_rewriting.rs`

**Files:**
- Modify: `crates/podfetch-web/src/url_rewriting.rs`

- [ ] **Step 1: Delete the obsolete items**

Delete from the file:
- `pub fn rewrite_url(url, old_base, new_base) -> String` (lines ~25-59)
- `fn is_local_path(path) -> bool` (lines ~71-80)
- `fn normalize_relative_local_path(path) -> Option<String>` (lines ~83-95)
- `fn is_local_host(host, old_base) -> bool` (lines ~98-117)
- `pub struct UrlRewriter` and its `impl` block (lines ~159-187)
- `pub fn create_url_rewriter(headers) -> UrlRewriter` (lines ~189-193)

Also delete the tests in `#[cfg(test)] mod tests` block that reference these items (the `test_rewrite_url_*`, `test_url_rewriter_struct`, `test_create_url_rewriter_from_headers` tests). Keep `test_normalize_server_url` and all `resolve_image_url_*` tests added in Task 1.2.

- [ ] **Step 2: Verify the file still compiles**

```bash
cargo build -p podfetch-web --features sqlite 2>&1 | grep -E "error\[" | head -20
```

Expected: zero errors. If any errors mention `UrlRewriter` or `create_url_rewriter`, fix those call sites (they should have been removed in Phase 3 but may have been missed).

- [ ] **Step 3: Run tests**

```bash
cargo test -p podfetch-web --lib --features sqlite url_rewriting 2>&1 | tail -10
```

Expected: only the kept tests pass — no leftover tests reference the deleted items.

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/url_rewriting.rs
git commit -m "refactor: delete UrlRewriter — no remaining callers"
```

---

### Task 5.2: Remove `ENVIRONMENT_SERVICE.server_url` and friends from config

**Files:**
- Modify: `crates/common-infrastructure/src/config.rs`

- [ ] **Step 1: Delete `INTERNAL_SERVER_URL` constant (line 30)**

```rust
// Delete this line:
pub const INTERNAL_SERVER_URL: &str = "http://localhost:8000/";
```

- [ ] **Step 2: Delete `server_url` field from `EnvironmentService`**

In the `pub struct EnvironmentService { ... }` block (~line 122), remove:

```rust
pub server_url: String,
```

In `EnvironmentService::new()` (~line 279), remove:

```rust
let server_url = INTERNAL_SERVER_URL.to_string();
```

and the field initializer `server_url: server_url.clone(),`.

- [ ] **Step 3: Delete `get_server_url` method (~line 403)**

```rust
// Delete this entire method:
pub fn get_server_url(&self) -> String {
    self.server_url.clone()
}
```

- [ ] **Step 4: Delete `build_url_to_rss_feed` method (~line 270)**

```rust
// Delete this entire method:
pub fn build_url_to_rss_feed(&self) -> Url { ... }
```

- [ ] **Step 5: Update `ConfigModel.rss_feed` / `server_url` population in `get_config`**

The `get_config(&self)` method (~line 442) currently fills `rss_feed: self.server_url.clone() + "rss"` and `server_url: self.server_url.clone()`. These need a `server_url: &str` parameter:

Change the method signature:

```rust
pub fn get_config(&self, server_url: &str) -> ConfigModel {
    ConfigModel {
        podindex_configured: !self.podindex_api_key.is_empty()
            && !self.podindex_api_secret.is_empty(),
        rss_feed: format!("{server_url}rss"),
        server_url: server_url.to_string(),
        reverse_proxy: self.reverse_proxy,
        basic_auth: self.http_basic,
        oidc_configured: self.oidc_configured,
        oidc_config: self.oidc_config.clone(),
        ws_url: String::new(),
    }
}
```

- [ ] **Step 6: Find and update all callers of `get_config`**

```bash
grep -rn "\.get_config(" crates/ 2>&1 | grep -v target/
```

For each call site (likely in `sys_info_controller.rs`), pass `&resolve_server_url_from_headers(&headers)` as the new argument.

- [ ] **Step 7: Drop unused imports**

```bash
grep -n "INTERNAL_SERVER_URL\|build_url_to_rss_feed\|get_server_url" crates/
```

Expected output: only matches in the deleted file context — no remaining usages anywhere.

- [ ] **Step 8: Run the whole test suite**

```bash
cargo test -p podfetch-web --lib --features sqlite 2>&1 | tail -5
```

Expected: `test result: ok. <N> passed; 0 failed` where N is at least 395 (the pre-flight baseline) plus the new tests for `resolve_image_url` (≥ 5). Some tests that asserted on `localhost:8000` URLs may need their assertions updated — for each failure, update the assertion to use header-resolved hosts or use empty strings where the empty default is now correct.

- [ ] **Step 9: Commit**

```bash
git add crates/common-infrastructure/src/config.rs <other-files-touched>
git commit -m "refactor: remove ENVIRONMENT_SERVICE.server_url and friends"
```

---

### Task 5.3: Remove obsolete `ENVIRONMENT_SERVICE` references in test code

**Files:**
- Modify: any test that asserts against `ENVIRONMENT_SERVICE.server_url` (e.g. `controllers/manifest_controller.rs:124`)

- [ ] **Step 1: Locate remaining references**

```bash
grep -rn "ENVIRONMENT_SERVICE.server_url\|ENVIRONMENT_SERVICE.get_server_url" crates/ 2>&1 | grep -v target/
```

- [ ] **Step 2: For each test that asserts the old field**

Update the assertion to use a literal expected URL or remove the test if it has become tautological. Example for `controllers/manifest_controller.rs:120-126` test `test_get_manifest_direct_handler_without_headers_uses_env_fallback`:

```rust
#[tokio::test]
#[serial]
async fn test_get_manifest_direct_handler_without_headers_uses_empty_fallback() {
    let response = super::get_manifest(HeaderMap::new()).await.unwrap();
    // With no headers, resolve_server_url_from_headers returns empty;
    // start_url ends up as "ui/" (the resolver's empty + the manifest path)
    assert_eq!(response.0.start_url, "ui/");
}
```

(The exact expected value depends on how `resolve_server_url_from_headers` is implemented to handle missing headers — see Task 5.4 next.)

---

### Task 5.4: Decide and implement `resolve_server_url_from_headers` fallback when headers absent

**Files:**
- Modify: `crates/podfetch-web/src/url_rewriting.rs`

The current fallback at line 135 reads `ENVIRONMENT_SERVICE.server_url.clone()` — which is being deleted. We need a fallback for the (extremely rare) case where no Host header is present.

- [ ] **Step 1: Update the fallback**

Replace the line:

```rust
return ENVIRONMENT_SERVICE.server_url.clone();
```

with:

```rust
// HTTP/1.1 requires Host. If somehow none of x-forwarded-host / host /
// :authority are present, return empty — callers building URLs will
// emit relative paths, which is the safest degraded behaviour.
return String::new();
```

- [ ] **Step 2: Remove the now-unused `ENVIRONMENT_SERVICE` import from `url_rewriting.rs`**

The earlier import line:

```rust
use common_infrastructure::runtime::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
```

becomes:

```rust
use common_infrastructure::runtime::DEFAULT_IMAGE_URL;
```

- [ ] **Step 3: Update tests in `url_rewriting.rs` that assumed a non-empty fallback**

```bash
cargo test -p podfetch-web --lib --features sqlite url_rewriting 2>&1 | tail -10
```

For each failing test, either fix the assertion (empty string fallback) or remove if it's now meaningless.

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/url_rewriting.rs
git commit -m "refactor(url): resolve_server_url_from_headers returns empty when no host"
```

---

## Phase 6: Verification

### Task 6.1: Full test suite

- [ ] **Step 1: Run the entire suite**

```bash
cargo test -p podfetch-web --lib --features sqlite 2>&1 | tail -10
```

Expected: `test result: ok. <N> passed; 0 failed`.

- [ ] **Step 2: Confirm no remaining references to removed items**

```bash
grep -rn "ENVIRONMENT_SERVICE.server_url\|ENVIRONMENT_SERVICE.get_server_url\|build_url_to_rss_feed\|INTERNAL_SERVER_URL\|UrlRewriter\|create_url_rewriter\|rewrite_in_place\|rewrite_url\b\|with_rewritten_urls\|rewrite_urls" crates/ 2>&1 | grep -v target/
```

Expected: zero matches (or only matches inside this plan file / spec file).

- [ ] **Step 3: Compile under the postgresql feature too**

```bash
cargo build -p podfetch-web --features postgresql --no-default-features 2>&1 | tail -10
```

Expected: clean build, no errors.

- [ ] **Step 4: Sanity-check the issue #2081 regression test still passes**

```bash
cargo test -p podfetch-web --lib --features sqlite controllers::websocket_controller::tests::test_get_rss_feed_for_podcast_rewrites_episode_urls_using_forwarded_headers 2>&1 | tail -5
```

Expected: PASS.

### Task 6.2: Manual smoke test

- [ ] **Step 1: Boot the app locally**

```bash
cargo run -p podfetch-web --features sqlite
```

- [ ] **Step 2: With curl, request the RSS feed and verify URLs reflect the Host header**

```bash
curl -s -H 'X-Forwarded-Host: public.example.com' -H 'X-Forwarded-Proto: https' \
    'http://localhost:8000/rss/1?apiKey=<key>' | head -c 2000
```

Expected: response contains `https://public.example.com/...` URLs in `<enclosure>` and `<itunes:image>` tags. Zero occurrences of `localhost:8000` in episode-level URLs.

- [ ] **Step 3: Verify default-image substitution**

For a podcast/episode with empty `image_url` in the DB:

```bash
curl -s -H 'X-Forwarded-Host: public.example.com' 'http://localhost:8000/api/v1/podcasts/<id>' | jq .image_url
```

Expected: `"https://public.example.com/ui/default.jpg"`.

- [ ] **Step 4: Final commit (no code changes; documents verification)**

If any test fixes or small adjustments came up during smoke testing, commit them. Otherwise skip.

```bash
git status
```

If clean: done.

---

## Final summary commit

After Phase 6 passes, optionally squash or rebase the branch (operator preference). Push:

```bash
git push -u origin refactor/remove-server-url-from-env-service
```

Then open the PR referencing the spec:

```
Closes #2081 follow-up: complete elimination of ENVIRONMENT_SERVICE.server_url.

Spec: docs/superpowers/specs/2026-05-22-remove-server-url-from-env-service-design.md
Plan: docs/superpowers/plans/2026-05-22-remove-server-url-from-env-service.md
```
