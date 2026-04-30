# Kodi Tutorial Page — Design

**Date:** 2026-04-30
**Issue:** [#372](https://github.com/SamTV12345/PodFetch/issues/372)
**Author:** SamTV12345 (with Claude)

## Goal

Document how to use the third-party Kodi addon "RSS Podcasts"
(Heckie75/kodi-addon-podcast) against a PodFetch instance, and
pre-empt the reverse-proxy / Basic Auth class of bug that
drove issue #372.

## Non-goals

- Documenting any Kodi addon other than `Heckie75/kodi-addon-podcast`.
- Writing or maintaining any Kodi-side code.
- Editing the GitHub wiki — mdBook (`docs/src/`) is canonical.
- Adding screenshots in v1 (deferred to a follow-up commit).

## Placement

- **New file:** `docs/src/tutorials/Kodi.md`.
- **`docs/src/SUMMARY.md`:** under the `# Tutorials` heading, add
  `- [Kodi (RSS Podcasts addon)](./tutorials/Kodi.md)` immediately
  after the existing `- [Adding GPodder support](./tutorials/GPodder.md)`
  line, at the same top-level bullet indent.
- **Cross-link:** add a one-line "See also: [Kodi](./Kodi.md)"
  pointer near the top of `docs/src/tutorials/GPodder.md` so
  AntennaPod readers find the Kodi page.

## Page structure

```
# Kodi (RSS Podcasts addon)
## What this addon does
## Prerequisites
## Setup in Kodi
## Known limitations
## Troubleshooting
  ### Empty group / 401 after "Import subscriptions to group"
  ### "Provider gpodder.net" rejects credentials
  ### Import succeeds but group is empty
## Reporting issues
```

Target length: 120–150 lines of Markdown.

### What this addon does

2–3 sentences. States that the addon imports gpodder subscriptions
into a Kodi group via PodFetch's gpodder Simple API. Links to
`https://github.com/Heckie75/kodi-addon-podcast`.

### Prerequisites

Bulleted, each item one line:
- PodFetch reachable from the Kodi machine over HTTP or HTTPS.
- `GPODDER_INTEGRATION_ENABLED=true` — link to
  `GPodder.md#activating-the-gpodder-api`.
- A PodFetch user with username + password (link to
  `GPodder.md#create-a-user-via-the-cli`).

### Setup in Kodi

Numbered list, taken verbatim from `update-freak`'s confirmed flow
in issue #372:

1. Add-ons → Install from repo → All repos → Music → "RSS Podcasts".
2. Open the addon → Configure → Provider: `gpodder.net`.
3. Hostname: bare host only (e.g. `podfetch.example.com`); **no
   scheme, no trailing slash, no path**.
4. Username + password: PodFetch credentials.
5. Back in the addon → "Import subscriptions to group" → pick a
   group name.

### Known limitations

A small Markdown table. v1 rows are derived from a diff between:

- PodFetch routes under `crates/podfetch-web/src/gpodder_api/**/*.rs`
  (use Grep on `#[utoipa::path` / `path=` to enumerate).
- The Kodi addon's API calls in `Heckie75/kodi-addon-podcast`
  (fetch `default.py` and any `gpodder*.py` via WebFetch).

Columns: **Feature** | **Kodi calls** | **PodFetch route** |
**Status**.

Status values:
- `endpoint exists` — PodFetch implements a matching route.
- `endpoint missing` — PodFetch does not implement it; Kodi will
  see a 404.
- `endpoint exists, behavior unverified` — route exists but no
  end-to-end Kodi test confirms it.

Initial rows to populate (pending the diff):
- Subscription OPML import
- Subscription JSON
- Episode actions (GET)
- Episode actions (POST / upload)
- Device list
- Device update

Section ends with a one-line note: "Statuses above are
code-derived and not yet empirically verified through Kodi.
Please report mismatches in [issue #372]."

### Troubleshooting

Three subsections, each ≤6 lines.

#### Empty group / 401 after "Import subscriptions to group"

Most common cause: a reverse proxy is stripping or not forwarding
the `Authorization` header. PodFetch's Simple API uses HTTP Basic
Auth (see `gpodder_api/subscription/subscriptions.rs` and the
`test_simple_api_*` cases).

Verification command — should return `200` with
`Content-Type: text/x-opml+xml`:

```sh
curl -i -u <user>:<pass> https://<host>/subscriptions/<user>.opml
```

If `curl` works but Kodi fails, the proxy is the suspect. Minimal
working snippets:

- **nginx** — confirm `proxy_set_header Authorization $http_authorization;`
  is **not** removed and that no `auth_request` directive is
  shadowing it.
- **Traefik** — ensure no `headers` middleware drops `Authorization`;
  by default it is forwarded.

#### "Provider gpodder.net" rejects credentials

Check, in order:
1. `GPODDER_INTEGRATION_ENABLED=true` is set on the PodFetch
   container.
2. The user exists: `docker exec <id> /app/podfetch users add`
   (re-running shows whether the user is already present).
3. Hostname field has no scheme (`http://`) and no trailing slash.

#### Import succeeds but group is empty

PodFetch's OPML endpoint only emits `<outline>` entries for
podcasts the user is subscribed to. Add at least one podcast in
PodFetch first, then re-run import.

### Reporting issues

One sentence pointing to issue #372. Asks reporters to include:
PodFetch version, Kodi version, addon version, redacted reverse-proxy
config (if any), and the relevant PodFetch server log excerpt.

## Verification

Build the docs locally:

```sh
cd docs && mdbook build
```

Confirm:
- `docs/book/tutorials/Kodi.html` is generated.
- The new `SUMMARY.md` entry renders in the sidebar.
- The "See also" link in `GPodder.md` resolves.
- All intra-doc anchor links resolve (no broken-link warnings
  from `mdbook build`).

## Out of scope / follow-ups

- **Screenshots:** add `kodi_install.png`, `kodi_configure.png`,
  `kodi_import_group.png` in a follow-up PR after capturing them
  on a real Kodi instance.
- **Empirical verification pass:** once `update-freak` (or any
  user) confirms a feature works end-to-end, promote that row's
  status from `endpoint exists, behavior unverified` to a
  ✓ marker, and remove the "code-derived" footer once the table
  is fully verified.
- **Closing issue #372:** depends on `update-freak`'s
  reverse-proxy retest, not on this doc PR.
