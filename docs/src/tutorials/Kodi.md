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

The table below maps each Kodi addon feature to the PodFetch route
it depends on.

| Feature | Kodi calls | PodFetch route | Status |
| --- | --- | --- | --- |
| _to be filled in Task 2_ | | | |

> Statuses above are **code-derived** and not yet empirically
> verified end-to-end through Kodi. If a row marked _endpoint exists_
> does not work for you, please report it on
> [issue #372](https://github.com/SamTV12345/PodFetch/issues/372).

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
