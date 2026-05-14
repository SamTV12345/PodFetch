# Audiobookshelf API

PodFetch can answer to the audiobookshelf protocol. That means the official
[audiobookshelf mobile apps](https://www.audiobookshelf.org/) (Android and
iOS) and the audiobookshelf web client connect to your PodFetch server,
list your podcasts and play episodes with progress sync.

This page walks you through enabling the integration and pairing the apps.

## 1. Enable the integration

Set the environment variable and restart the server:

```bash
AUDIOBOOKSHELF_INTEGRATION_ENABLED=true
```

In `docker-compose.yml`:

```yaml
services:
  podfetch:
    image: samuel19982/podfetch:latest
    environment:
      AUDIOBOOKSHELF_INTEGRATION_ENABLED: "true"
      # … your other variables …
```

On startup PodFetch mounts the audiobookshelf-shaped routes at the root
paths the mobile apps hardcode (`/login`, `/api/...`, `/public/...`,
`/hls/...`, `/socket.io/`) and creates a default *Podcasts* library
containing every podcast you have already subscribed to in PodFetch.

> **HTTPS is mandatory.** The audiobookshelf apps refuse plain-HTTP
> servers. If you already host PodFetch behind a TLS-terminating reverse
> proxy (Caddy, nginx, Traefik) you're done. For a quick local test use
> a [Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/)
> or `tailscale serve` — both give you a valid certificate without
> opening ports.

## 2. Optional configuration

All of these have sensible defaults; only set them if you need to tweak
the behaviour.

| Variable | Default | What it does |
|---|---|---|
| `AUDIOBOOKSHELF_DATA_DIR` | `<podfetch_data>/audiobookshelf` | Working dir for the HLS segment cache. Point this at fast local storage if you transcode a lot. |
| `AUDIOBOOKSHELF_HLS_CACHE_MAX_MB` | `2048` | Hard cap on the HLS segment cache (LRU eviction). Raise on big servers, lower on Raspberry Pis. |
| `AUDIOBOOKSHELF_TRANSCODER_MAX_CONCURRENT` | `2` | Maximum ffmpeg processes that may run in parallel. One per concurrent transcoding listener. |
| `AUDIOBOOKSHELF_ROTATE_API_KEY_ON_LOGOUT` | `false` | When `true`, signing out from the mobile app rotates `users.api_key`, immediately invalidating any other device still logged in with the old token. |

`ffmpeg` must be on `PATH` for HLS transcoding to work. The Docker image
ships with it; on a manual install verify with `ffmpeg -version`.

## 3. Install the mobile app

Pick the store that matches your phone:

- Android: [Play Store](https://play.google.com/store/apps/details?id=com.audiobookshelf.app),
  [F-Droid](https://f-droid.org/packages/com.audiobookshelf.app/) or
  the [APK from the project's releases page](https://github.com/advplyr/audiobookshelf-app/releases).
- iOS: [App Store](https://apps.apple.com/us/app/audiobookshelf/id1631241544).

## 4. Connect to PodFetch

1. Open the app and pick **Connect to server**.
2. Enter your PodFetch URL — the same URL you use for the web UI,
   including the scheme: `https://podcasts.example.com`. Do **not**
   append `/ui/` or any path.
3. Enter your PodFetch **username** and **password**. The same
   credentials the web UI uses; the app does not need a separate
   account.
4. Tap **Connect**.

After a successful login the app shows a single library called
**Podcasts**. Open it to see every podcast you have subscribed to in
PodFetch. Tap an episode to start playback.

The token the app stores after login is your PodFetch `users.api_key`.
You can rotate it from the PodFetch web UI in *Profile → API key* — the
app will be signed out the next time it tries to sync.

## 5. Verify it works

Quick smoke test, in order:

1. **Listing:** every podcast you subscribed to in PodFetch shows up.
2. **Playback:** tap an episode. The player should open at the saved
   position (or 0 for the first play) and start within a few seconds.
   FLAC episodes take longer the first time because they are transcoded
   per segment.
3. **Progress sync:** play 30 seconds, pause, force-close the app,
   re-open. The episode should resume at roughly where you stopped.
   Cross-check in the PodFetch web UI under *Home → Continue listening*.
4. **Stats:** browse to *Stats* in the mobile app. After at least one
   played episode you should see a non-zero total and today's listening
   time.

## 6. Adding new podcasts from the app

In the app, tap **+** and either

- type a search term (the app calls iTunes through PodFetch and shows
  matching podcasts), or
- paste an RSS feed URL directly.

Pick a podcast and confirm. PodFetch subscribes it the same way the web
UI's *Add Podcast* dialog does, so the new feed appears in PodFetch as
well as in the app.

> PodFetch has no concept of library folders, so the *Library* and
> *Folder* dropdowns in the audiobookshelf "New Podcast" form are
> ignored. The podcast always lands in the default *Podcasts* library.

## 7. Reverse proxy notes

If you front PodFetch with a reverse proxy, two things matter:

- Pass through `/socket.io/` with WebSocket upgrade. The app uses
  socket.io for real-time progress and notification events; without it
  multi-device sync still works (over HTTP polling) but with a 5–10
  second delay.
- Forward the `Authorization` header. PodFetch reads the bearer token
  from it; some default proxy configs strip unknown headers.

Sample Caddy snippet:

```caddy
podcasts.example.com {
    reverse_proxy localhost:8000
}
```

Sample nginx snippet (the important parts):

```nginx
location / {
    proxy_pass http://127.0.0.1:8000;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header Authorization $http_authorization;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection $connection_upgrade;
}
```

`$connection_upgrade` requires the standard nginx upgrade map:

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    '' close;
}
```

## Troubleshooting

**The app accepts login but the library list is empty.**
The default *Podcasts* library is created during server startup. If you
enabled the integration on a running server, restart PodFetch once. After
restart, the library appears even if you don't have any subscribed
podcasts yet — but it will be empty until you add some.

**Login fails with "Unexpected error".**
Most often the URL is wrong: the app's *Server URL* field expects the
root (`https://podcasts.example.com`), not a path. Strip `/ui/`,
`/login` or any other suffix. Double-check the scheme — `http://`
fails because the app requires TLS.

**Playback button shows a spinner forever.**
Check the server log for a line like `POST /api/items/.../play …`. If
it never appears, the bearer token did not reach PodFetch — your
reverse proxy is dropping the `Authorization` header. See section 7.

**Progress does not sync between devices.**
The socket.io connection is probably blocked. Check
`https://your-server/socket.io/?EIO=4&transport=polling` in the browser
— it should return a JSON handshake. If it returns a 502 or 404 your
proxy is not forwarding `/socket.io/`; add it to the proxy config.

**FLAC / OGG episode stutters or won't play.**
PodFetch transcodes those to HLS/AAC via ffmpeg. Make sure ffmpeg is
on `PATH`. If transcoding is slow, raise
`AUDIOBOOKSHELF_TRANSCODER_MAX_CONCURRENT` (more parallel ffmpegs) or
point `AUDIOBOOKSHELF_DATA_DIR` at faster storage.

**Cover images don't appear.**
The app caches covers per server. After a server restart or an HTTPS
certificate change, force-refresh once (pull-to-refresh on the
library screen).

**I want to sign all devices out at once.**
Set `AUDIOBOOKSHELF_ROTATE_API_KEY_ON_LOGOUT=true` and log out from
one device. PodFetch generates a new `users.api_key` and every other
device drops to the login screen on its next sync. Alternatively,
rotate the key manually in *Profile → API key* in the web UI.

## Disabling the integration

Set `AUDIOBOOKSHELF_INTEGRATION_ENABLED=false` (or remove the variable)
and restart PodFetch. The audiobookshelf routes disappear from the
server; the rest of PodFetch keeps working unchanged. Existing user
data (subscribed podcasts, listening history, playlists) stays — the
audiobookshelf endpoints just expose them in a different format.
