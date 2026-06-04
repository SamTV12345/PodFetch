# SponsorBlock Integration — Design

**Date:** 2026-06-04
**Issue:** [#601 — SponsorBlock integration](https://github.com/SamTV12345/PodFetch/issues/601)
**Status:** Approved design, ready for implementation planning

## Summary

For episodes that originate from a YouTube podcast feed, PodFetch fetches
[SponsorBlock](https://github.com/ajayyy/SponsorBlock) segments at download time
and lets each user auto-skip the categories they choose, during playback, in the
web and mobile players — **without ever modifying the audio file**.

SponsorBlock's crowdsourced segment database is keyed by YouTube video ID, so the
feature is only meaningful for episodes that exist as YouTube videos (the podcasts
YouTube exposes via RSS). For any other episode the feature is inert.

## Core decisions

| Decision | Choice |
|---|---|
| What we do with segments | **Mark as skippable** (non-destructive — audio file is never altered) |
| Auto-skip surfaces in v1 | **Web player + mobile app**; segments exposed via API for all other clients |
| Category selection | **Per-user** — each user toggles which categories to skip; segment data is shared |
| When segments are fetched | **At download time** (snapshot all categories once per episode); re-fetch via rescan |
| Architecture | **Approach B** — capture the YouTube video ID explicitly at feed ingest |

### Out of scope for v1
- Cutting/muting audio (segments are marked only).
- Auto-skip on Mopidy / Chromecast (they still receive the data via API).
- SponsorBlock sources other than YouTube.
- Submitting/voting segments back to SponsorBlock.
- A mobile settings *editing* screen (mobile applies prefs and skips from day one;
  editing categories on mobile is a fast-follow — web is the guaranteed v1 config surface).

## Architecture overview

End-to-end flow:

1. **Ingest** — when an episode is parsed from its feed, extract a YouTube video ID
   (from `<link>`, falling back to `guid`, then enclosure `url`) and store it on the
   episode. No ID → not a YouTube episode → feature is inert for it.
2. **Download** — after the audio file is written (in `download_podcast_episode`), if
   the episode has a video ID *and* the feature is globally enabled, query SponsorBlock
   once and store all returned segments (every category) for that episode.
3. **Serve** — an API endpoint returns an episode's segments plus the caller's effective
   preferences; other endpoints get/set the current user's per-category skip preferences.
4. **Skip** — the web and mobile players load the episode's segments + the user's prefs and,
   during playback, seek past any segment whose category the user has enabled.
5. **Re-fetch** — a new rescan option re-queries SponsorBlock for already-downloaded
   episodes (data improves over time / backfill for episodes from before the feature existed).

## Data model & migrations

All migrations are **additive** (one nullable column + new tables); no table rebuilds, so
the SQLite FK-pragma table-rebuild gotcha does not apply. Migrations are authored for both
the SQLite and Postgres migration directories, as the repo requires.

### a) `podcast_episodes` — one new column
- `youtube_video_id TEXT NULL` — populated at ingest via a plain `ALTER TABLE ADD COLUMN`
  (works on SQLite and Postgres).

### b) New table `episode_sponsor_segments` (shared across users, one row per SponsorBlock segment)

| column | type | notes |
|---|---|---|
| `id` | Text (UUID) PK | matches the project's UUID-PK convention |
| `episode_id` | Text FK → episode | references the episode the same way `podcast_episode_chapters` does |
| `uuid` | Text | SponsorBlock's segment UUID |
| `category` | Text | `sponsor`, `selfpromo`, `interaction`, `intro`, `outro`, `preview`, `filler`, `music_offtopic` |
| `action_type` | Text | SponsorBlock `actionType` (`skip`/`mute`/`poi`/`full`); v1 acts on `skip` |
| `start_ms` | BigInt | converted from SponsorBlock's float seconds |
| `end_ms` | BigInt | converted from SponsorBlock's float seconds |
| `votes` | Int | stored for optional client-side filtering of low-quality segments (not used by v1 logic) |
| `locked` | Bool | stored for optional client-side filtering (not used by v1 logic) |
| `duration_mismatch` | Bool | true when SponsorBlock's `videoDuration` diverges from the episode's actual duration beyond tolerance — see Timestamp-alignment guard |
| `fetched_at` | Timestamp | when this snapshot was taken |

- **Unique index on `(episode_id, uuid)`** so re-fetch is idempotent (upsert, no duplicates).

### c) New per-user table `sponsorblock_user_settings` (mirrors the flat `filters` / `favorites` style, keyed by `user_id`)
- `user_id` Text PK, FK → `users(id)`
- `enabled` Bool — master per-user switch
- one Bool per category: `skip_sponsor`, `skip_selfpromo`, `skip_interaction`, `skip_intro`,
  `skip_outro`, `skip_preview`, `skip_filler`, `skip_music_offtopic`
- **Defaults for a new/absent row:** `enabled=true`, `skip_sponsor=true`, `skip_selfpromo=true`,
  everything else `false`. A missing row is treated as these defaults, so existing users need
  no backfill.
- Fixed boolean columns (rather than a row-per-category table) — simpler; adding a future
  category is a migration, which is acceptable since categories rarely change.

### d) Global setting
- Add `sponsorblock_enabled` (Bool, default `true`) to the existing global `Settings`. Admin-level
  master switch that gates all outbound SponsorBlock calls (for offline / air-gapped instances).
- The SponsorBlock API base URL defaults to `https://sponsor.ajay.app` and is overridable via the
  `SPONSORBLOCK_API_URL` env var, for self-hosted SponsorBlock mirrors.

### Frontend types
- New API DTOs require hand-editing **`ui/schema.d.ts`** and **`mobile/schema.d.ts`** (these are
  maintained by hand in this repo).

## Backend

### Video-ID extraction (ingest)
- New helper `services/sponsorblock/video_id.rs` →
  `extract_youtube_video_id(item: &rss::Item) -> Option<String>`:
  - Try `<link>` first (`youtube.com/watch?v=`, `youtu.be/`, `/embed/`, `music.youtube.com`),
    then `guid` (`yt:video:ID`), then enclosure `url`.
  - Validate the 11-char `[A-Za-z0-9_-]` shape.
- Wired into the single place episodes are created (`usecases/podcast_episode/mod.rs`, ~L209–231).
  PodFetch currently discards the RSS `<link>` element — we start reading `item.link` and write the
  result into `youtube_video_id`. On feed re-scan, fill it in if still null (backfill without
  re-adding podcasts).

### SponsorBlock client
- New `services/sponsorblock/client.rs` — a thin client built on the **existing**
  `get_async_sync_client(&ENVIRONMENT_SERVICE)` + `COMMON_USER_AGENT` + `map_reqwest_error`,
  rather than the `sponsor-block` crate (avoids reqwest/tokio version skew with the workspace).
- Uses the **privacy-preserving** API: `sha256(videoID)` → first 4 hex chars →
  `GET {base}/api/skipSegments/{prefix}?categories=[…all…]&actionTypes=["skip"]`, then filter the
  response to the exact video ID locally. The exact video ID is never sent to the API.
- Requests all supported categories (storage is category-agnostic; users filter at playback).
- Parses JSON → segment structs; converts `segment: [startSec, endSec]` floats to ms; captures
  `category`, `actionType`, `UUID`, `votes`, `locked`, and `videoDuration`.

### Fetch + store
- New `services/sponsorblock/service.rs::fetch_and_store(episode)`:
  - Guards on global `sponsorblock_enabled` and `episode.youtube_video_id.is_some()`.
  - Upserts into `episode_sponsor_segments` keyed on `(episode_id, uuid)`; deletes rows absent
    from the latest fetch (so re-fetch reflects removals).
  - Computes `duration_mismatch` per the alignment guard below.
- Called from `download_podcast_episode` **after** the file write, with the same non-fatal error
  isolation as metadata insertion — a SponsorBlock failure logs and is swallowed, never failing the
  download. Runs inline (bounded by `max_parallel_downloads`).

### API (utoipa-documented, alongside existing controllers)
- `GET …/episodes/{episode_id}/sponsorblock` → the episode's segments (all categories) **plus** the
  caller's effective prefs, in one payload (one round trip; client does the filtering/skip).
- `GET /api/v1/settings/sponsorblock` → current user's prefs (defaults applied if no row).
- `PUT /api/v1/settings/sponsorblock` → upsert current user's prefs.
- `sponsorblock_enabled` added to the existing global Settings DTO + update endpoint (admin only).
- `refetchSponsorblock` added to `RescanOptions`, handled in `EpisodeRescanService::apply_to_episode`
  (refresh video ID if null, then `fetch_and_store`).

## Client (web + mobile)

### Web player skip
- Extend the audio-player Zustand store: when an episode loads, call `GET …/sponsorblock` and keep
  `segments` + the user's `prefs`. Precompute a sorted list of **active skip ranges** = segments whose
  `actionType === "skip"`, whose `category` is enabled in the user's prefs, and which are **not**
  `duration_mismatch`-flagged (skip everything if `prefs.enabled` is false).
- In `HiddenAudioPlayer.tsx`'s existing `onTimeUpdate` handler (~L201–246), if `currentTime` is inside
  an active range, set `audioPlayer.currentTime = range.end` (+ tiny epsilon). Optional unobtrusive
  toast ("Skipped sponsor").
- **Loop guard:** remember the last range auto-skipped + a short cooldown, so if the user manually
  seeks back into it we don't fight them. If `range.end ≥ duration`, advance/clamp like a normal
  end-of-episode.
- **Overlap/adjacency:** merge overlapping active ranges to avoid seek jitter; clamp negative/zero-length
  segments.

### Mobile player skip
- Same range logic in the mobile player's progress handler, using the shared types from
  `mobile/schema.d.ts`.

### Settings UI
- **Web (the v1 config surface):** a new section in user settings — master toggle + a checkbox per
  category (sponsor, self-promo, interaction, intro, outro, preview, filler, music-off-topic) with i18n
  labels/explanations (`en.json` at minimum; other locales follow the existing pattern). `PUT` on change.
- **Admin:** the global `sponsorblock_enabled` toggle in the existing global-settings page.
- **Mobile** applies prefs and skips from day one; a mobile screen to edit categories is a fast-follow.

## Error handling & edge cases

- **SponsorBlock unreachable / 429 / 404 (no data):** log at `warn`, never fail the download. A
  404/empty response means "no segments" → store none and clear stale rows.
- **Global disabled or no video ID:** fetch is skipped entirely; non-YouTube libraries never touch the API.
- **Timestamp-alignment guard (key correctness risk):** SponsorBlock timestamps are relative to the
  YouTube video. If the downloaded audio differs in length, they'd misalign. Mitigation: store
  SponsorBlock's `videoDuration` and compare to the episode's actual duration at fetch time. If they
  diverge beyond tolerance (≈ >1% or >2s), set `duration_mismatch = true` on those rows and players do
  **not** auto-skip flagged segments (still stored/visible via API). This prevents silently seeking past
  wanted content.
- **Privacy:** the hash-prefix API means the exact video ID is never sent.
- **Backfill:** episodes from before the feature get a video ID + segments via the `refetchSponsorblock`
  rescan option.

## Testing

### Rust unit
- `extract_youtube_video_id`: table of inputs (`watch?v=`, `youtu.be`, `/embed/`, `music.youtube`,
  `yt:video:`, non-YouTube → `None`, malformed → `None`).
- SponsorBlock JSON parsing + seconds→ms conversion + category/actionType mapping (pure parse, no network).
- `fetch_and_store` upsert / stale-delete idempotency, against SQLite **and** Postgres per CI.
- `duration_mismatch` flagging logic.
- Default prefs when no row; `PUT` upsert.
- "Fetch failure does not fail the download" (mirrors the existing non-fatal metadata-insertion
  regression test).

### Frontend (Vitest)
- Active-range computation (category filter, master-off, actionType filter, `duration_mismatch` filter,
  overlap merge).
- Skip decision given `currentTime` (inside / outside / boundary, `end ≥ duration`, loop-guard cooldown).

### API
- Segments + prefs payload; settings GET/PUT; rescan option triggers refetch.

### Verification gate (per repo CI)
- `clippy -D warnings` + test suites on both DBs + frontend `tsc`/build must pass.
- `ui/schema.d.ts` and `mobile/schema.d.ts` updated by hand.
- No `cargo fmt` gate (do not run `cargo fmt`).

## Key files touched (orientation, not exhaustive)

- `crates/podfetch-persistence/` — `schema.rs`, new migrations (SQLite + Postgres), episode + new entities.
- `crates/podfetch-web/src/usecases/podcast_episode/mod.rs` — video-ID extraction at ingest.
- `crates/podfetch-web/src/services/download/service.rs` — call `fetch_and_store` post-download.
- `crates/podfetch-web/src/services/sponsorblock/` — new `video_id.rs`, `client.rs`, `service.rs`.
- `crates/podfetch-web/src/services/episode_rescan/service.rs` — `refetchSponsorblock` rescan option.
- `crates/podfetch-web/src/controllers/` + `settings.rs` — new endpoints, global setting.
- `ui/src/store/AudioPlayerSlice.ts`, `ui/src/components/HiddenAudioPlayer.tsx` — skip logic.
- `ui/src/...` settings page + `ui/src/language/json/en.json` — settings UI + i18n.
- `ui/schema.d.ts`, `mobile/schema.d.ts` — DTOs (hand-edited).
- mobile player progress handler — skip logic.
