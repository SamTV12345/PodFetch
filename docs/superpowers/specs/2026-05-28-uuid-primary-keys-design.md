# UUID Primary Keys — Design

- **Date:** 2026-05-28
- **Status:** Approved (design); pending implementation plan
- **Author:** SamTV12345 (with Claude)

## Goal

Replace sequential integer primary keys with UUIDs across all PodFetch
entities, to remove resource enumeration (guessing `/api/v1/podcasts/2`,
`/podcasts/3`, …). Preserve backwards compatibility for the durable external
links that embed the old integer ids — specifically podcast RSS feed URLs and
episode deep links — so existing subscriptions keep working.

## Non-goals

- No change to the gPodder sync protocol (it identifies podcasts by feed URL,
  not by internal id).
- No `mysql/` migration work: `crates/podfetch-persistence/src/db.rs` embeds
  only the `sqlite` and `postgres` migration sets, so the `migrations/mysql/`
  directory is neither compiled nor run. It is out of scope.
- No reversible migration. The change is forward-only (see Rollout).

## Decisions (settled during brainstorming)

| Decision | Choice |
|---|---|
| Scope | **All** surrogate-keyed tables get UUID primary keys. Junction tables (e.g. `favorites`, `sessions`, `favorite_podcast_episodes`) keep their composite keys, but each component FK becomes a UUID. |
| UUID version | **UUIDv7** for new ids (time-ordered → good B-tree locality). Existing rows backfilled with random (v4-shaped) UUIDs during the migration. |
| Storage | **TEXT** (36-char canonical form) in both backends. The shared `diesel::MultiConnection` schema makes a single column type mandatory; a native `uuid`/`BLOB` split per backend would fight that architecture. |
| Rust id type | **Plain `uuid::Uuid`** everywhere (one diesel `Text` mapping, validated on read). No per-entity newtypes. |
| API contract | A resource's JSON `id` becomes a **UUID string**. `podcasts` and `podcast_episodes` additionally expose `legacyId: number` (nullable). |
| Backwards compat | `legacy_id INTEGER UNIQUE NULL` retained on **`podcasts`** and **`podcast_episodes`** only. Routes accept either a UUID or the legacy integer. |
| Rollout | **Big-bang**: one migration per backend, one PR, backend + `ui/` + `mobile/` together. |
| Clients in scope | `ui/` (web) **and** `mobile/` (Expo). |

## 1. ID representation & conventions

- Every former `i32` id or FK becomes `uuid::Uuid`, stored as `TEXT`.
- A single diesel `Text` (de)serialization is used for `Uuid` so all columns map
  uniformly across SQLite and Postgres.
- **Generation helper:** `podfetch_domain::ids::new_id() -> Uuid` returns a
  UUIDv7 (`uuid` crate, `v7` feature). All inserts call it explicitly — see §4.
- **Existing rows** are backfilled in the migration with **well-formed random**
  UUIDs (v4-shaped). Legacy rows do not need time-ordering, and this keeps the
  migration pure SQL. Version/variant nibbles are set so `uuid::Uuid` parses
  them cleanly on read.
- **The no-auth standard user.** `STANDARD_USER_ID` (currently the integer
  `9999`, seeded by the FK fix) becomes a **fixed, well-known UUID constant**:
  `00000000-0000-7000-8000-000000009999`. Fixed (not random) so it is stable
  across installs. Used by `read_only_admin_user`, `read_only_admin_id`, and
  `ensure_standard_user_present`.
- **`legacy_id`.** Only `podcasts` and `podcast_episodes` keep their old integer
  as `legacy_id INTEGER UNIQUE NULL`. Frozen at migration time; new rows get
  `NULL` (UUID-only). No sequence/autoincrement on it — which also retires the
  `sqlite_sequence`-drift class of bug for every other table.

## 2. Schema migration (big-bang, pure SQL)

New migration per embedded backend:

- `migrations/sqlite/2026-05-28-120000_uuid_primary_keys/{up,down}.sql`
- `migrations/postgres/2026-05-28-120000_uuid_primary_keys/{up,down}.sql`

The complete table list is enumerated from `crates/podfetch-persistence/src/schema.rs`
at implementation time. It includes the integer-keyed parent tables (`users`,
`podcasts`, `podcast_episodes`, `playlists`, `devices`, `invites`,
`notifications`, `podcast_settings`, `gpodder_settings`, `device_sync_groups`,
and all `audiobookshelf_*` tables) and the junctions/children (`favorites`,
`favorite_podcast_episodes`, `subscriptions`, `episodes`, `listening_events`,
`tags_podcasts`, `filters`, `sessions`, `playlist_items`, …).

**Already-compliant tables are excluded.** Some tables already use a
non-enumerable string PK — notably `tags`, whose `id` was made `TEXT` by the
`username_to_user_id` migration. These keep their PK; only their incoming FK
columns are verified to match the UUID `TEXT` shape (e.g. `tags_podcasts.tag_id`
already references `tags.id`). The implementation step confirms each table's
current PK type from `schema.rs` before deciding to convert it.

**Mechanics reuse the existing `2026-04-15-100000_username_to_user_id` pattern.**

SQLite (`PRAGMA foreign_keys = OFF` for the duration):

1. For each parent table, add `uuid TEXT` and populate it with a well-formed
   random UUID. Generation expression (formatted `8-4-4-4-12`, version nibble
   `4`, variant nibble in `8..b`):
   built from `lower(hex(randomblob(16)))` with the version/variant nibbles
   overwritten via `substr`.
2. For each child FK, add `<fk>_uuid TEXT` and fill it by joining on the old
   integer: `UPDATE child SET x_uuid = (SELECT p.uuid FROM parent p WHERE p.id = child.x)`.
3. Recreate each table with `id TEXT PRIMARY KEY`, FK columns as
   `TEXT REFERENCES parent(id)` (preserving the existing `ON DELETE` behaviour),
   carrying `legacy_id` on podcasts/episodes, copying data, and dropping the old
   integer columns.
4. `PRAGMA foreign_keys = ON`.

Postgres:

1. Add UUID columns (`gen_random_uuid()`), backfill child FK columns via
   `UPDATE … FROM`.
2. Drop old PK/FK constraints and the integer columns.
3. Add the new `id`-as-`TEXT` primary keys and the UUID FK constraints.

**Naming:** the primary-key column stays named `id` everywhere (now `TEXT`) so
`schema.rs` and application code keep referring to `id`. The retired integer
survives only as `legacy_id` on podcasts/episodes.

**Reversibility:** original integer ids for non-legacy tables cannot be
reconstructed, so the migration is forward-only. `down.sql` is a documented
no-op (it may best-effort restore podcasts/episodes from `legacy_id`, but does
not restore the rest). A DB backup before upgrade is mandatory (see Rollout).

## 3. Backwards-compat routing & API contract

**One resolver for every podcast/episode route.** Path params change from
`Path<i32>` to `Path<String>` and go through a shared helper:

```
resolve(segment):
  if segment parses as UUID    -> look up by id
  else if segment parses as i32 -> look up by legacy_id
  else                          -> 400 Bad Request
  (no row found -> 404)
```

Routes adopting it:

- `/rss/{id}` and `/rss/apiKey/{key}/{id}` — the external-subscriber URLs.
- `/api/v1/podcasts/{id}` and all `/podcasts/{id}/{refresh,name,active,settings}`
  plus DELETE.
- `/podcasts/reverse/episode/{id}` and the episode deep-link routes.

**Generated URLs go UUID-first.** `build_podfetch_feed`
(`crates/podfetch-web/src/podcast.rs`) emits `/rss/{uuid}`; old integer URLs
still resolve via `legacy_id`.

**Audiobookshelf.** Item ids become `pod_{uuid}` (and the episode equivalents).
The incoming-id parser accepts both `pod_{uuid}` and the legacy `pod_{int}`
(resolved via `legacy_id`).

**JSON contract** (utoipa annotations updated, then `ui/schema.d.ts`
regenerated):

- `id` becomes a `string` (the UUID) on every DTO.
- `PodcastDto` and `PodcastEpisodeDto` gain `legacyId: number` (nullable).
- Relational fields follow: `podcast_id`, `user_id`, `added_by`, etc. become
  UUID strings.

## 4. Backend code & clients

**ID generation moves into the application.** Inserts no longer rely on
autoincrement; each insert sets `id = new_id()` before writing. The
`UserAdminRepository::ensure_with_id` seam added by the standard-user FK fix
generalizes to "creates always carry their id."

**Backend (`crates/*`):**

- Domain/DTO structs and repository signatures: every `i32` id/FK (`User.id`,
  `Podcast.id`/`added_by`, `*_id`, `podcast_id`, `user_id`, …) → `uuid::Uuid`.
- Auth/session/socket: `User.id: Uuid`; the socket.io room name is the UUID
  string; `sessions.user_id: Uuid`. API-key lookups (keyed by the key string)
  and OIDC/JWT (keyed by username/`sub`) are unaffected.
- Standard user: `STANDARD_USER_ID` becomes the fixed `Uuid` constant; the
  bug-fix tests in `services/user_auth/service.rs` are retyped.
- `schema.rs`: affected columns `Integer` → `Text`; `legacy_id` added on
  podcasts/episodes.

**Clients (same PR):**

- `ui/`: regenerate `schema.d.ts`; `User.id` and podcast/episode id types
  `number` → `string`; remove now-unneeded `String(...)` coercions in URL
  building (`DrawerAudioPlayer`, `EpisodeSearchModal`, `PodcastCard`, …). UI
  links are already string-based, so most route construction is unaffected.
- `mobile/`: the same type/id updates so the Expo app keeps working against the
  new API.

## 5. Testing

Run on **both** SQLite and Postgres.

- **Migration round-trip (critical).** Seed a fixture DB with integer-id data
  across the FK graph (users, podcasts, episodes, favorites, subscriptions,
  listening_events, tags_podcasts, …), run the migration, then assert: every PK
  is a valid UUID; row counts unchanged; **every FK join still resolves to the
  same logical row**; `legacy_id` equals the old integer on podcasts/episodes
  and is NULL elsewhere.
- **Dual-resolution routing.** `/rss/{legacy_int}` and `/rss/{uuid}` both `200`;
  `/podcasts/{legacy_int}` and `/podcasts/{uuid}` resolve to the same podcast;
  malformed id → `400`; unknown id → `404`. Same for the ABS `pod_{int}` vs
  `pod_{uuid}` parser.
- **Contract.** Responses expose `id` as a UUID string and `legacyId` only on
  podcasts/episodes.
- **Existing suite** updated for the type change (the harness's integer-id
  assertions, the standard-user tests), plus `cargo clippy` and an
  OpenAPI-regen check so `schema.d.ts` cannot drift.

## 6. Rollout & risk

- **Forward-only**: mandate a DB backup before upgrade; release note flags the
  breaking API change (`id` is now a string).
- **Big-bang PR**: reviewers should focus on the migration SQL and the resolver,
  which carry the most risk.
- The migration touches the entire FK graph; the round-trip test is the primary
  guard against silent relationship loss.

## Open implementation notes (resolved during planning, not blockers)

- Exact SQLite SQL expression for the well-formed random UUID (nibble masking).
- Whether `down.sql` is a pure no-op or best-effort for podcasts/episodes.
- Final enumeration of `audiobookshelf_*` tables and their FK columns from
  `schema.rs`.
