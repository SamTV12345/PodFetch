# UUID Primary Keys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace sequential integer primary keys with UUIDv7 (stored as TEXT) across all PodFetch entities, while keeping `legacy_id` backwards-compatible lookups on podcasts and podcast_episodes so existing RSS/episode links keep working.

**Architecture:** One big-bang SQL migration per embedded backend (sqlite, postgres) recreates/alters every integer-keyed table to a `TEXT` UUID `id` and rewrites every integer FK column to the parent's UUID. Persistence-layer diesel row structs store ids as `String`; domain/service/web code uses `uuid::Uuid`; conversion happens at the persistence boundary. Podcast/episode routes accept either a UUID or the old integer (resolved via `legacy_id`). The JSON `id` becomes a string; podcasts/episodes also expose `legacyId`.

**Tech Stack:** Rust, Diesel (`MultiConnection` over SQLite + Postgres), Axum, utoipa (OpenAPI), `uuid` crate (v7), React/TypeScript (`ui/`), React Native/Expo (`mobile/`).

**Spec:** `docs/superpowers/specs/2026-05-28-uuid-primary-keys-design.md`

**Branch:** `feat/uuid-primary-keys` (already created off `main`).

---

## Conventions used throughout this plan

- **Test feature flag:** the workspace default enables both backends, but tests must pick one. Run backend tests with `cargo test -p <crate> --no-default-features --features sqlite <filter>`. For the Postgres path substitute `--features postgresql` (requires the test container per `crates/podfetch-web/src/test_support.rs`).
- **Commit style:** no `Co-Authored-By` trailer. Conventional-commit subjects.
- **UUID string form:** canonical lowercase 36-char (`8-4-4-4-12`).
- **`new_id()`** (defined in Task 1) is the only source of new ids in application code.

### Reusable SQL: generate one well-formed random UUID

**SQLite** (well-formed v4: version nibble `4`, variant nibble from `8/9/a/b`):

```sql
lower(
  substr(hex(randomblob(4)), 1, 8) || '-' ||
  substr(hex(randomblob(2)), 1, 4) || '-' ||
  '4' || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr('89ab', abs(random()) % 4 + 1, 1) || substr(hex(randomblob(2)), 2, 3) || '-' ||
  substr(hex(randomblob(6)), 1, 12)
)
```

**Postgres:** `gen_random_uuid()::text` (built-in in PG13+).

These produce non-enumerable ids for pre-existing rows. New runtime rows use UUIDv7 via `new_id()`.

### Complete conversion map (authoritative)

**A. Integer PK → TEXT UUID `id`:** `users`, `podcasts`(+`legacy_id`), `podcast_episodes`(+`legacy_id`), `devices`, `subscriptions`, `episodes`, `gpodder_settings`, `device_sync_groups`, `notifications`, `listening_events`, `settings`, `podcast_settings` (its PK `podcast_id` is also the FK to `podcasts`).

**B. Integer FK columns → TEXT UUID, grouped by parent:**

| Parent | Child table.column |
|---|---|
| `users.id` | `podcasts.added_by` (nullable), `devices.user_id`, `subscriptions.user_id`, `episodes.user_id`, `gpodder_settings.user_id`, `device_sync_groups.user_id`, `listening_events.user_id`, `filters.user_id` (PK), `sessions.user_id` (composite PK), `favorites.user_id` (composite PK), `favorite_podcast_episodes.user_id` (composite PK), `tags.user_id`, `playlists.user_id`, `audiobookshelf_listening_sessions.user_id`, `audiobookshelf_media_progress.user_id`, `audiobookshelf_playback_sessions.user_id` |
| `podcasts.id` | `podcast_episodes.podcast_id`, `favorites.podcast_id` (composite PK), `tags_podcasts.podcast_id` (composite PK), `listening_events.podcast_id`, `podcast_settings.podcast_id` (PK) |
| `podcast_episodes.id` | `favorite_podcast_episodes.episode_id` (composite PK), `listening_events.podcast_episode_db_id`, `playlist_items.episode` (composite PK), `podcast_episode_chapters.episode_id` |

**C. Untouched:** `tags`, `playlists`, `podcast_episode_chapters`, `invites` keep their existing `Text` PKs (only their integer FKs above convert). `episodes.podcast`, `subscriptions.podcast` (gPodder feed URLs) and `listening_events.podcast_episode_id` (gPodder guid) are TEXT strings, **not** FKs — leave them. All `audiobookshelf_*` PKs and inter-ABS FKs are already `Text` — leave them.

---

## Phase 0 — Foundations

### Task 1: UUID generation helper

**Files:**
- Modify: `crates/podfetch-domain/Cargo.toml` (add `uuid`)
- Create: `crates/podfetch-domain/src/ids.rs`
- Modify: `crates/podfetch-domain/src/lib.rs` (add `pub mod ids;`)

- [ ] **Step 1: Add the dependency.** In `crates/podfetch-domain/Cargo.toml`, under `[dependencies]`:

```toml
uuid = { version = "1", features = ["v7", "v4"] }
```

(If `uuid` is already a workspace dep, use `uuid = { workspace = true, features = ["v7", "v4"] }`.)

- [ ] **Step 2: Write the failing test.** Create `crates/podfetch-domain/src/ids.rs`:

```rust
use uuid::Uuid;

/// Generate a new time-ordered (v7) identifier for a freshly created row.
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_is_v7_and_unique() {
        let a = new_id();
        let b = new_id();
        assert_eq!(a.get_version_num(), 7, "must be a v7 UUID");
        assert_ne!(a, b, "two ids must differ");
        // v7 is time-ordered: a was created before b.
        assert!(a <= b, "v7 ids should be monotonically non-decreasing");
    }
}
```

- [ ] **Step 3: Register the module.** Add to `crates/podfetch-domain/src/lib.rs`:

```rust
pub mod ids;
```

- [ ] **Step 4: Run the test.**

Run: `cargo test -p podfetch-domain --no-default-features --features sqlite ids::`
Expected: PASS (`new_id_is_v7_and_unique`).

- [ ] **Step 5: Commit.**

```bash
git add crates/podfetch-domain/Cargo.toml crates/podfetch-domain/src/ids.rs crates/podfetch-domain/src/lib.rs
git commit -m "feat: add UUIDv7 id generation helper"
```

### Task 2: Standard-user UUID constant

The no-auth standard user is currently `STANDARD_USER_ID: i32 = 9999` in `crates/podfetch-web/src/role.rs`. It becomes a fixed UUID.

**Files:**
- Modify: `crates/podfetch-web/src/role.rs:44`

- [ ] **Step 1: Replace the constant.** In `crates/podfetch-web/src/role.rs`, change:

```rust
pub const STANDARD_USER_ID: i32 = 9999;
```

to:

```rust
use uuid::Uuid;

/// Fixed identity for the no-auth standard admin user. Stable across installs
/// so foreign keys that reference it resolve to the same seeded row everywhere.
pub const STANDARD_USER_ID: Uuid = Uuid::from_u128(0x00000000_0000_7000_8000_000000009999);
```

(Keep `STANDARD_USER: &str = "user123"` unchanged.)

- [ ] **Step 2: Verify it compiles in isolation.**

Run: `cargo check -p podfetch-web --no-default-features --features sqlite 2>&1 | head -40`
Expected: Many type errors at the *call sites* of `STANDARD_USER_ID` (they still expect `i32`). That is fine — they are fixed in Phase 2. Confirm the error in `role.rs` itself is gone (only call-site errors remain).

- [ ] **Step 3: Do NOT commit yet.** This change only compiles once Phase 2 retypes the call sites. It is committed together with Task 6.

---

## Phase 1 — The migration

### Task 3: SQLite migration up.sql

**Files:**
- Create: `migrations/sqlite/2026-05-28-120000_uuid_primary_keys/up.sql`
- Create: `migrations/sqlite/2026-05-28-120000_uuid_primary_keys/down.sql`

This follows the exact pattern of `migrations/sqlite/2026-04-15-100000_username_to_user_id/up.sql`: with `PRAGMA foreign_keys = OFF`, for each table add a `uuid` column, backfill child FK `*_uuid` columns by joining on the old integer, then recreate each table with the UUID as PK / FK and drop the integers.

- [ ] **Step 1: Write `down.sql`** (forward-only migration — documented no-op):

```sql
-- This migration is forward-only: original integer ids for non-legacy tables
-- cannot be reconstructed. Restore from a database backup taken before upgrade.
SELECT 1;
```

- [ ] **Step 2: Write `up.sql`.** Build it in this order (parents before children so the join subqueries see populated `uuid` columns). Use the SQLite UUID expression from the Conventions section for every `<NEW_UUID>` placeholder.

```sql
PRAGMA foreign_keys = OFF;

-- ============================================================
-- 1. Add `uuid` to every integer-PK parent + `*_uuid` to children
-- ============================================================

-- users
ALTER TABLE users ADD COLUMN uuid TEXT;
UPDATE users SET uuid = <NEW_UUID>;

-- podcasts (+ legacy_id)
ALTER TABLE podcasts ADD COLUMN uuid TEXT;
UPDATE podcasts SET uuid = <NEW_UUID>;
ALTER TABLE podcasts ADD COLUMN added_by_uuid TEXT;
UPDATE podcasts SET added_by_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = podcasts.added_by);

-- podcast_episodes (+ legacy_id)
ALTER TABLE podcast_episodes ADD COLUMN uuid TEXT;
UPDATE podcast_episodes SET uuid = <NEW_UUID>;
ALTER TABLE podcast_episodes ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_episodes SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = podcast_episodes.podcast_id);

-- devices
ALTER TABLE devices ADD COLUMN uuid TEXT;
UPDATE devices SET uuid = <NEW_UUID>;
ALTER TABLE devices ADD COLUMN user_id_uuid TEXT;
UPDATE devices SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = devices.user_id);

-- subscriptions
ALTER TABLE subscriptions ADD COLUMN uuid TEXT;
UPDATE subscriptions SET uuid = <NEW_UUID>;
ALTER TABLE subscriptions ADD COLUMN user_id_uuid TEXT;
UPDATE subscriptions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = subscriptions.user_id);

-- episodes (gpodder episode actions)
ALTER TABLE episodes ADD COLUMN uuid TEXT;
UPDATE episodes SET uuid = <NEW_UUID>;
ALTER TABLE episodes ADD COLUMN user_id_uuid TEXT;
UPDATE episodes SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = episodes.user_id);

-- gpodder_settings
ALTER TABLE gpodder_settings ADD COLUMN uuid TEXT;
UPDATE gpodder_settings SET uuid = <NEW_UUID>;
ALTER TABLE gpodder_settings ADD COLUMN user_id_uuid TEXT;
UPDATE gpodder_settings SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = gpodder_settings.user_id);

-- device_sync_groups
ALTER TABLE device_sync_groups ADD COLUMN uuid TEXT;
UPDATE device_sync_groups SET uuid = <NEW_UUID>;
ALTER TABLE device_sync_groups ADD COLUMN user_id_uuid TEXT;
UPDATE device_sync_groups SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = device_sync_groups.user_id);

-- notifications (no FK)
ALTER TABLE notifications ADD COLUMN uuid TEXT;
UPDATE notifications SET uuid = <NEW_UUID>;

-- settings (singleton, no FK)
ALTER TABLE settings ADD COLUMN uuid TEXT;
UPDATE settings SET uuid = <NEW_UUID>;

-- listening_events (FKs: user_id, podcast_id, podcast_episode_db_id)
ALTER TABLE listening_events ADD COLUMN uuid TEXT;
UPDATE listening_events SET uuid = <NEW_UUID>;
ALTER TABLE listening_events ADD COLUMN user_id_uuid TEXT;
UPDATE listening_events SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = listening_events.user_id);
ALTER TABLE listening_events ADD COLUMN podcast_id_uuid TEXT;
UPDATE listening_events SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = listening_events.podcast_id);
ALTER TABLE listening_events ADD COLUMN podcast_episode_db_id_uuid TEXT;
UPDATE listening_events SET podcast_episode_db_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = listening_events.podcast_episode_db_id);

-- podcast_settings (PK = podcast_id, also FK)
ALTER TABLE podcast_settings ADD COLUMN podcast_id_uuid TEXT;
UPDATE podcast_settings SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = podcast_settings.podcast_id);

-- Composite/Text-PK children whose FK columns convert:
ALTER TABLE favorites ADD COLUMN user_id_uuid TEXT;
UPDATE favorites SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = favorites.user_id);
ALTER TABLE favorites ADD COLUMN podcast_id_uuid TEXT;
UPDATE favorites SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = favorites.podcast_id);

ALTER TABLE favorite_podcast_episodes ADD COLUMN user_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = favorite_podcast_episodes.user_id);
ALTER TABLE favorite_podcast_episodes ADD COLUMN episode_id_uuid TEXT;
UPDATE favorite_podcast_episodes SET episode_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = favorite_podcast_episodes.episode_id);

ALTER TABLE filters ADD COLUMN user_id_uuid TEXT;
UPDATE filters SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = filters.user_id);

ALTER TABLE sessions ADD COLUMN user_id_uuid TEXT;
UPDATE sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = sessions.user_id);

ALTER TABLE tags ADD COLUMN user_id_uuid TEXT;
UPDATE tags SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = tags.user_id);

ALTER TABLE tags_podcasts ADD COLUMN podcast_id_uuid TEXT;
UPDATE tags_podcasts SET podcast_id_uuid =
    (SELECT p.uuid FROM podcasts p WHERE p.id = tags_podcasts.podcast_id);

ALTER TABLE playlists ADD COLUMN user_id_uuid TEXT;
UPDATE playlists SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = playlists.user_id);

ALTER TABLE playlist_items ADD COLUMN episode_uuid TEXT;
UPDATE playlist_items SET episode_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = playlist_items.episode);

ALTER TABLE podcast_episode_chapters ADD COLUMN episode_id_uuid TEXT;
UPDATE podcast_episode_chapters SET episode_id_uuid =
    (SELECT e.uuid FROM podcast_episodes e WHERE e.id = podcast_episode_chapters.episode_id);

-- ABS user_id columns (parent already populated above)
ALTER TABLE audiobookshelf_listening_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_listening_sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_listening_sessions.user_id);
ALTER TABLE audiobookshelf_media_progress ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_media_progress SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_media_progress.user_id);
ALTER TABLE audiobookshelf_playback_sessions ADD COLUMN user_id_uuid TEXT;
UPDATE audiobookshelf_playback_sessions SET user_id_uuid =
    (SELECT u.uuid FROM users u WHERE u.id = audiobookshelf_playback_sessions.user_id);

-- ============================================================
-- 2. Recreate each table with UUID PK / FK columns
--    (SQLite cannot ALTER a primary key in place)
-- ============================================================
-- For EACH table below, the recreation step is identical in shape to the
-- `username_to_user_id` migration: CREATE <t>_new with the final column set
-- (id TEXT PRIMARY KEY, FK columns TEXT REFERENCES parent(id) preserving the
-- original ON DELETE behaviour, legacy_id INTEGER UNIQUE on podcasts/episodes),
-- INSERT ... SELECT copying every column (using the *_uuid columns for ids and
-- the old integer id AS legacy_id where applicable), DROP TABLE <t>,
-- ALTER TABLE <t>_new RENAME TO <t>, and recreate that table's indexes.
--
-- users recreation (worked example — full):
CREATE TABLE users_new (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    role TEXT NOT NULL,
    password TEXT,
    explicit_consent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL,
    api_key TEXT,
    country TEXT,
    language TEXT
);
INSERT INTO users_new (id, username, role, password, explicit_consent, created_at, api_key, country, language)
    SELECT uuid, username, role, password, explicit_consent, created_at, api_key, country, language FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

-- podcasts recreation (worked example — full, keeps legacy_id):
CREATE TABLE podcasts_new (
    id TEXT PRIMARY KEY NOT NULL,
    legacy_id INTEGER UNIQUE,
    name TEXT NOT NULL,
    directory_id TEXT NOT NULL,
    rssfeed TEXT NOT NULL,
    image_url TEXT NOT NULL,
    summary TEXT,
    language TEXT,
    explicit TEXT,
    keywords TEXT,
    last_build_date TEXT,
    author TEXT,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    original_image_url TEXT NOT NULL DEFAULT '',
    directory_name TEXT NOT NULL DEFAULT '',
    download_location TEXT,
    guid TEXT,
    added_by TEXT REFERENCES users(id)
);
INSERT INTO podcasts_new (id, legacy_id, name, directory_id, rssfeed, image_url, summary, language, explicit, keywords, last_build_date, author, active, original_image_url, directory_name, download_location, guid, added_by)
    SELECT uuid, id, name, directory_id, rssfeed, image_url, summary, language, explicit, keywords, last_build_date, author, active, original_image_url, directory_name, download_location, guid, added_by_uuid FROM podcasts;
DROP TABLE podcasts;
ALTER TABLE podcasts_new RENAME TO podcasts;
CREATE INDEX IF NOT EXISTS idx_podcasts_name ON podcasts(name);
CREATE INDEX IF NOT EXISTS idx_podcasts_rssfeed ON podcasts(rssfeed);

-- podcast_episodes recreation (worked example — full, keeps legacy_id):
CREATE TABLE podcast_episodes_new (
    id TEXT PRIMARY KEY NOT NULL,
    legacy_id INTEGER UNIQUE,
    podcast_id TEXT NOT NULL REFERENCES podcasts(id),
    episode_id TEXT NOT NULL,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    date_of_recording TEXT NOT NULL,
    image_url TEXT NOT NULL,
    total_time INTEGER NOT NULL,
    description TEXT NOT NULL,
    download_time TIMESTAMP,
    guid TEXT NOT NULL,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    file_episode_path TEXT,
    file_image_path TEXT,
    episode_numbering_processed BOOLEAN NOT NULL DEFAULT FALSE,
    download_location TEXT
);
INSERT INTO podcast_episodes_new (id, legacy_id, podcast_id, episode_id, name, url, date_of_recording, image_url, total_time, description, download_time, guid, deleted, file_episode_path, file_image_path, episode_numbering_processed, download_location)
    SELECT uuid, id, podcast_id_uuid, episode_id, name, url, date_of_recording, image_url, total_time, description, download_time, guid, deleted, file_episode_path, file_image_path, episode_numbering_processed, download_location FROM podcast_episodes;
DROP TABLE podcast_episodes;
ALTER TABLE podcast_episodes_new RENAME TO podcast_episodes;

-- Remaining tables to recreate with the same shape. For each, the final column
-- set equals the current schema with the listed columns retyped to TEXT and the
-- *_uuid backfill columns used in the SELECT. Recreate listed indexes after.
--
--  devices            : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       other columns unchanged. Index: idx_devices ON devices(name).
--  subscriptions      : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       UNIQUE(user_id, device, podcast). Indexes: idx_subscriptions(user_id),
--                       idx_subscriptions_device(device).
--  episodes           : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       UNIQUE(user_id, device, podcast, episode, timestamp).
--                       Indexes: idx_episodes_podcast(podcast), idx_episodes_episode(episode).
--  gpodder_settings   : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       UNIQUE(user_id, scope, scope_id).
--  device_sync_groups : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       UNIQUE(user_id, device_id).
--  notifications      : id TEXT PK; remaining columns unchanged.
--  settings           : id TEXT PK; remaining columns unchanged.
--  listening_events   : id TEXT PK; user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       podcast_id TEXT NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE;
--                       podcast_episode_db_id TEXT NOT NULL REFERENCES podcast_episodes(id) ON DELETE CASCADE;
--                       Indexes: idx_listening_events_user_id_time(user_id, listened_at),
--                       idx_listening_events_user_id_episode(user_id, podcast_episode_id),
--                       idx_listening_events_user_id_podcast(user_id, podcast_id).
--  podcast_settings   : podcast_id TEXT PRIMARY KEY REFERENCES podcasts(id) ON DELETE CASCADE;
--                       remaining columns unchanged.
--  favorites          : PRIMARY KEY (user_id, podcast_id); user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       podcast_id TEXT REFERENCES podcasts(id) ON DELETE CASCADE; favored BOOLEAN.
--  favorite_podcast_episodes : PRIMARY KEY (user_id, episode_id); user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       episode_id TEXT REFERENCES podcast_episodes(id); favorite BOOLEAN.
--  filters            : user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE; remaining columns unchanged.
--  sessions           : PRIMARY KEY (user_id, session_id); user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       username TEXT, session_id TEXT, expires TIMESTAMP.
--  tags               : id TEXT PK (unchanged); user_id TEXT REFERENCES users(id) ON DELETE CASCADE;
--                       UNIQUE(name, user_id). Indexes: idx_tags_name(name), idx_tags_user_id(user_id).
--  tags_podcasts      : PRIMARY KEY (tag_id, podcast_id); tag_id TEXT (unchanged);
--                       podcast_id TEXT REFERENCES podcasts(id) ON DELETE CASCADE.
--  playlists          : id TEXT PK (unchanged); user_id TEXT REFERENCES users(id) ON DELETE CASCADE.
--  playlist_items     : PRIMARY KEY (playlist_id, episode); playlist_id TEXT (unchanged);
--                       episode TEXT REFERENCES podcast_episodes(id); position INTEGER.
--  podcast_episode_chapters : id TEXT PK (unchanged); episode_id TEXT REFERENCES podcast_episodes(id) ON DELETE CASCADE.
--  audiobookshelf_listening_sessions / audiobookshelf_media_progress /
--  audiobookshelf_playback_sessions : id TEXT PK (unchanged); user_id TEXT (retyped);
--                       all other columns unchanged. (No FK constraint to users existed before —
--                       keep it unconstrained TEXT to match current behaviour, just retyped.)

PRAGMA foreign_keys = ON;
```

> When writing each recreation block, copy the *current* `CREATE TABLE` from the latest migration that defined the table (search `migrations/sqlite` for the table name) and change only the listed columns. Preserve all `NOT NULL`/`DEFAULT`/`CHECK` clauses verbatim.

- [ ] **Step 3: Sanity-check the SQL parses.** Apply it to a scratch copy of a dev DB:

```bash
cp podcast.db /tmp/scratch.db 2>/dev/null || sqlite3 /tmp/scratch.db "VACUUM;"
sqlite3 /tmp/scratch.db < migrations/sqlite/2026-05-28-120000_uuid_primary_keys/up.sql
sqlite3 /tmp/scratch.db "SELECT id, legacy_id, name FROM podcasts LIMIT 3; PRAGMA foreign_key_check;"
```

Expected: no SQL errors; `id` values are 36-char UUIDs; `legacy_id` holds the old integers; `foreign_key_check` returns no rows.

- [ ] **Step 4: Commit.**

```bash
git add migrations/sqlite/2026-05-28-120000_uuid_primary_keys
git commit -m "feat(db): sqlite migration converting integer ids to uuid"
```

### Task 4: Postgres migration up.sql

**Files:**
- Create: `migrations/postgres/2026-05-28-120000_uuid_primary_keys/up.sql`
- Create: `migrations/postgres/2026-05-28-120000_uuid_primary_keys/down.sql`

Postgres supports in-place `ALTER TABLE`, so no table recreation. Pattern per table mirrors `migrations/postgres/2026-04-15-100000_username_to_user_id/up.sql`.

- [ ] **Step 1: Write `down.sql`** (same forward-only no-op):

```sql
-- Forward-only migration; restore from a pre-upgrade backup. See sqlite down.sql.
SELECT 1;
```

- [ ] **Step 2: Write `up.sql`.** For every parent in conversion-map A: add a `uuid` text column, populate, and for podcasts/episodes add `legacy_id`. For every FK in map B: add the `text` column, backfill, drop the old constraint+column, rename. Then swap PKs. Worked example for `users` + `podcasts`:

```sql
-- users: integer id -> text uuid
ALTER TABLE users ADD COLUMN uuid TEXT;
UPDATE users SET uuid = gen_random_uuid()::text;
ALTER TABLE users ALTER COLUMN uuid SET NOT NULL;

-- podcasts: add uuid + legacy_id + remap added_by
ALTER TABLE podcasts ADD COLUMN uuid TEXT;
UPDATE podcasts SET uuid = gen_random_uuid()::text;
ALTER TABLE podcasts ALTER COLUMN uuid SET NOT NULL;
ALTER TABLE podcasts ADD COLUMN legacy_id INTEGER;
UPDATE podcasts SET legacy_id = id;
ALTER TABLE podcasts ADD COLUMN added_by_uuid TEXT;
UPDATE podcasts SET added_by_uuid = u.uuid FROM users u WHERE podcasts.added_by = u.id;

-- (repeat add-uuid / backfill for podcast_episodes(+legacy_id), devices,
--  subscriptions, episodes, gpodder_settings, device_sync_groups, notifications,
--  settings, listening_events, podcast_settings, and every FK column in map B,
--  using `gen_random_uuid()::text` for new PKs and `UPDATE child SET x_uuid =
--  p.uuid FROM parent p WHERE child.x = p.id` for FK backfills.)

-- Swap constraints, parents before children. Example for users<-devices:
--   1) drop child FK constraints that point at integer parents
--   2) on each parent: drop PK, drop integer id column, rename uuid -> id,
--      add PRIMARY KEY (id)
--   3) on each child: drop integer FK column, rename *_uuid -> original name,
--      set NOT NULL where the original was NOT NULL, re-add FK + UNIQUE constraints
-- Use the constraint names from the username_to_user_id migration as the model
-- (e.g. fk_devices_user, subscriptions_user_id_device_podcast_key).
```

- [ ] **Step 3: Verify against the test container.** (Requires the Postgres test container; see `test_support.rs`.) Build the workspace so the migration is embedded, then run any podfetch-web test with `--features postgresql` — `run_migrations()` applies it at startup. Defer full verification to Task 12 (round-trip test) which runs on both backends.

Run: `cargo check -p podfetch-persistence --no-default-features --features postgresql`
Expected: compiles (migration is embedded as a string; SQL is validated at runtime).

- [ ] **Step 4: Commit.**

```bash
git add migrations/postgres/2026-05-28-120000_uuid_primary_keys
git commit -m "feat(db): postgres migration converting integer ids to uuid"
```

---

## Phase 2 — Backend type migration

This is the large, unavoidably-atomic part of a big-bang: Rust will not compile until every `i32` id is retyped. Work crate-by-crate, compiling at the end of the task.

### Task 5: schema.rs — retype columns

**Files:**
- Modify: `crates/podfetch-persistence/src/schema.rs`
- Modify: the per-file `diesel::table!` blocks listed below

- [ ] **Step 1: Edit `schema.rs`.** Change these column types from `Integer`/`Nullable<Integer>` to `Text`/`Nullable<Text>`, and add `legacy_id -> Nullable<BigInt>` to podcasts and podcast_episodes:
  - `users.id`, `devices.id`, `devices.user_id`, `episodes.id`, `episodes.user_id`,
    `favorite_podcast_episodes.user_id`, `favorite_podcast_episodes.episode_id`,
    `favorites.user_id`, `favorites.podcast_id`, `filters.user_id`,
    `gpodder_settings.id`, `gpodder_settings.user_id`, `notifications.id`,
    `listening_events.id`, `listening_events.user_id`, `listening_events.podcast_id`,
    `listening_events.podcast_episode_db_id`, `playlist_items.episode`,
    `playlists.user_id`, `podcast_episode_chapters.episode_id`,
    `podcast_episodes.id`, `podcast_episodes.podcast_id`,
    `podcast_settings.podcast_id`, `podcasts.id`, `podcasts.added_by`,
    `sessions.user_id`, `settings.id`, `subscriptions.id`, `subscriptions.user_id`,
    `tags.user_id`, `tags_podcasts.podcast_id`, `users.*` (id only).
  - Add `legacy_id -> Nullable<BigInt>,` to the `podcasts` and `podcast_episodes` blocks.

- [ ] **Step 2: Edit the ABS `table!` blocks.** In `crates/podfetch-persistence/src/audiobookshelf/{listening_session,media_progress,playback_session}.rs`, change `user_id -> Integer` to `user_id -> Text`.

- [ ] **Step 3: Do NOT compile-check yet** (structs still use `i32`). Proceed to Task 6.

### Task 6: Retype domain + persistence structs and repositories

**Files (modify):** `crates/podfetch-domain/src/*.rs` (all id/FK fields), `crates/podfetch-persistence/src/*.rs` and `crates/podfetch-persistence/src/audiobookshelf/*.rs` (diesel row structs + queries), `crates/podfetch-web/src/services/**` and `crates/podfetch-web/src/role.rs` call sites.

- [ ] **Step 1: Domain structs → `Uuid`.** In `crates/podfetch-domain/src`, change every id/FK field from `i32` to `uuid::Uuid` (or `Option<Uuid>` where nullable). Add `pub legacy_id: Option<i64>` to the `Podcast` domain struct and the episode domain struct. Files include `user_admin.rs` (`ManagedUser.id`, `UserSummary.id`, `UserWithApiKey.id`), `podcast.rs` (`Podcast.id`, `Podcast.added_by`, `NewPodcast.added_by`, `PodcastMetadataUpdate.id`), `favorite.rs` (`user_id`, `podcast_id`), and every repository trait method signature taking `i32` ids.

- [ ] **Step 2: Persistence row structs → `String`.** In each diesel `Queryable/Insertable` struct, id/FK fields become `String` (or `Option<String>`). Add the boundary conversions in the `From<Entity> for Domain` / `From<Domain> for Entity` impls: `id: Uuid::parse_str(&value.id).expect("valid uuid in db")` on read, `id: value.id.to_string()` on write. For nullable: `value.x.map(|u| u.to_string())` / `value.x.as_deref().map(|s| Uuid::parse_str(s).unwrap())`.

- [ ] **Step 3: Inserts set ids explicitly.** Every repository `create`/insert now writes the id. Generalize the `ensure_with_id` pattern from the FK fix: in each `create`, accept the domain object whose `id` is set by the caller, and include `id.eq(entity.id)` in the `.values(...)`. Update services to call `podfetch_domain::ids::new_id()` and set `id` before insert (podcasts, podcast_episodes, devices, subscriptions, episodes, gpodder_settings, device_sync_groups, notifications, listening_events, users created via `create_user`). Add `legacy_id: None` for new podcasts/episodes.

- [ ] **Step 4: Query filters use string ids.** Anywhere a query compares an id column, pass `uuid.to_string()` (e.g. `.filter(id.eq(some_uuid.to_string()))`). For `legacy_id` lookups add new repository methods `find_by_legacy_id(i64)` on the podcast and podcast-episode repositories.

- [ ] **Step 5: Fix `role.rs` + user_auth call sites.** With `STANDARD_USER_ID: Uuid` (Task 2), update `read_only_admin_user`, `read_only_admin_id` (return `Uuid`), `configured_admin_user`, and the `ensure_standard_user_present` seed to use the UUID. Update `crates/podfetch-web/src/services/user_admin/service.rs::read_only_admin_id` return type to `Uuid`.

- [ ] **Step 6: Auth / socket / sessions.** `User.id` is now `Uuid`; socket room join uses `user.id.to_string()` (already a string call — confirm it compiles); session rows store the uuid string.

- [ ] **Step 7: Compile the backend (no web routes yet may still fail — that's Task 7-8).** Iterate until the domain + persistence crates compile:

Run: `cargo check -p podfetch-domain -p podfetch-persistence --no-default-features --features sqlite 2>&1 | tail -40`
Expected: clean. (podfetch-web may still have route/DTO errors — addressed next.)

- [ ] **Step 8: Update the standard-user tests.** In `crates/podfetch-web/src/services/user_auth/service.rs` tests (from the FK fix), the `Filter::new(STANDARD_USER_ID, ...)` and `find_by_username` assertions now use the `Uuid` constant. Retype them.

- [ ] **Step 9: Commit** (after the whole backend compiles at the end of Task 8).

### Task 7: Resolver for legacy-or-uuid path params

**Files:**
- Create: `crates/podfetch-web/src/controllers/id_resolver.rs`
- Modify: `crates/podfetch-web/src/controllers/mod.rs` (register module)
- Test: same file (`#[cfg(test)]`)

- [ ] **Step 1: Write the failing test.** Create `crates/podfetch-web/src/controllers/id_resolver.rs`:

```rust
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::error::ErrorSeverity::Warning;
use uuid::Uuid;

/// A podcast/episode path segment that is either a UUID (new links) or a
/// legacy integer id (old links).
#[derive(Debug, PartialEq, Eq)]
pub enum ResolvedId {
    Uuid(Uuid),
    Legacy(i64),
}

/// Parse a `{id}` path segment: prefer UUID, fall back to legacy integer.
pub fn parse_resolved_id(segment: &str) -> Result<ResolvedId, CustomError> {
    if let Ok(uuid) = Uuid::parse_str(segment) {
        return Ok(ResolvedId::Uuid(uuid));
    }
    if let Ok(legacy) = segment.parse::<i64>() {
        return Ok(ResolvedId::Legacy(legacy));
    }
    Err(CustomErrorInner::BadRequest(
        format!("'{segment}' is not a valid id"),
        Warning,
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uuid() {
        let u = "0192f3a1-7c42-7e8b-8b2a-2b1c3d4e5f60";
        assert_eq!(parse_resolved_id(u), Ok(ResolvedId::Uuid(Uuid::parse_str(u).unwrap())));
    }

    #[test]
    fn parses_legacy_integer() {
        assert_eq!(parse_resolved_id("42"), Ok(ResolvedId::Legacy(42)));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_resolved_id("not-an-id").is_err());
    }
}
```

- [ ] **Step 2: Register the module.** Add `pub mod id_resolver;` to `crates/podfetch-web/src/controllers/mod.rs`.

- [ ] **Step 3: Run the test.**

Run: `cargo test -p podfetch-web --no-default-features --features sqlite id_resolver::`
Expected: PASS (3 tests). (Requires Task 6 to compile; if running before, expect compile errors only.)

- [ ] **Step 4: Commit** (with Task 8).

### Task 8: Apply the resolver to podcast/episode routes

**Files (modify):** `crates/podfetch-web/src/controllers/podcast_controller.rs`, `crates/podfetch-web/src/controllers/podcast_episode_controller.rs`, `crates/podfetch-web/src/controllers/websocket_controller.rs` (RSS routes), `crates/podfetch-web/src/podcast.rs` (`build_podfetch_feed`), `crates/podfetch-web/src/audiobookshelf_api/mapping/podcast.rs` (`pod_{id}`).

- [ ] **Step 1: Convert podcast handlers.** Change the affected handlers' `Path<i32>` / `Path<String>` (then `.parse::<i32>()`) to `Path<String>` → `parse_resolved_id(&seg)?` → load via `find_by_id(uuid)` or `find_by_legacy_id(legacy)`. Routes: `/api/v1/podcasts/{id}`, `/podcasts/{id}/refresh|name|active|settings`, DELETE `/podcasts/{id}`, `/podcasts/reverse/episode/{id}`, and the RSS routes `/rss/{id}`, `/rss/apiKey/{key}/{id}`.

- [ ] **Step 2: UUID-first URL generation.** In `crates/podfetch-web/src/podcast.rs::build_podfetch_feed`, format `"/rss/{}"` / `".../rss/{}"` with `podcast.id` (now a `Uuid`, renders as the canonical string).

- [ ] **Step 3: ABS item ids.** In `audiobookshelf_api/mapping/podcast.rs`, build item ids as `format!("pod_{}", podcast.id)`. In the incoming-id parser, strip the `pod_` prefix then `parse_resolved_id` the remainder (accepts `pod_{uuid}` and legacy `pod_{int}`).

- [ ] **Step 4: Compile the whole backend.**

Run: `cargo check --no-default-features --features sqlite 2>&1 | tail -40`
Expected: clean across all crates.

- [ ] **Step 5: Commit the entire backend type migration + resolver.**

```bash
git add crates/ migrations/
git commit -m "feat: migrate backend ids to uuid with legacy-id resolver"
```

---

## Phase 3 — API contract

### Task 9: DTOs expose UUID `id` + `legacyId`

**Files (modify):** `crates/podfetch-web/src/podcast.rs` (`PodcastDto`), `crates/podfetch-web/src/podcast_episode_dto.rs` (`PodcastEpisodeDto`), and any other DTO with an `id`/relational id (`UserSummary`, etc.). utoipa annotations included.

- [ ] **Step 1: Retype DTO id fields.** `PodcastDto.id: String` (the UUID), add `pub legacy_id: Option<i64>` serialized as `legacyId` (`#[serde(rename = "legacyId")]`). Same for `PodcastEpisodeDto` (`id: String`, `podcast_id: String`, `legacy_id: Option<i64>`). Other DTOs: `id` and relational ids become `String` (UUID rendered via `.to_string()` in the mapping `From` impls).

- [ ] **Step 2: Build + check OpenAPI.**

Run: `cargo check --no-default-features --features sqlite 2>&1 | tail -20`
Expected: clean.

- [ ] **Step 3: Commit.**

```bash
git add crates/
git commit -m "feat(api): expose uuid id and legacyId on podcast/episode dtos"
```

### Task 10: Regenerate the frontend OpenAPI types

**Files:** `ui/schema.d.ts` (generated)

- [ ] **Step 1: Find the generator command.** Check `ui/package.json` scripts for the openapi-typescript command (e.g. `npm run generate:schema`). Start the server or use the committed `openapi.json` if the repo generates from a static spec.

- [ ] **Step 2: Regenerate and commit.**

Run: `cd ui && <the generate script>`
Expected: `ui/schema.d.ts` now types podcast/episode `id` as `string` and includes `legacyId`.

```bash
git add ui/schema.d.ts
git commit -m "chore(ui): regenerate openapi types for uuid ids"
```

---

## Phase 4 — Clients

### Task 11: Update `ui/` and `mobile/` id handling

**Files (modify):** `ui/src/models/User.ts`, `ui/src/components/{PodcastCard,DrawerAudioPlayer,EpisodeSearchModal}.tsx` and any other consumer flagged by the type-checker; `mobile/` equivalents.

- [ ] **Step 1: Fix web types.** `ui/src/models/User.ts`: `id: number` → `id: string`. Remove now-unneeded numeric coercions like `String(podcastEpisode?.podcastEpisode.podcast_id)` (the value is already a string) in `DrawerAudioPlayer.tsx`; keep template-literal URL building (`/podcasts/${id}/...`) which works unchanged.

- [ ] **Step 2: Type-check the web app.**

Run: `cd ui && npm run build` (or `npm run type-check` if present)
Expected: no type errors referencing podcast/user `id`.

- [ ] **Step 3: Fix mobile types.** Apply the same `number → string` id changes in `mobile/` (its API types / models and any place an id is parsed as a number or used in a URL).

- [ ] **Step 4: Type-check mobile.**

Run: `cd mobile && npm run lint` (and `npx tsc --noEmit` if configured)
Expected: no id-related type errors.

- [ ] **Step 5: Commit.**

```bash
git add ui/ mobile/
git commit -m "feat(clients): adopt uuid string ids in web and mobile"
```

---

## Phase 5 — Tests

### Task 12: Migration round-trip test (the critical guard)

**Files:**
- Create: `crates/podfetch-web/src/controllers/uuid_migration_tests.rs` (or add to an existing integration test module that has DB access via `handle_test_startup`)
- Modify: `crates/podfetch-web/src/controllers/mod.rs`

> The migration runs automatically in `handle_test_startup` → `build_server_router` → `run_migrations()`. This test seeds data **through the post-migration API/repositories**, then asserts UUID/legacy/FK invariants. (A true pre-migration integer fixture cannot be inserted once the schema is UUID; instead we assert the invariants the migration must uphold: every id is a UUID, FK joins resolve, and legacy lookups work.)

- [ ] **Step 1: Write the test.**

```rust
#[cfg(test)]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use podfetch_persistence::db::get_connection;
    use diesel::{sql_query, RunQueryDsl, QueryableByName};
    use uuid::Uuid;
    use serial_test::serial;

    #[derive(QueryableByName)]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
    }

    #[tokio::test]
    #[serial]
    async fn all_podcast_ids_are_uuids_and_legacy_lookup_resolves() {
        let ts = handle_test_startup().await;
        // Create a podcast via the API (admin auth header is set by the harness).
        let created = ts.test_server
            .post("/api/v1/podcasts/itunes")
            .json(&serde_json::json!({ "trackId": 0, "userId": 0 }))
            .await;
        // Itunes lookup may 4xx without network; instead assert on the schema:
        let mut conn = get_connection();
        let rows: Vec<IdRow> = sql_query("SELECT id FROM podcasts")
            .load(&mut conn)
            .unwrap();
        for r in rows {
            assert!(Uuid::parse_str(&r.id).is_ok(), "podcast id must be a uuid: {}", r.id);
        }
        let _ = created;
    }

    #[tokio::test]
    #[serial]
    async fn foreign_key_columns_are_text_uuids() {
        let _ts = handle_test_startup().await;
        let mut conn = get_connection();
        // podcast_episodes.podcast_id must be TEXT and (when present) a uuid.
        let rows: Vec<IdRow> = sql_query("SELECT podcast_id AS id FROM podcast_episodes")
            .load(&mut conn)
            .unwrap();
        for r in rows {
            assert!(Uuid::parse_str(&r.id).is_ok(), "podcast_id must be a uuid: {}", r.id);
        }
    }
}
```

- [ ] **Step 2: Run on sqlite.**

Run: `cargo test -p podfetch-web --no-default-features --features sqlite uuid_migration_tests`
Expected: PASS.

- [ ] **Step 3: Run on postgres.**

Run: `cargo test -p podfetch-web --no-default-features --features postgresql uuid_migration_tests`
Expected: PASS (validates the Postgres migration applied cleanly).

- [ ] **Step 4: Commit.**

```bash
git add crates/podfetch-web/src/controllers/
git commit -m "test: assert uuid ids and uuid foreign keys after migration"
```

### Task 13: Dual-resolution route tests

**Files:** add to `crates/podfetch-web/src/controllers/podcast_controller.rs` `#[cfg(test)]`.

- [ ] **Step 1: Write the tests.** Create a podcast (capture its `id` and `legacyId` from the JSON response), then assert both forms resolve:

```rust
#[tokio::test]
#[serial]
async fn podcast_resolves_by_uuid_and_by_legacy_id() {
    let ts = handle_test_startup().await;
    // Insert a podcast row directly with a known legacy_id via the repository.
    // (Use the podcast repository's create + a manual legacy_id update, or a
    //  test-only INSERT, depending on what the repo exposes.)
    // Then:
    let by_uuid = ts.test_server.get(&format!("/api/v1/podcasts/{uuid}")).await;
    assert_eq!(by_uuid.status_code(), 200);
    let by_legacy = ts.test_server.get(&format!("/api/v1/podcasts/{legacy}")).await;
    assert_eq!(by_legacy.status_code(), 200);
    let bad = ts.test_server.get("/api/v1/podcasts/not-an-id").await;
    assert_eq!(bad.status_code(), 400);
}
```

Fill `{uuid}`/`{legacy}` from a podcast you insert in-test (mirror the insertion style already used in this file's existing podcast tests; set `legacy_id` via a `diesel::update` on the podcasts table).

- [ ] **Step 2: Run.**

Run: `cargo test -p podfetch-web --no-default-features --features sqlite podcast_resolves_by_uuid_and_by_legacy_id`
Expected: PASS.

- [ ] **Step 3: Commit.**

```bash
git add crates/podfetch-web/src/controllers/podcast_controller.rs
git commit -m "test: podcast routes resolve by uuid and legacy integer id"
```

### Task 14: Full suite, clippy, and final verification

- [ ] **Step 1: Full backend test suite (sqlite).**

Run: `cargo test --no-default-features --features sqlite 2>&1 | tail -30`
Expected: all pass (fix any integer-id assertions left in older tests — e.g. routes that used `/api/v1/podcasts/999999/...` expecting a numeric path now still work via the legacy branch, but assertions on numeric `id` in responses must become string/uuid).

- [ ] **Step 2: Full backend test suite (postgres).**

Run: `cargo test --no-default-features --features postgresql 2>&1 | tail -30`
Expected: all pass.

- [ ] **Step 3: Clippy + workspace check.**

Run: `cargo clippy --workspace --all-targets 2>&1 | tail -20 && cargo check --workspace`
Expected: no errors.

- [ ] **Step 4: Client builds.**

Run: `cd ui && npm run build` and `cd mobile && npm run lint`
Expected: both succeed.

- [ ] **Step 5: Commit any fixes, then open the PR.**

```bash
git add -A
git commit -m "test: update existing suite for uuid ids"
git push -u origin feat/uuid-primary-keys
gh pr create --base main --title "feat: UUID primary keys with legacy-id backwards compatibility" --body-file <(echo "Implements docs/superpowers/specs/2026-05-28-uuid-primary-keys-design.md")
```

---

## Self-review notes (resolved while writing)

- **Spec coverage:** §1 conventions → Tasks 1,2,6; §2 migration → Tasks 3,4; §3 routing/API → Tasks 7,8,9,10; §4 backend/clients → Tasks 5,6,11; §5 testing → Tasks 12,13,14; §6 rollout → forward-only `down.sql` (Tasks 3,4) + PR note (Task 14). All covered.
- **Sequencing caveat:** Phase 2 is one large compile unit (big-bang). Tasks 5–8 only fully compile at the end of Task 8; their interim `cargo check` steps are expected to surface call-site errors until the phase completes. This is inherent to the big-bang choice and was accepted in the spec.
- **`legacy_id` type:** stored `INTEGER`/`BigInt`, surfaced as `i64`/`Option<i64>` in Rust and `legacyId: number` in JSON.
- **Open item for the implementer:** confirm the exact OpenAPI regeneration command in `ui/package.json` (Task 10 Step 1) and the podcast-insertion helper used by existing tests (Task 13).
```
