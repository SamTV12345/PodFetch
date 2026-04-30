# Kodi (RSS Podcasts addon)

This page covers using the third-party Kodi addon
[**RSS Podcasts** by Heckie75](https://github.com/Heckie75/kodi-addon-podcast)
to import your PodFetch subscriptions into Kodi. The addon talks to
PodFetch's gpodder-compatible Simple API.

## Prerequisites

- PodFetch reachable from the Kodi machine over HTTP or HTTPS.
- `GPODDER_INTEGRATION_ENABLED=true` on the PodFetch container — see
  [Adding GPodder support § Activating the GPodder API](./GPodder.md#-activating-the-gpodder-api).
- A PodFetch user with username and password — see
  [Adding GPodder support § Create a user via the CLI](./GPodder.md#-create-a-user-via-the-cli).

## Setup in Kodi

1. **Install the addon.** In Kodi, go to *Add-ons → Install from
   repository → All repositories → Music → RSS Podcasts*.
2. **Configure the provider.** Open the addon and select *Configure
   → Provider: `gpodder.net`*.
3. **Enter the hostname.** Use the bare host only — for example
   `podfetch.example.com`. Do **not** include a scheme (`https://`),
   port suffix, or trailing slash; the addon adds those itself.
4. **Enter credentials.** Username and password of the PodFetch user
   you created in the prerequisites.
5. **Import subscriptions.** Open the addon and choose *Import
   subscriptions to group*, then pick a Kodi group name. Your
   PodFetch subscriptions appear inside that group.

## Known limitations

The "RSS Podcasts" addon talks to PodFetch on a deliberately narrow
surface: it logs in once for a session cookie, then downloads your
subscriptions as OPML. Everything else — episode play-state sync,
device management, pushing changes back from Kodi — is **not
implemented by the addon itself**, regardless of what PodFetch
supports. The addon is effectively a one-way subscription importer.

| Feature | Kodi calls | PodFetch route | Status |
| --- | --- | --- | --- |
| Login (Basic Auth → session cookie) | `POST /api/2/auth/{user}/login.json` | `login` in `gpodder_api/auth/authentication.rs` | endpoint exists ✓ |
| Import subscriptions (OPML) | `GET /subscriptions/{user}.opml` | `get_simple_subscriptions` in `gpodder_api/subscription/subscriptions.rs` | endpoint exists ✓ |
| Push subscription changes from Kodi | — | — | not supported by the addon |
| Episode play-state / position sync | — | — | not supported by the addon |
| Device list / management | — | — | not supported by the addon |

> The ✓ rows are empirically confirmed working end-to-end against
> recent PodFetch releases. If you hit a problem on either of those
> rows, please report it on
> [issue #372](https://github.com/SamTV12345/PodFetch/issues/372).
> If you want bidirectional sync (episode actions, push), use
> AntennaPod via the [GPodder tutorial](./GPodder.md) — that flow
> exercises the full gpodder API.

## Troubleshooting

### Empty group / 401 after "Import subscriptions to group"

The most common cause is a reverse proxy stripping or failing to
forward the `Authorization` header. PodFetch's Simple API uses HTTP
Basic Auth.

First, confirm the endpoint works directly:

```sh
curl -i -u <user>:<pass> https://<host>/subscriptions/<user>.opml
```

A working response is `HTTP/1.1 200 OK` with
`Content-Type: text/x-opml+xml` and an OPML body. If `curl` works
but Kodi fails, the proxy is stripping `Authorization`.

- **nginx** — make sure no `auth_request` directive overrides the
  upstream auth, and that any `proxy_set_header` block does not
  unset `Authorization`. nginx forwards it by default unless you
  remove it.
- **Traefik** — by default the `Authorization` header is forwarded.
  Check that no custom `headers` middleware lists it under
  `customRequestHeaders` with an empty value (which would strip it).

### "Provider gpodder.net" rejects credentials

Check, in order:

1. `GPODDER_INTEGRATION_ENABLED=true` is set on the PodFetch
   container and the container has been restarted since.
2. The user actually exists. From the host, run
   `docker exec -it <container> /app/podfetch users add` — if it
   reports the user already exists, that confirms the account.
3. The hostname field in Kodi has no scheme prefix (`http://` /
   `https://`) and no trailing slash.

### Import succeeds but the group is empty

PodFetch's OPML endpoint only emits `<outline>` entries for podcasts
the user is currently subscribed to. Add at least one podcast in
PodFetch first (via the web UI or AntennaPod), then re-run *Import
subscriptions to group* in Kodi.

## Reporting issues

If you hit a problem not covered above, please comment on
[issue #372](https://github.com/SamTV12345/PodFetch/issues/372)
and include:

- PodFetch version (e.g. `4.2.2`).
- Kodi version and platform (e.g. `LibreELEC 11.0.3 / RPi3`).
- "RSS Podcasts" addon version.
- Reverse-proxy software and any auth-related config (redacted).
- The relevant excerpt from the PodFetch server log.
