# Configurable Parallel Downloads — Design

- **Date:** 2026-06-04
- **Status:** Approved (design)
- **Issue:** #2069 — "Set number of simultaneous transcode for Transcode to Opus"

## Goal

Let operators bound how many episodes download (and therefore transcode to
Opus) at once, instead of the hardcoded `MAX_PARALLEL_DOWNLOADS = 3`. On a slow
CPU, lowering this to 1 means one episode downloads+transcodes at a time.

## Background

`PodcastEpisodeService` / `schedule_episode_download`
(`crates/podfetch-web/src/services/podcast/service.rs`) spawns episode
downloads in chunks of `MAX_PARALLEL_DOWNLOADS` OS threads. The Opus transcode
(`DownloadService::transcode_to_opus`, gated by `settings.auto_transcode_opus`)
runs **inline** inside each download thread. So the number of parallel
downloads is also the number of simultaneous Opus transcodes — the "3" the
issue reports.

## Decision

A single **global setting** `max_parallel_downloads` (integer, default **3**),
editable in the web UI, replaces the constant. Clamped to **min 1** at the read
site so a bad value can never stall downloads. Global only (not per-podcast —
YAGNI). The download scheduler already loads `settings` in scope, so it reads
`settings.max_parallel_downloads.max(1)` instead of the constant.

This bounds simultaneous Opus transcodes by bounding parallel downloads. A
dedicated transcode-only semaphore (decoupled from download concurrency) was
considered and rejected as more code for marginal benefit on the slow-CPU case.

## Changes

- **Migration** (sqlite + postgres), new `2026-06-04-…_settings_max_parallel_downloads`:
  `ALTER TABLE settings ADD COLUMN max_parallel_downloads INTEGER NOT NULL DEFAULT 3;`
  (plain add-column; reversible `DROP COLUMN` in `down.sql`; transaction-safe,
  no table rebuild).
- **schema.rs** + **persistence `settings.rs`**: add `max_parallel_downloads -> Integer`
  to the `table!` block and `SettingEntity`, both `From` impls, and
  `insert_default_settings` (default 3).
- **domain `Setting`** and **web `Setting` DTO** (+ utoipa, camelCase
  `maxParallelDownloads`): add `max_parallel_downloads: i32`.
- **`schedule_episode_download`**: chunk size = `settings.max_parallel_downloads.max(1) as usize`.
- **Clients**: regenerate `ui`/`mobile` `schema.d.ts`; add a numeric input to the
  `ui/` settings form (label "Max parallel downloads", help text noting it also
  bounds simultaneous Opus transcodes). `mobile/` only if it edits settings.
- **Tests**: settings round-trip persists the field; the scheduler reads the
  configured value (default 3 preserved).

## Non-goals

- Per-podcast override. A separate transcode-only concurrency limit. Changing
  download/transcode architecture.
