# Kodi Tutorial Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `docs/src/tutorials/Kodi.md` page documenting the Heckie75 "RSS Podcasts" Kodi addon against PodFetch, with a code-derived limitations table and reverse-proxy / Basic Auth troubleshooting that pre-empts issue #372.

**Architecture:** Pure documentation change — one new mdBook page, one `SUMMARY.md` line, one cross-link line in the existing GPodder tutorial. Limitations table is sourced by diffing PodFetch's gpodder routes (in `crates/podfetch-web/src/gpodder_api/`) against the Kodi addon's API calls (`Heckie75/kodi-addon-podcast`). Verification is `mdbook build` succeeding with no warnings.

**Tech Stack:** Markdown, mdBook (`docs/book.toml`), Git.

**Spec:** `docs/superpowers/specs/2026-04-30-kodi-tutorial-design.md`

---

### Task 1: Create the Kodi.md skeleton with all sections

**Files:**
- Create: `docs/src/tutorials/Kodi.md`

- [ ] **Step 1: Create the file with all sections except the limitations table body**

Create `docs/src/tutorials/Kodi.md` with this content:

````markdown
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
````

- [ ] **Step 2: Commit the skeleton**

```sh
git add docs/src/tutorials/Kodi.md
git commit -m "docs: add Kodi tutorial page (skeleton)

Setup walkthrough and troubleshooting sections for the Heckie75
RSS Podcasts Kodi addon. Limitations table populated in a follow-up.

Refs #372"
```

---

### Task 2: Populate the "Known limitations" table

**Files:**
- Modify: `docs/src/tutorials/Kodi.md` (replace the placeholder table row)

- [ ] **Step 1: Enumerate PodFetch's gpodder routes**

Run:

```sh
grep -rn '#\[utoipa::path' crates/podfetch-web/src/gpodder_api/
grep -rn 'path *= *"' crates/podfetch-web/src/gpodder_api/
```

Also read these files end-to-end to confirm the routes the routers
actually expose (a `#[utoipa::path]` may be present on a handler
that is not wired in):

- `crates/podfetch-web/src/gpodder_api/subscription/subscriptions.rs`
- `crates/podfetch-web/src/gpodder_api/episodes/gpodder_episodes.rs`
- `crates/podfetch-web/src/gpodder_api/device/device_controller.rs`
- `crates/podfetch-web/src/gpodder_api/auth/authentication.rs`
- `crates/podfetch-web/src/gpodder_api/settings/settings_controller.rs`
- `crates/podfetch-web/src/gpodder_api/mod.rs` (router wiring)

Record route → handler pairs in scratch notes. Note which routes
accept Basic Auth (Simple API, mounted at `/subscriptions/...`)
versus session/cookie auth (Advanced API at `/api/2/...`).

- [ ] **Step 2: Enumerate the Kodi addon's API calls**

Fetch the addon source. Use WebFetch on these URLs and grep for
`urllib`, `requests`, `gpodder`, `subscriptions`, `episodes`,
`devices`:

- `https://github.com/Heckie75/kodi-addon-podcast/blob/master/default.py`
- `https://github.com/Heckie75/kodi-addon-podcast/tree/master/resources/lib`

For each HTTP call the addon makes, record: method, URL template,
and which Kodi feature triggers it (e.g. *Import subscriptions to
group* → `GET /subscriptions/<user>.opml`).

- [ ] **Step 3: Build the table**

Replace the placeholder row in `docs/src/tutorials/Kodi.md` (the
single `_to be filled in Task 2_` row) with one row per Kodi feature
discovered in Step 2. Use exactly these status values:

- `endpoint exists` — PodFetch implements a matching route.
- `endpoint missing` — PodFetch does not implement it; Kodi will
  see a 404.
- `endpoint exists, behavior unverified` — route is present but no
  end-to-end Kodi test has confirmed it.

Expected rows (verify each against the Step 1 / Step 2 evidence
before committing — do not copy this list blindly):

| Feature | Kodi calls | PodFetch route | Status |
| --- | --- | --- | --- |
| Import subscriptions (OPML) | `GET /subscriptions/{user}.opml` | `get_simple_subscriptions` in `gpodder_api/subscription/subscriptions.rs` | endpoint exists |
| Import subscriptions (JSON) | `GET /subscriptions/{user}.json` | `get_simple_subscriptions` | endpoint exists |
| Push subscriptions | `PUT /subscriptions/{user}/{device}.{fmt}` | `put_simple_subscriptions` / `put_device_subscriptions` | endpoint exists, behavior unverified |
| Episode actions (read) | `GET /api/2/episodes/{user}` | check `gpodder_api/episodes/gpodder_episodes.rs` | _fill from Step 1_ |
| Episode actions (write) | `POST /api/2/episodes/{user}` | check `gpodder_api/episodes/gpodder_episodes.rs` | _fill from Step 1_ |
| Device list / update | `GET`/`POST /api/2/devices/{user}` | check `gpodder_api/device/device_controller.rs` | _fill from Step 1_ |

If a row does not apply (the addon does not call that endpoint),
omit it. If the addon calls an endpoint not listed above, add it.

- [ ] **Step 4: Commit the populated table**

```sh
git add docs/src/tutorials/Kodi.md
git commit -m "docs: populate Kodi limitations table from route diff

Code-derived from a diff between PodFetch's gpodder_api routes and
the Heckie75 RSS Podcasts addon's API calls. Empirical verification
remains as a follow-up.

Refs #372"
```

---

### Task 3: Wire the new page into SUMMARY.md

**Files:**
- Modify: `docs/src/SUMMARY.md`

- [ ] **Step 1: Read the current SUMMARY.md and confirm the anchor line**

Run:

```sh
grep -n 'GPodder' docs/src/SUMMARY.md
```

Expected output (line number may differ):

```
20:- [Adding GPodder support](./tutorials/GPodder.md)
```

- [ ] **Step 2: Insert the Kodi entry directly after the GPodder line**

Edit `docs/src/SUMMARY.md` so the `# Tutorials` block reads:

```markdown
# Tutorials

- [Setting up basic auth](./tutorials/BasicAuth.md)
- [Setting up OIDC](./tutorials/OIDC.md)
- [Adding GPodder support](./tutorials/GPodder.md)
- [Kodi (RSS Podcasts addon)](./tutorials/Kodi.md)
```

- [ ] **Step 3: Commit**

```sh
git add docs/src/SUMMARY.md
git commit -m "docs: link Kodi tutorial from SUMMARY"
```

---

### Task 4: Add the cross-link from the GPodder tutorial

**Files:**
- Modify: `docs/src/tutorials/GPodder.md` (insert one line after the `# GPodder API` heading)

- [ ] **Step 1: Insert the See-also line**

Open `docs/src/tutorials/GPodder.md`. Immediately after the
`# GPodder API` heading on line 1, insert a blank line and then:

```markdown
> **See also:** [Kodi (RSS Podcasts addon)](./Kodi.md) for using
> PodFetch with the Kodi RSS Podcasts addon.
```

The first three lines of the file should now read:

```markdown
# GPodder API

> **See also:** [Kodi (RSS Podcasts addon)](./Kodi.md) for using
> PodFetch with the Kodi RSS Podcasts addon.
```

- [ ] **Step 2: Commit**

```sh
git add docs/src/tutorials/GPodder.md
git commit -m "docs: cross-link Kodi tutorial from GPodder page"
```

---

### Task 5: Build the docs and resolve warnings

**Files:**
- None (verification only)

- [ ] **Step 1: Build mdBook**

```sh
cd docs && mdbook build
```

Expected: exit code 0, no `WARN` lines about broken links. If
`mdbook` is not installed, install it first with
`cargo install mdbook` (or document in the PR description that
the maintainer should run the build).

- [ ] **Step 2: Confirm the new page rendered**

```sh
ls docs/book/tutorials/Kodi.html
```

Expected: file exists. Open it in a browser (or `start` /
`xdg-open`) and visually confirm:

- The sidebar shows the new "Kodi (RSS Podcasts addon)" entry under
  the *Tutorials* group.
- All intra-doc links in the Prerequisites section
  (`./GPodder.md#...`) resolve and scroll to the right anchor.
- The "See also" callout at the top of the GPodder page links to
  the Kodi page.
- The limitations table renders with one row per feature.

- [ ] **Step 3: Fix any broken-link warnings**

If `mdbook build` reported warnings, fix them in the source
Markdown and rebuild until the build is clean. Common cause:
anchor slugs differ from heading text — adjust the link target
to match the slug mdBook generated (visible in the rendered
HTML's `id` attributes).

- [ ] **Step 4: Commit any fixes**

If you made fix-up edits in Step 3:

```sh
git add docs/src/
git commit -m "docs: fix Kodi tutorial link anchors"
```

If the build was clean on first try, skip the commit.

---

### Task 6: Final verification and PR-ready commit log

**Files:**
- None (verification only)

- [ ] **Step 1: Confirm the commit log is sensible**

```sh
git log --oneline main..HEAD
```

Expected: a small, ordered series of `docs:` commits, each scoped
to one task. No fix-ups that should have been squashed into the
same commit.

- [ ] **Step 2: Confirm the working tree is clean**

```sh
git status
```

Expected: `nothing to commit, working tree clean`.

- [ ] **Step 3: Confirm there are no stray edits outside `docs/`**

```sh
git diff --stat main..HEAD
```

Expected: every changed path starts with `docs/`. If any other
path changed, investigate before opening the PR.

---
