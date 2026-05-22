# Remove `ENVIRONMENT_SERVICE.server_url` — Design

**Date:** 2026-05-22
**Status:** Approved, pending implementation
**Related issue:** [#2081](https://github.com/SamTV12345/PodFetch/issues/2081)

## Problem

`EnvironmentService.server_url` is hardcoded to `http://localhost:8000/` (via the `INTERNAL_SERVER_URL` constant). The codebase uses this value in 17 places across 9 files to construct user-visible URLs — episode `local_url`, episode `local_image_url`, podcast `image_url`, RSS feed URLs, default image fallbacks. Some of these URLs are also persisted to the database.

The downstream effects:

- External clients (e.g. AntennaPod) receive RSS feeds containing `http://localhost:8000` URLs they cannot reach (issue #2081).
- The `UrlRewriter` was added later as a post-hoc workaround: it rewrites internal URLs to the external host on a best-effort basis at response time, using request headers to discover the external host. Each controller must remember to call `rewrite_in_place(...)` on every URL field — and the RSS controllers forgot, hence the bug.
- Database rows accumulate stale absolute URLs (`http://localhost:8000/ui/default.jpg`). When the operator changes their reverse proxy or hostname, the stored URLs go stale.

## Goals

- Eliminate `ENVIRONMENT_SERVICE.server_url` (and `INTERNAL_SERVER_URL`, `get_server_url()`, `build_url_to_rss_feed()`).
- All user-visible URLs are constructed at request time from headers (`X-Forwarded-Host` / `X-Forwarded-Proto` / `Host`), using the existing `resolve_server_url_from_headers()` helper.
- Background jobs (RSS crawl, episode insert) never persist internal URLs — they store either remote URLs (from the source feed) or nothing.
- Existing database rows with `http://localhost:8000/` / `http://localhost:8080/` URLs are migrated to a clean state.
- Delete the `UrlRewriter` machinery — it is no longer needed once URLs are constructed correctly from the start.

## Non-goals

- Audiobookshelf HLS URL construction (different infrastructure path, leave for now).
- The S3 endpoint URL field in `S3Config` (separate config, not `server_url`).
- Migration tooling for downstream consumers that may have cached `http://localhost:8000` URLs (extremely unlikely use case).

## Design

### 1. Removed surface

From `crates/common-infrastructure/src/config.rs`:

- The constant `INTERNAL_SERVER_URL`.
- The field `EnvironmentService.server_url` (and the field with the same name in any derived/sister struct in the file).
- The methods `get_server_url()` and `build_url_to_rss_feed()`.
- Any `rss_feed` field on derived config that was derived from `server_url`.

From `crates/podfetch-web/src/url_rewriting.rs`:

- The `UrlRewriter` struct, including `new()`, `rewrite()`, `rewrite_in_place()`.
- The free functions `create_url_rewriter()` and `rewrite_url()`.
- Helpers used only by the rewriter (`is_local_path`, `is_local_host`, `normalize_relative_local_path`).

What stays in `url_rewriting.rs`:

- `resolve_server_url_from_headers(headers: &HeaderMap) -> String`
- `normalize_server_url(server_url: &str) -> String`
- `get_header_value(...)` helper

The file is renamed to `server_url.rs` (or merged into an existing module) since "rewriting" is no longer the purpose. Final name decided during implementation; consumers of `crate::url_rewriting::resolve_server_url_from_headers` are updated accordingly.

### 2. New URL-construction API

DTOs and mapping functions take `server_url: &str` as an explicit parameter. The existing tuple-based `From` impls on `PodcastEpisodeDto` are replaced by named constructors:

```rust
impl PodcastEpisodeDto {
    pub fn from_episode_with_user(
        ep: PodcastEpisode,
        user: Option<User>,
        fav: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self;

    pub fn from_episode_with_api_key(
        ep: PodcastEpisode,
        api_key: Option<String>,
        fav: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self;
}
```

Same pattern in `crates/podfetch-web/src/podcast.rs`:

```rust
pub fn map_podcast_to_dto(value: Podcast, server_url: &str) -> PodcastDto;
pub fn map_podcast_with_context_to_dto(
    value: Podcast,
    favorite: Option<bool>,
    tags: Vec<Tag>,
    user: &User,
    server_url: &str,
) -> PodcastDto;
```

And `build_podfetch_feed`:

```rust
fn build_podfetch_feed(podcast_id: i32, api_key: Option<&str>, server_url: &str) -> String;
```

The url-encoding helpers (`map_file_url`, `map_file_url_with_api_key`, `map_local_file_url_with_api_key`, `map_url`) likewise take `server_url: &str` and use that instead of reading from `ENVIRONMENT_SERVICE`.

The `PodcastDto::rewrite_urls` / `with_rewritten_urls` methods are removed — the DTO no longer needs to be rewritten after construction.

### 3. Controller pattern

Every controller that builds DTOs follows this pattern:

```rust
let server_url = resolve_server_url_from_headers(&headers);
let dto = PodcastEpisodeDto::from_episode_with_user(ep, user, fav, &server_url);
```

No more `rewriter.rewrite_in_place(...)` calls. Controllers affected (current `rewrite_in_place` sites):

- `controllers/podcast_controller.rs` (1 site, ~line 444)
- `controllers/podcast_episode_controller.rs` (3 sites, ~lines 105, 155, 218)
- `controllers/watch_time_controller.rs` (1 site, ~line 65)
- `controllers/websocket_controller.rs` (2 sites, just added in the #2081 fix — these are removed and replaced with the new constructor pattern)

### 4. Background-job URL handling

In `crates/podfetch-web/src/usecases/podcast_episode/mod.rs`:

- The fallback `format!("{}{}", ENVIRONMENT_SERVICE.server_url, DEFAULT_IMAGE_URL)` at the episode-image fallback site (~line 535) is removed. When the RSS source provides no episode image, `image_url` is stored as the empty string. The response layer substitutes the default at read time.
- `handle_podcast_image_insert`'s `None` branch (~line 588) no longer calls `update_original_image_url` with a default URL. If the source feed has no channel image, `original_image_url` is left empty (or the column is set to `''` explicitly). Substitution happens at read time.

### 5. Default-image substitution at response time

A small helper centralises the "empty means default" rule:

```rust
// in url construction module or a new module
pub fn resolve_image_url(stored: &str, server_url: &str) -> String {
    if stored.is_empty() {
        format!("{server_url}{DEFAULT_IMAGE_URL}")
    } else if stored.starts_with("http://") || stored.starts_with("https://") {
        stored.to_string()
    } else {
        format!("{server_url}{}", stored.trim_start_matches('/'))
    }
}
```

This is called by the DTO constructors / mapping functions when building the image_url field. The same logic applies to both podcast and episode images.

Stored values fall into three cases the helper distinguishes:

1. Empty → substitute default image at `server_url + DEFAULT_IMAGE_URL`.
2. Absolute URL (`http://` / `https://`) → external remote URL, pass through unchanged.
3. Relative path → prepend `server_url`.

### 6. Database migration

New Diesel migration: `db/migrations/2026-05-22-strip-internal-urls/` containing `up.sql` and `down.sql`.

`up.sql` (works under both SQLite and PostgreSQL; if backend-specific SQL is needed, separate files are produced):

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

`down.sql` is intentionally a no-op (or reverts to `http://localhost:8000/ui/default.jpg`) — reversing a normalisation step does not restore semantic meaning, and the rollback case is undefined for this change.

Real remote URLs in those columns (e.g. `https://podcast-cdn.example.com/cover.jpg`) are untouched because the migration only matches the exact internal default values.

### 7. Default-image controllers

The existing controllers/usecases that emit the default-image URL using `ENVIRONMENT_SERVICE.server_url + DEFAULT_IMAGE_URL` (`controllers/controller_utils.rs:10,16`, `controllers/podcast_controller.rs:623,625`) get refactored to use `resolve_image_url("", server_url)` (or be removed if the substitution helper supersedes them entirely). The `DEFAULT_IMAGE_URL = "ui/default.jpg"` constant stays.

### 8. Tests

Test changes track the production changes:

- All existing tests that hand-constructed `UrlRewriter` (e.g. `url_rewriting.rs` unit tests) are removed along with the struct.
- Controller integration tests that asserted `localhost:8000` URLs in responses are updated to pass `x-forwarded-host` headers and assert the resolved host.
- A new unit test for `resolve_image_url` covers the three cases (empty → default, http/s → passthrough, relative → prepend).
- A new migration round-trip test asserts that representative rows (default-internal, default-internal-8080, real-remote, empty) come out of the migration in the expected state.
- The #2081 regression test added in `websocket_controller.rs` stays; it should continue to pass against the new constructors.

## Risks and tradeoffs

- **Diff size**: roughly 12–18 source files plus the migration. Mitigation: keep the change in one branch/PR; sequence the changes so the build stays green at each step (introduce new constructors, switch callers, then delete the old `From` impls + `UrlRewriter`).
- **Signature kaskade**: `PodcastEpisodeDto` and `PodcastDto` mapping functions are used in many controllers; every call site gains a `server_url` parameter. Mitigation: this is the explicit goal of the refactor — explicit threading is preferable to global state, and the call sites already have access to `&headers`.
- **Background jobs and notifications**: `crates/common-infrastructure/src/telegram.rs` does not currently construct URLs, so no fallback is needed there. The invite service already accepts `server_url` as a parameter (no global state). If a future feature needs URLs in background contexts, that feature must take a configured `PUBLIC_URL` env var or accept the design constraint that background-emitted URLs aren't supported.
- **Operator confusion during upgrade**: an admin upgrading PodFetch will see existing default-image rows reset to empty. The default substitution at read time keeps the UX identical, so no user-visible regression — but this is worth a release note.

## Implementation sequencing

Not exhaustive; the writing-plans skill produces the detailed plan. High-level order:

1. Migration: add Diesel migration; run it; verify rows are stripped.
2. New API: add `resolve_image_url` helper and the new constructors (`from_episode_with_user`, `from_episode_with_api_key`, named-constructor variants of `map_podcast_to_dto` / `map_podcast_with_context_to_dto` / `build_podfetch_feed`) alongside the existing impls. Build stays green.
3. Switch call sites: every controller and usecase moves to the new constructors. `rewrite_in_place` calls are removed. Build stays green.
4. Background-job cleanup: `usecases/podcast_episode/mod.rs` stops persisting internal URLs.
5. Delete obsolete API: drop the old `From` impls, `UrlRewriter`, `create_url_rewriter`, `rewrite_url`, `ENVIRONMENT_SERVICE.server_url`, `INTERNAL_SERVER_URL`, `get_server_url`, `build_url_to_rss_feed`. Build stays green.
6. Tests: full suite passes; new tests added for `resolve_image_url` and the migration.

## Open questions

None. All design decisions captured above were taken via explicit user choice during brainstorming on 2026-05-22.
