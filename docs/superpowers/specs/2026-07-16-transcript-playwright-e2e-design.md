# Playwright E2E Tests for Transcript Features — Design

**Goal:** Browser-level regression coverage for the Podcasting 2.0 transcript UI
(player tab, full-text search mode, transcribe action/badge, auto-transcribe
setting), running in GitHub CI against the real server.

## Architecture

Full-stack: Playwright drives the real PodFetch binary (sqlite, fresh temp DB
per run) serving the real built UI from `./static`. Playwright's `webServer`
array owns the lifecycle of all three processes:

1. **Fixture feed server** (`ui/e2e/servers/fixture-feed.mjs`, Node,
   127.0.0.1:9123) — serves `feed.xml` (one episode with a
   `<podcast:transcript>` VTT tag), a minimal MP3 and `transcript.vtt`.
2. **Mock Whisper server** (`ui/e2e/servers/mock-whisper.mjs`, Node,
   127.0.0.1:9998) — OpenAI-compatible `POST /v1/audio/transcriptions`
   returning fixed `verbose_json` segments after a ~1s delay (so the pending
   badge is observable).
3. **PodFetch** (built binary, 127.0.0.1:8000) — `DATABASE_URL` pointing at a
   temp sqlite file, `TRANSCRIPTION_API_BASE_URL=http://127.0.0.1:9998`,
   working directory prepared with `static/` = built UI dist.

Seeding happens through the API (Playwright request context): global setup
adds the fixture podcast via `POST /api/v1/podcasts/feed` and waits until the
episode is downloaded and its feed transcript parsed. Tests then only drive
UI flows.

## Test cases (`ui/e2e/tests/transcripts.spec.ts`)

1. **Player transcript tab** — play the episode, open the detailed player,
   switch to the Transcript tab; expect segments with timestamps and speaker
   "Alice"; click a segment and expect the audio element's `currentTime` to
   jump to the segment start.
2. **Transcript search** — on the episode search page switch to the
   Transcripts mode, type the fixture keyword; expect an episode card with a
   highlighted (`<b>`) snippet.
3. **Transcribe action** — the episode row shows the transcribe icon; click
   it; expect the pending state, then (mock responds) the done state.
4. **Auto-transcribe setting** — the podcast settings modal shows the
   Auto-transcribe toggle (transcription is enabled via env).

## CI

New workflow `.github/workflows/playwright.yml` (pull_request + push to
main): checkout, Rust toolchain + cache, `cargo build --no-default-features
--features sqlite`, pnpm install + `pnpm run build` in `ui/`, copy `ui/dist`
to `static/`, `pnpm exec playwright install --with-deps chromium`, run
Playwright. On failure upload `playwright-report`/traces as artifact.

## Error handling / stability

- No fixed sleeps in tests; all waits are Playwright web-first assertions
  with polling (`expect(...).toBeVisible()` etc.).
- Fresh DB per run (temp dir) — no cross-run state.
- All servers bind 127.0.0.1 only.
- Login: instance runs without auth; if the UI still routes through a login
  screen, global setup performs the minimal login step once and stores
  storage state.
