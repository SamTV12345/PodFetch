# Audiobookshelf integration

PodFetch speaks the audiobookshelf REST API and socket.io protocol. The
official [audiobookshelf mobile apps](https://www.audiobookshelf.org/)
(Android + iOS) and the audiobookshelf web client can connect to PodFetch
as if it were an audiobookshelf server — list your podcasts, stream them,
sync listening progress and aggregate listening statistics.

> Audiobooks themselves are not yet exposed through the mobile app; the
> Phase B scanner is in place but the mobile clients consume the podcast
> library first. See [Roadmap](#roadmap--limitations) below.

## Why bother

Building and maintaining a first-class PodFetch mobile app (background
playback, offline downloads, lock-screen controls, Android Auto, CarPlay)
is months of work for a side project. The audiobookshelf community has
already solved that, and their apps are open-source. Hosting PodFetch
behind an audiobookshelf-compatible API gives you their app
ecosystem for free — without bundling audiobookshelf-server.

## Enable it

Set the environment flag and restart the server:

```bash
AUDIOBOOKSHELF_INTEGRATION_ENABLED=true
```

That mounts the audiobookshelf-shaped routes at the root paths the
mobile apps hardcode: `/login`, `/api/...`, `/public/...`, `/hls/...`,
`/socket.io/`. They run alongside PodFetch's own UI and gpodder API.

Optional knobs:

| Variable | Default | What it does |
|---|---|---|
| `AUDIOBOOKSHELF_DATA_DIR` | `<podfetch_data>/audiobookshelf` | Working dir for HLS segments & co. |
| `AUDIOBOOKSHELF_HLS_CACHE_MAX_MB` | `2048` | LRU cap for transcoded HLS segments. |
| `AUDIOBOOKSHELF_TRANSCODER_MAX_CONCURRENT` | `2` | ffmpeg processes running in parallel. |
| `AUDIOBOOKSHELF_ROTATE_API_KEY_ON_LOGOUT` | `false` | Generate a fresh `users.api_key` on `/logout`. |

## Connecting the official mobile app

Install the audiobookshelf app
([Play Store](https://play.google.com/store/apps/details?id=com.audiobookshelf.app),
[F-Droid](https://f-droid.org/packages/com.audiobookshelf.app/),
[App Store](https://apps.apple.com/us/app/audiobookshelf/id1631241544)).

1. Open the app, pick **Connect to server**.
2. Enter your PodFetch URL (e.g. `https://podcasts.example.com`).
3. Sign in with your normal PodFetch username / password — PodFetch's
   `/login` endpoint validates against the same user store that the web
   UI uses, no separate account needed.
4. After login the app sees one library called `Podcasts` containing
   every podcast you have subscribed to in PodFetch.

The session token the app stores is your `users.api_key`. PodFetch
re-uses the existing column instead of issuing a separate
audiobookshelf token — so resetting the key in *Profile → API key*
also invalidates the app session.

> **HTTPS is required.** The mobile app rejects plain-HTTP servers.
> Behind a reverse proxy that terminates TLS this works out of the box;
> for local dev, a Cloudflare Tunnel or `tailscale serve` is the
> easiest way to get a real cert.

## Authentication

PodFetch accepts the bearer token the audiobookshelf clients send:

```
Authorization: Bearer <users.api_key>
```

Streaming endpoints additionally accept `?token=<api_key>` as a query
parameter because the `<audio>` element can't send custom headers from
the browser/mobile player.

The `/login` response is shaped exactly like upstream audiobookshelf's,
so the mobile app accepts PodFetch as a legitimate server without any
client-side patches:

```json
{
  "user": {
    "id": "1",
    "username": "samuel",
    "token": "abs_...",
    "mediaProgress": [...],
    "permissions": { ... },
    "librariesAccessible": [],
    "isActive": true,
    "type": "root"
  },
  "userDefaultLibraryId": "lib_default_podcasts",
  "serverSettings": { ... }
}
```

## Streaming

When the mobile app hits `POST /api/items/<id>/play/<episodeId>`,
PodFetch decides per request whether to direct-stream the file or
transcode it on the fly to HLS:

* **`playMethod=0` (direct)** — used when the source codec is in the
  client's `supportedMimeTypes` (mp3, aac, m4a, opus, …). The track URL
  is `/api/items/<itemId>/file/<ino>` and supports HTTP Range, so the
  player can seek without downloading the whole episode.
* **`playMethod=1` (HLS transcode)** — used for FLAC, OGG with
  unsupported codecs, or when the client passes `forceTranscode: true`.
  PodFetch spawns ffmpeg per segment, AAC-encodes to a mpegts segment
  and caches the output under `<AUDIOBOOKSHELF_DATA_DIR>/hls/<sid>/`
  bounded by the LRU cap above.

A `Semaphore` keeps `AUDIOBOOKSHELF_TRANSCODER_MAX_CONCURRENT` ffmpeg
processes running at most.

## Progress sync

The mobile app calls `POST /api/session/<sid>/sync` every ~10 s with
`{ currentTime, timeListened, duration }`. PodFetch:

1. Updates `playback_sessions.current_time / time_listening_total`.
2. Mirrors the position into `media_progress` (the per-libraryItem
   table that drives the "Continue listening" shelf).
3. Emits `user_item_progress_updated` over socket.io so any second
   logged-in client (web UI, second phone) refreshes immediately.

On session close (`POST /api/session/<sid>/close`) PodFetch finalises
the row, copies the snapshot into `audiobookshelf_listening_sessions`
for history, and marks the episode finished when
`current_time / duration > 0.95`.

## Listening statistics

`GET /api/me/listening-stats` aggregates everything in
`audiobookshelf_listening_sessions` into the shape the upstream web
dashboard expects:

```json
{
  "totalTime": <seconds>,
  "today": <seconds>,
  "items": { "li_pod_<id>": { "timeListening": ..., "mediaMetadata": ..., "lastUpdate": ... } },
  "days":      { "2026-05-14": <seconds>, ... },
  "dayOfWeek": { "Monday": <seconds>, ... },
  "recentSessions": [ ... last 10 ... ]
}
```

`date` / `dayOfWeek` / `mediaMetadata` are derived per request from
`started_at` and `display_title/displayAuthor`; PodFetch doesn't store
them as separate columns.

## Playlists

PodFetch's existing playlist domain is exposed via the audiobookshelf
playlist surface so the mobile app's playlist tab works end-to-end:

```
GET    /api/playlists                              # list
GET    /api/playlists/:id                          # detail
POST   /api/playlists                              # create
PATCH  /api/playlists/:id                          # rename + reorder
DELETE /api/playlists/:id
POST   /api/playlists/:id/item                     # add one
DELETE /api/playlists/:id/item/:liId[/:episodeId]
POST   /api/playlists/:id/batch/{add,remove}
GET    /api/libraries/:id/playlists                # per-library, paginated
```

PodFetch playlists are podcast-episode-only — the request grammar
still mirrors upstream's `{ libraryItemId, episodeId? }` pairs, but
`episodeId` is required server-side.

## Search & adding podcasts

When the user taps **+ Add podcast** in the mobile app:

* `GET /api/search/podcast?term=...&country=us` runs an iTunes Search
  via PodFetch's existing `PodcastService::find_podcast` and returns
  the audiobookshelf-shaped result array (`title`, `artistName`,
  `feedUrl`, `cover`, `explicit`, ...).
* `POST /api/podcasts/feed { rssFeed }` fetches the RSS feed and
  returns a `{ podcast: { metadata, episodes } }` preview — also used
  when the user pastes a feed URL directly.
* `POST /api/podcasts { media: { metadata: { feedUrl } }, libraryId, … }`
  actually subscribes via `PodcastService::handle_insert_of_podcast`
  (dedup by feedUrl, spawn episode discovery, schedule downloads) and
  returns the new `LibraryItem`. PodFetch has no library-folder
  concept yet, so `libraryId / folderId / path` on the request body
  are accepted-and-ignored — the podcast lands in the single default
  `Podcasts` library.

## Socket.io events

The mobile app and web client subscribe to socket.io for real-time
updates. PodFetch broadcasts the same events upstream emits, scoped
per-user where appropriate:

| Event | Payload | When |
|---|---|---|
| `init` | `{ user, libraries, serverSettings }` | After client `auth` event |
| `user_item_progress_updated` | progress row | On every sync / close |
| `user_updated` | full user | After progress patch |
| `library_updated` | library | On scan complete |
| `scan_progress` / `scan_complete` | scan id + ratio | Scanner phases |

The handshake auth supports both query token (`?token=...`) and the
audiobookshelf-app's `socket.emit('auth', <token>)` event-based flow.

## API surface in one glance

| Group | Endpoints |
|---|---|
| Auth | `POST /login`, `POST /api/authorize`, `POST /logout` |
| Server | `GET /ping`, `GET /status`, `GET /api/server-settings` |
| Libraries | `GET /api/libraries`, `GET /api/libraries/:id`, `GET /api/libraries/:id/items`, `GET /api/libraries/:id/recent-episodes`, `GET /api/libraries/:id/personalized`, `POST /api/libraries/:id/scan` |
| Items | `GET /api/items/:id`, `GET /api/items/:id/cover`, `GET /api/items/:id/file/:ino`, `GET /api/items/:id/episode/:epId` |
| Me | `GET /api/me`, `GET /api/me/items-in-progress`, `GET /api/me/listening-sessions`, `GET /api/me/listening-stats`, `PATCH /api/me/progress/:liId[/:epId]`, `PATCH /api/me/progress/batch/update` |
| Sessions | `POST /api/items/:id/play[/:epId]`, `GET /api/session/:id`, `POST /api/session/:id/sync`, `POST /api/session/:id/close` |
| Streaming | `GET /public/session/:sid/track/:idx` (direct + Range), `GET /hls/:sid/master.m3u8`, `GET /hls/:sid/index.m3u8`, `GET /hls/:sid/seg-:n.ts` |
| Search | `GET /api/search/podcast?term=...` |
| Podcasts | `POST /api/podcasts/feed`, `POST /api/podcasts` |
| Playlists | see above |
| Uploads | `POST /api/upload` (multipart, for audiobook drops) |
| Socket | `GET /socket.io/` |

## Roadmap & limitations

**Working today**

* Full podcast browse / play / progress / playlists / stats from the
  mobile app and audiobookshelf web client.
* Direct streaming + HLS transcoding both engaged automatically based
  on client `supportedMimeTypes`.
* Multi-device progress sync via socket.io.

**Planned**

* Audiobook scanner (`AUDIOBOOKSHELF_INTEGRATION_ENABLED`-gated tables
  exist, watched-folder discovery + ffprobe + chapter extraction +
  metadata precedence chain are in tree) needs the corresponding
  mobile-app library-type wiring once we surface a Books library.
* Manual audiobook upload (`POST /api/upload` multipart) accepts files
  and triggers a single-folder scan; mobile-app UI for browsing the
  resulting Books library is the next step.

**Won't fix** for now

* PodFetch playlists are podcast-only. Audiobook playlists will land
  with the Books library.
* Library folders: PodFetch uses a single root dir. Library/folder/
  path fields on `POST /api/podcasts` are accepted-and-ignored so the
  mobile app doesn't error.

## Troubleshooting

**App accepts login but never lists podcasts**
The default `Podcasts` library is created at server startup. Restart
PodFetch once after enabling the integration; the
`AudiobookshelfLibraryService` runs its bootstrap on boot, not lazily.

**`MissingKotlinParameterException` in `adb logcat` after tapping Play**
Means an upstream non-null field is missing in PodFetch's play
response. Open an issue with the stack trace — these have been
caught one by one (audioFile.relPath, deviceInfo, genres, …) and the
test pin in `audiobookshelf_api::tests::play_response_passes_android_kotlin_required_fields`
guards each.

**Sync logs "Invalid JSON response End of input"**
Every audiobookshelf sync / close / progress endpoint must return a
JSON body — sonner's `makeRequest` parses the body and treats an
empty 200 as an error. PodFetch returns `{ "success": true }` for the
non-content endpoints; if you see this, something is short-circuiting
the response builder.

**Casting / Chromecast**
The audiobookshelf app's casting works only against shared HLS
streams. Direct casting from the app is on the audiobookshelf
roadmap, not PodFetch's.
