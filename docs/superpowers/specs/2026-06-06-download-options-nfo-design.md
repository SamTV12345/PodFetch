# Download Options: NFO Files, Numbering Token & Jellyfin Polish — Design

**Issue:** [#315 — More download options (folder structure, images, .nfo files etc)](https://github.com/SamTV12345/PodFetch/issues/315)
**Date:** 2026-06-06
**Status:** Approved (ready for implementation plan)

## 1. Background & problem

Issue #315 (open since 2023, last comment 2025-06) is a cluster of requests around how
downloaded podcast files are organized on disk, framed mostly around Jellyfin
compatibility. An audit of the current code shows **most of the original request is
already shipped**:

- **Configurable naming templates** — `episode_format` / `podcast_format` with tokens
  `{title}`, `{date}`, `{guid}`, `{url}`, `{description}`, `{duration}` (global +
  per-podcast override), via the `strfmt` crate.
- **Flat Jellyfin-style layout** — `direct_paths = true` produces
  `Podcast/Episode Title.mp3` with the episode image written as
  `Podcast/Episode Title.jpg` beside it (`podfetch-storage/src/filename.rs:67`).
- **pubdate ordering** — users can already use `{date} - {title}`.
- **Skip duplicate episode images** — `use_one_cover_for_all_episodes = true` skips
  downloading/writing per-episode images entirely (added 2026-05, after the duplicate-cover
  complaint).
- **Sanitization** of illegal filename characters via `replacement_strategy`.

The **genuine remaining gaps** this design addresses:

1. **`.nfo` metadata files** — nothing exists today (only embedded ID3/MP4 tags). This is
   the headline ask and the one item never addressed.
2. **Episode numbering as a filename token** — today `episode_numbering` only prepends the
   number to the embedded *title tag*; it never changes the filename, which is why it
   "does nothing" in a file browser / Jellyfin folder view. It is also applied for MP3 only;
   the MP4 writer skips it.
3. **Jellyfin cover filename** — the cover is hardcoded to `image.<ext>`; Jellyfin's scanner
   auto-detects art only from `cover.*` / `folder.*` / `poster.*`.

### Approved scope

Full scope: NFO generation (configurable per podcast), the `{episodeNumber}` filename token,
the MP4 numbering fix, and a configurable cover filename. Backfill of existing libraries
included.

### Key decisions (from brainstorming)

- **NFO format:** configurable — `nfo_format = "off" | "tvshow" | "album"` (global + per-podcast
  override). Both writers are built.
- **Generation/sync:** on each download + regenerate on podcast settings change + a backfill
  rescan for already-downloaded episodes (no audio re-download).
- **Cover filename:** configurable, default `image` (existing libraries untouched); backfill
  renames on change. Not auto-derived, not a forced migration.
- **Integration:** dedicated `nfo` service module with pure builders + a thin, non-fatal writer
  (Approach A), invoked at the download / reapply / backfill sites.

## 2. Data model / settings

Two new columns, added to **both** `settings` (global) and `podcast_settings` (per-podcast),
following the `use_one_cover_for_all_episodes` / `replacement_strategy` precedent:

| Column | Type | Default | Meaning |
|---|---|---|---|
| `nfo_format` | `TEXT NOT NULL` | `'off'` | `'off'` \| `'tvshow'` \| `'album'`. Parsed into `NfoFormat` via `FromStr`. Default `off` ⇒ opt-in; no change for existing users. |
| `cover_filename` | `TEXT NOT NULL` | `'image'` | Base name of the podcast cover file. Default `image` ⇒ existing libraries untouched. |

**Resolution rule** (unchanged pattern): if a `podcast_settings` row exists *and*
`activated = true`, use its values; otherwise fall back to global `settings`
(`.map(...).unwrap_or(global)`, as at `download/service.rs:210`).

**Migrations** target **sqlite + postgres only** (MySQL is not a maintained backend — 3
migrations vs 48 sqlite / 33 postgres; neither SponsorBlock nor `use_one_cover` was added
there). Two `ALTER TABLE ... ADD COLUMN` per backend, in
`migrations/{sqlite,postgres}/<ts>_nfo_options/{up,down}.sql`.

**Propagation touch-list** (standard for a new setting, identical to SponsorBlock T3):
`schema.rs` `table!` blocks (both tables) → domain `Setting` + `PodcastSetting` (+ `NfoFormat`
enum) → persistence entities/repos → web DTOs (`settings.rs`, `podcast_settings.rs`, with
`#[serde(default)]`) → UI settings + per-podcast forms → `ui/schema.d.ts` + `mobile/schema.d.ts`
(hand-edited).

## 3. NFO module

`crates/podfetch-web/src/services/nfo/`:

- `mod.rs` — `NfoFormat` enum (`Off`/`Tvshow`/`Album`) + `FromStr`; format dispatch.
- `builders.rs` — **pure** functions returning XML strings via `quick_xml::Writer` (indented).
  No DB, no FS ⇒ golden-XML unit tests. `quick_xml` is already a dependency (the audiobookshelf
  importer parses NFO with it).
- `service.rs` — resolves format from settings, computes paths, writes files; **non-fatal**
  (warn + continue, exactly like the SponsorBlock hook at `download/service.rs:327`).

### Available metadata

- **Podcast:** `name`, `summary`, `language`, `explicit`, `keywords`, `author`, `guid`,
  `last_build_date`.
- **Episode:** `name`, `description`, `date_of_recording` (string), `total_time` (seconds),
  `guid`, `url`.
- "Persons" maps to the podcast-level `author` (no per-episode person list / `podcast:person`
  parsing exists). No `itunes:episode` / `itunes:season` is parsed, so episode/track numbers
  come from the computed date-position only; there is no season concept.

### Builders / output

`build_tvshow_nfo(&Podcast)` → **`tvshow.nfo`** at the podcast root:

```xml
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<tvshow>
  <title>{podcast.name}</title>
  <plot>{podcast.summary}</plot>          <!-- omitted if None -->
  <studio>{podcast.author}</studio>       <!-- omitted if None -->
  <genre>{podcast.keywords}</genre>       <!-- omitted if None -->
  <uniqueid type="podfetch">{podcast.guid}</uniqueid>
</tvshow>
```

`build_episodedetails_nfo(&Podcast, &PodcastEpisode, position)` → per-episode
**`<audio-basename>.nfo`** (extension swapped from the resolved audio path ⇒ correct in both
flat and nested layouts):

```xml
<episodedetails>
  <title>{episode.name}</title>
  <showtitle>{podcast.name}</showtitle>
  <season>1</season>                            <!-- no season data; constant 1 -->
  <episode>{position}</episode>                 <!-- get_position_of_episode -->
  <plot>{episode.description}</plot>
  <aired>{YYYY-MM-DD from date_of_recording}</aired>
  <runtime>{total_time/60, rounded}</runtime>   <!-- Kodi runtime = minutes -->
  <actor><name>{podcast.author}</name></actor>  <!-- "persons"; omitted if None -->
  <uniqueid type="podfetch">{episode.guid}</uniqueid>
</episodedetails>
```

`build_album_nfo(&Podcast, &[(PodcastEpisode, position)])` → single **`album.nfo`** at the
root (no per-episode files); rewritten whenever an episode is added:

```xml
<album>
  <title>{podcast.name}</title>
  <artist>{podcast.author}</artist>       <!-- omitted if None -->
  <genre>{podcast.keywords}</genre>
  <review>{podcast.summary}</review>
  <track><position>{n}</position><title>{episode.name}</title><duration>{total_time}</duration></track>
  <!-- one per downloaded episode, ordered by date -->
</album>
```

### Notes

- **Escaping** handled by `quick_xml` (`BytesText` auto-escapes `& < > " '`); descriptions with
  `&`/HTML are safe. Descriptions are written as-is (escaped); HTML-stripping is out of scope.
- **Dispatch:** `off` → nothing; `tvshow` → per-episode `.nfo` + refresh `tvshow.nfo`;
  `album` → rewrite `album.nfo` from the full downloaded set (no per-episode files).
- `position` reuses `get_position_of_episode`; album `<position>`/ordering reuses
  `get_track_number_for_episode`.

## 4. Wiring & backfill

Three invocation sites — all best-effort / non-fatal (warn-and-continue, never break a
download or settings save):

1. **Download path** — in `download_podcast_episode`, after audio + image are written and after
   `handle_metadata_insertion`. Resolve `nfo_format` (per-podcast → global), then:
   - `tvshow` → write this episode's `<basename>.nfo` (position via `get_position_of_episode`)
     and overwrite `tvshow.nfo` (idempotent).
   - `album` → rewrite `album.nfo` from the podcast's full downloaded episode set.
   - Per-episode `.nfo` path = the resolved audio path with its extension swapped to `.nfo`
     (works in both layouts).
2. **Settings-change reapply path** — `podcast_settings/service.rs:64-95` already re-applies
   metadata to every downloaded episode when per-podcast settings change. Hook NFO regeneration
   + cover-rename here so flipping `nfo_format` / `cover_filename` updates the existing library.
3. **Backfill rescan option** — add `regenerate_nfo` to `episode_rescan` (which already carries
   `refetch_sponsorblock` and `apply_covers`). Per podcast, (re)write all NFO files and rename
   the cover if needed, **without re-downloading audio**. This is the "organize my existing
   collection" path.

**Cover-filename wiring:**

- Thread the resolved `cover_filename` into `build_podcast_image_paths`
  (`podfetch-storage/src/path.rs`), replacing the hardcoded `image`, so new downloads write
  `{dir}/{cover_filename}.{ext}`.
- **Rename-on-change:** when the resolved cover name differs from what's on disk (reapply or
  backfill), rename `{dir}/{old}.{ext}` → `{dir}/{cover_filename}.{ext}` and update the stored
  image path via the existing `update_*image*` repo methods. The old name is found from the
  podcast's recorded cover path.

**Concurrency:** `album.nfo` is rebuilt from the committed downloaded set on each write, so
concurrent same-podcast downloads are eventually consistent (last writer includes all-so-far;
the reapply/backfill path fully reconciles). `tvshow.nfo` writes are idempotent overwrites.

## 5. Numbering token, MP4 fix, cover setting

**`{episodeNumber}` filename token** (the real fix for "numbering does nothing"):

- Added to `perform_episode_variable_replacement` (`file/service.rs:340`). Value = the episode's
  **1-based position** = `get_position_of_episode(date, podcast_id)`, so the filename number and
  the embedded title number agree.
- That function is intentionally DB-free, so `episode_number` is **threaded in as a parameter**
  (computed by the caller — a query already done nearby in the download path). The signature
  change ripples to its callers (download + reapply/rescan), each of which has `podcast_id`.
- Inserted into `vars` as `episodeNumber`; users write `{episodeNumber}` in `episode_format`.
  **Zero-padding works via `strfmt` specs** — `{episodeNumber:0>3}` → `007` (survives the
  existing comma-strip/trim). No second token (YAGNI).
- **Independent** of the per-podcast `episode_numbering` bool: the token numbers *filenames*
  (works globally via `episode_format`); `episode_numbering` numbers the *embedded title tag*.
  Keeping them separate is deliberate.

**MP4 numbering fix** (`update_meta_data_mp4:558`): currently always `set_title(name)`. Extract
the title decision shared by both writers — `resolve_episode_title(episode, podcast, index)
-> (String, bool /*processed*/)` — and call it from both `update_meta_data_mp3` and
`update_meta_data_mp4`, so M4A episodes get numbered titles identically and the paths can't drift.

**Scope guard:** no global `episode_numbering` bool is added. The per-podcast title-tag toggle
stays (now MP4-correct); the global filename-numbering need is served by `{episodeNumber}` in
the global `episode_format`.

**Edge case:** episodes sharing an identical `date_of_recording` get the same position (a tie) —
the same behavior the existing numbering already has; acceptable.

## 6. Testing

Most logic is pure ⇒ covered without DB or FS:

| Test | Kind | Covers |
|---|---|---|
| NFO builder golden-XML | pure unit (`builders.rs`) | exact `tvshow`/`episodedetails`/`album` output; `None`-field omission; escaping (`&`, `<`); runtime sec→min; album multi-track ordering |
| `NfoFormat::from_str` | pure unit | `off`/`tvshow`/`album` + unknown → default |
| `perform_episode_variable_replacement` w/ `{episodeNumber}` | pure unit | plain + `{episodeNumber:0>3}` |
| `resolve_episode_title` | pure unit | enabled / disabled / no-settings / already-processed → title + `processed` |
| `.nfo` path derivation | pure unit | flat → `{stem}.nfo`, nested → `audio.nfo` |
| settings + podcast_settings round-trip | persistence (`--features sqlite`) | new columns default + update persists; uses crate-wide `db::test_db::setup()` lock |
| settings DTO serde | web unit | new fields (de)serialize with `#[serde(default)]` |

**Explicit non-goals (no silent gaps):**

- No full network download→file-on-disk integration test (heavy, live audio). Download path is
  covered via the unit-tested builder + the non-fatal call-site pattern (as with SponsorBlock).
- MP4 tag *writing* isn't unit-tested (needs an `.m4a` fixture); the **numbering decision** is,
  via `resolve_episode_title`.

**Verification gate** (final plan task, as SponsorBlock T12): `clippy -D warnings` on default /
`--features sqlite` / `--features postgresql`; tests on sqlite + postgres; frontend `tsc` +
build; `schema.d.ts` consistency. **No `cargo fmt`** (repo convention).

## 7. Out of scope

- HTML-stripping of descriptions inside NFO `<plot>`/`<review>` (written escaped; a later nicety).
- Global `episode_numbering` toggle (served by the `{episodeNumber}` template token).
- MySQL migrations (not a maintained backend).
- Per-episode person/host lists / `podcast:person` parsing (only podcast-level `author` exists).
- A full network download integration test.
