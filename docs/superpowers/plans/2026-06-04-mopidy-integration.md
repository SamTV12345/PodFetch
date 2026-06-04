# Mopidy Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a PodFetch user stream an episode to a Mopidy server (which plays it through its outputs) and drive play/pause/seek/volume, reusing the existing cast framework, gated behind a `MOPIDY_INTEGRATION_ENABLED` flag.

**Architecture:** Mopidy is a new device `kind` (`mopidy_personal` / `mopidy_shared`) stored in the `devices` table with a `base_url`. A `MopidyDriver` (a sibling of `AgentDispatcher`, **not** a `CastDriver` impl) talks Mopidy JSON-RPC over HTTP and runs a per-session status pump that emits events over an mpsc channel; a startup-wired consumer feeds those into the existing `CastOrchestrator` (status broadcast + watchtime). The orchestrator routes start/control to the Mopidy driver by device kind. Servers are managed from a settings page via `/api/v1/mopidy/servers`.

**Tech Stack:** Rust (axum, diesel, reqwest, tokio, utoipa), TypeScript/React (vite, openapi-react-query), SQLite + PostgreSQL migrations.

---

## File Structure

**New files**

- `migrations/sqlite/2026-06-04-130000_devices_base_url/up.sql` + `down.sql` — add `base_url` column.
- `migrations/postgres/2026-06-04-130000_devices_base_url/up.sql` + `down.sql` — add `base_url` column.
- `crates/podfetch-web/src/services/mopidy/mod.rs` — module root.
- `crates/podfetch-web/src/services/mopidy/rpc.rs` — JSON-RPC client + pure mapping helpers (most unit tests live here).
- `crates/podfetch-web/src/services/mopidy/driver.rs` — `MopidyDriver`, `MopidyTarget`, `MopidyEvent`, status pump.
- `crates/podfetch-web/src/services/mopidy/consumer.rs` — event consumer wired at startup.
- `crates/podfetch-web/src/controllers/mopidy_controller.rs` — `/mopidy/servers` endpoints + DTOs.
- `ui/src/pages/MopidyIntegration.tsx` — settings page.

**Modified files**

- `crates/common-infrastructure/src/config.rs` — env flag + `ConfigModel` field.
- `crates/podfetch-domain/src/device.rs` — kind constants/helpers + `base_url` on `Device` + repo method signatures.
- `crates/podfetch-persistence/src/device.rs` — `table!` + `DeviceEntity` + repo impls.
- `crates/podfetch-persistence/src/schema.rs` — `devices` table column.
- `crates/podfetch-web/src/services/cast/service.rs` — orchestrator routing + `ActiveSession.device_kind`.
- `crates/podfetch-web/src/cast.rs` — orchestrator constructor call site (type alias unchanged).
- `crates/podfetch-web/src/app_state.rs` — build channel + driver, store receiver.
- `crates/podfetch-web/src/services/mod.rs` — `pub mod mopidy;`
- `crates/podfetch-web/src/controllers/mod.rs` — `pub mod mopidy_controller;`
- `crates/podfetch-web/src/services/device/service.rs` — `delete_by_id` / `find_by_id` passthroughs.
- `crates/podfetch-web/src/startup.rs` — mount router + spawn consumer (both flag-gated).
- `ui/schema.d.ts` — regenerated.
- `ui/src/pages/SettingsPage.tsx` — conditional nav link.
- `ui/src/App.tsx` — route.
- `ui/src/components/CastButton.tsx` — Mopidy kind label.
- `ui/src/language/json/{en,de,da,es,fr,pl,zh}.json` — i18n keys.

---

## Task 1: Config flag + ConfigModel field

**Files:**
- Modify: `crates/common-infrastructure/src/config.rs`

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/common-infrastructure/src/config.rs` (create a `#[cfg(test)] mod tests` block if none exists):

```rust
#[cfg(test)]
mod mopidy_config_tests {
    use super::*;

    #[test]
    fn config_model_exposes_mopidy_flag() {
        let mut env = EnvironmentService::new();
        env.mopidy_integration_enabled = true;
        let model = env.get_config("http://localhost:8000/");
        assert!(model.mopidy_integration_enabled);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p common-infrastructure mopidy_config_tests`
Expected: FAIL — `no field mopidy_integration_enabled on EnvironmentService` / `ConfigModel`.

- [ ] **Step 3: Implement**

In `crates/common-infrastructure/src/config.rs`:

Add the constant near the other `*_INTEGRATION_ENABLED` consts (after line 32):

```rust
pub const MOPIDY_INTEGRATION_ENABLED: &str = "MOPIDY_INTEGRATION_ENABLED";
```

Add the field to `struct EnvironmentService` (next to `audiobookshelf_integration_enabled`):

```rust
    pub mopidy_integration_enabled: bool,
```

Add to `ConfigModel` (after `reverse_proxy: bool,`):

```rust
    pub mopidy_integration_enabled: bool,
```

In `EnvironmentService::new()`, next to the `audiobookshelf_integration_enabled:` initializer:

```rust
            mopidy_integration_enabled: is_env_var_present_and_true(MOPIDY_INTEGRATION_ENABLED),
```

In `EnvironmentService::for_tests()`, after `environment.audiobookshelf_integration_enabled = true;`:

```rust
        environment.mopidy_integration_enabled = true;
```

In `get_config(...)`, add to the returned `ConfigModel { ... }`:

```rust
            mopidy_integration_enabled: self.mopidy_integration_enabled,
```

In `get_environment(...)`, after the audiobookshelf info line:

```rust
        tracing::info!(
            "Mopidy integration enabled: {}",
            self.mopidy_integration_enabled
        );
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p common-infrastructure mopidy_config_tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/common-infrastructure/src/config.rs
git commit -m "feat(mopidy): add MOPIDY_INTEGRATION_ENABLED config flag (#505)"
```

---

## Task 2: Device domain — kinds, helpers, base_url

**Files:**
- Modify: `crates/podfetch-domain/src/device.rs`

- [ ] **Step 1: Write the failing test**

Append to `crates/podfetch-domain/src/device.rs`:

```rust
#[cfg(test)]
mod mopidy_kind_tests {
    use super::kind;

    #[test]
    fn mopidy_kinds_are_mopidy_and_castable_but_not_chromecast() {
        assert!(kind::is_mopidy(kind::MOPIDY_PERSONAL));
        assert!(kind::is_mopidy(kind::MOPIDY_SHARED));
        assert!(!kind::is_mopidy(kind::CHROMECAST_SHARED));

        assert!(kind::is_castable(kind::MOPIDY_SHARED));
        assert!(kind::is_castable(kind::CHROMECAST_PERSONAL));
        assert!(!kind::is_castable(kind::DESKTOP));

        assert!(!kind::is_chromecast(kind::MOPIDY_PERSONAL));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p podfetch-domain mopidy_kind_tests`
Expected: FAIL — `MOPIDY_PERSONAL` / `is_mopidy` not found.

- [ ] **Step 3: Implement**

In `crates/podfetch-domain/src/device.rs`, inside `pub mod kind`, after the chromecast constants/helpers (after line 26):

```rust
    /// Personal Mopidy server — visible only to the owning user.
    pub const MOPIDY_PERSONAL: &str = "mopidy_personal";
    /// Shared/household Mopidy server — visible to every user on the instance.
    pub const MOPIDY_SHARED: &str = "mopidy_shared";

    /// True for any mopidy_* kind.
    pub fn is_mopidy(kind: &str) -> bool {
        matches!(kind, MOPIDY_PERSONAL | MOPIDY_SHARED)
    }

    /// True for any kind that can be a remote-playback target (chromecast or mopidy).
    pub fn is_castable(kind: &str) -> bool {
        is_chromecast(kind) || is_mopidy(kind)
    }
```

Add `base_url` to the `Device` struct (after `pub ip: Option<String>,`):

```rust
    /// Mopidy RPC base URL (e.g. `http://mopidy.local:6680`). `None` for non-mopidy devices.
    pub base_url: Option<String>,
```

Add two methods to the `DeviceRepository` trait (after `find_by_chromecast_uuid`). They carry **default bodies** so existing test doubles (`FakeDeviceRepo`) keep compiling; the real persistence implementors override them in Task 3:

```rust
    /// Look up a single device row by its primary id. Real implementors override this.
    fn find_by_id(&self, _id: Uuid) -> Result<Option<Device>, Self::Error> {
        Ok(None)
    }

    /// Delete a single device row by primary id; returns rows removed.
    /// Real implementors override this.
    fn delete_by_id(&self, _id: Uuid) -> Result<usize, Self::Error> {
        Ok(0)
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p podfetch-domain mopidy_kind_tests`
Expected: PASS. (The crate won't fully build yet because implementors lack `find_by_id`/`delete_by_id` and the `base_url` field — Task 3 fixes the implementors. `cargo test -p podfetch-domain` only compiles this crate, which has no `DeviceRepository` implementor, so it passes.)

- [ ] **Step 5: Commit**

```bash
git add crates/podfetch-domain/src/device.rs
git commit -m "feat(mopidy): add mopidy device kinds and base_url to domain (#505)"
```

---

## Task 3: Persistence — base_url column, migrations, repo methods

**Files:**
- Create: `migrations/sqlite/2026-06-04-130000_devices_base_url/up.sql`, `down.sql`
- Create: `migrations/postgres/2026-06-04-130000_devices_base_url/up.sql`, `down.sql`
- Modify: `crates/podfetch-persistence/src/device.rs`
- Modify: `crates/podfetch-persistence/src/adapters.rs`
- Modify: `crates/podfetch-persistence/src/schema.rs`

> The `base_url` column is a nullable trailing `ADD COLUMN` — **not** a table rebuild — so the SQLite FK-pragma `run_in_transaction=false` gotcha does not apply and no `metadata.toml` is needed.

- [ ] **Step 1: Create the migrations**

`migrations/sqlite/2026-06-04-130000_devices_base_url/up.sql`:

```sql
ALTER TABLE devices ADD COLUMN base_url TEXT;
```

`migrations/sqlite/2026-06-04-130000_devices_base_url/down.sql`:

```sql
ALTER TABLE devices DROP COLUMN base_url;
```

`migrations/postgres/2026-06-04-130000_devices_base_url/up.sql`:

```sql
ALTER TABLE devices ADD COLUMN base_url TEXT;
```

`migrations/postgres/2026-06-04-130000_devices_base_url/down.sql`:

```sql
ALTER TABLE devices DROP COLUMN base_url;
```

- [ ] **Step 2: Write the failing test**

Append to `crates/podfetch-persistence/src/device.rs` (it already has `use` for the table; the test uses the public repo):

```rust
#[cfg(test)]
mod mopidy_persistence_tests {
    use super::*;
    use crate::db::database;
    use podfetch_domain::device::kind as device_kind;

    fn mopidy_device(owner: Uuid, kind_str: &str, url: &str) -> Device {
        Device {
            id: None,
            deviceid: url.to_string(),
            kind: kind_str.to_string(),
            name: "Living Room".to_string(),
            user_id: owner,
            chromecast_uuid: Some(podfetch_domain::ids::new_id().to_string()),
            agent_id: None,
            last_seen_at: None,
            ip: None,
            base_url: Some(url.to_string()),
        }
    }

    #[test]
    fn create_persists_base_url_and_list_castable_includes_shared_mopidy() {
        let repo = DieselDeviceRepository::new(database());
        let owner = podfetch_domain::ids::new_id();
        let viewer = podfetch_domain::ids::new_id();

        let created = repo
            .create(mopidy_device(owner, device_kind::MOPIDY_SHARED, "http://m.local:6680"))
            .expect("create mopidy device");
        assert_eq!(created.base_url.as_deref(), Some("http://m.local:6680"));

        // A different viewer sees a shared mopidy server.
        let castable = repo.list_castable_for_user(viewer).expect("list castable");
        assert!(castable.iter().any(|d| d.kind == device_kind::MOPIDY_SHARED));

        // Round-trip find + delete.
        let found = repo.find_by_id(created.id.unwrap()).expect("find").expect("present");
        assert_eq!(found.base_url, created.base_url);
        assert_eq!(repo.delete_by_id(created.id.unwrap()).expect("delete"), 1);
        assert!(repo.find_by_id(created.id.unwrap()).expect("find again").is_none());
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p podfetch-persistence mopidy_persistence_tests`
Expected: FAIL — missing `base_url` field / `find_by_id` / `delete_by_id`.

- [ ] **Step 4: Implement**

In `crates/podfetch-persistence/src/device.rs`, add `base_url` to the local `diesel::table!` (after `ip -> Nullable<Text>,`):

```rust
        base_url -> Nullable<Text>,
```

Add `base_url: Option<String>` to `struct DeviceEntity` (after `ip: Option<String>,`).

In `impl From<Device> for DeviceEntity`, add (after `ip: value.ip,`):

```rust
            base_url: value.base_url,
```

In `impl From<DeviceEntity> for Device`, add (after `ip: value.ip,`):

```rust
            base_url: value.base_url,
```

In `upsert_chromecast_from_agent`, the `None =>` branch builds a `DeviceEntity` literal — add `base_url: None,` to it (after `ip: ip_value.map(ToString::to_string),`).

Replace the body of `list_castable_for_user` so it also returns mopidy devices:

```rust
    fn list_castable_for_user(&self, viewer_user_id: Uuid) -> Result<Vec<Device>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;

        let viewer = viewer_user_id.to_string();
        // Any shared chromecast/mopidy on the instance, OR a personal one owned by the viewer.
        devices
            .filter(
                kind.eq(device_kind::CHROMECAST_SHARED)
                    .or(kind.eq(device_kind::MOPIDY_SHARED))
                    .or(kind
                        .eq(device_kind::CHROMECAST_PERSONAL)
                        .and(user_id.eq(&viewer)))
                    .or(kind
                        .eq(device_kind::MOPIDY_PERSONAL)
                        .and(user_id.eq(&viewer))),
            )
            .load::<DeviceEntity>(&mut conn)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
```

Add the two new repo methods inside `impl DeviceRepository for DieselDeviceRepository` (after `find_by_chromecast_uuid`):

```rust
    fn find_by_id(&self, id_to_find: Uuid) -> Result<Option<Device>, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;
        match devices
            .filter(id.eq(id_to_find.to_string()))
            .first::<DeviceEntity>(&mut conn)
        {
            Ok(entity) => Ok(Some(entity.into())),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_by_id(&self, id_to_delete: Uuid) -> Result<usize, Self::Error> {
        use self::devices::dsl::*;

        let mut conn = self.database.connection()?;
        diesel::delete(devices.filter(id.eq(id_to_delete.to_string())))
            .execute(&mut conn)
            .map_err(Into::into)
    }
```

In `crates/podfetch-persistence/src/schema.rs`, add `base_url` to the `devices` table block (after `ip -> Nullable<Text>,`):

```rust
        base_url -> Nullable<Text>,
```

In `crates/podfetch-persistence/src/adapters.rs`, the `DeviceRepositoryImpl` wrapper delegates to its inner `DieselDeviceRepository`. The default trait bodies would silently no-op here, so add explicit delegating overrides inside `impl DeviceRepository for DeviceRepositoryImpl` (after `find_by_chromecast_uuid`):

```rust
    fn find_by_id(&self, id: Uuid) -> Result<Option<Device>, CustomError> {
        self.inner.find_by_id(id).map_err(Into::into)
    }

    fn delete_by_id(&self, id: Uuid) -> Result<usize, CustomError> {
        self.inner.delete_by_id(id).map_err(Into::into)
    }
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p podfetch-persistence mopidy_persistence_tests`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add migrations/sqlite/2026-06-04-130000_devices_base_url migrations/postgres/2026-06-04-130000_devices_base_url crates/podfetch-persistence/src/device.rs crates/podfetch-persistence/src/adapters.rs crates/podfetch-persistence/src/schema.rs
git commit -m "feat(mopidy): persist devices.base_url and list mopidy as castable (#505)"
```

---

## Task 4: Mopidy JSON-RPC client + pure mappings

**Files:**
- Create: `crates/podfetch-web/src/services/mopidy/mod.rs`
- Create: `crates/podfetch-web/src/services/mopidy/rpc.rs`
- Modify: `crates/podfetch-web/src/services/mod.rs`

- [ ] **Step 1: Register the module**

In `crates/podfetch-web/src/services/mod.rs` add (keep alphabetical with the existing `pub mod` lines):

```rust
pub mod mopidy;
```

Create `crates/podfetch-web/src/services/mopidy/mod.rs` (declare modules incrementally — `driver` and `consumer` are added in Tasks 5 and 7 as those files are created):

```rust
pub mod rpc;
```

- [ ] **Step 2: Write the failing test**

Create `crates/podfetch-web/src/services/mopidy/rpc.rs`:

```rust
//! Mopidy JSON-RPC client and the pure value mappings between PodFetch's
//! cast value types and Mopidy's `core.*` API.

use podfetch_cast::{CastState, ControlCmd};
use serde_json::{Value, json};

#[derive(Debug, thiserror::Error)]
pub enum MopidyRpcError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("mopidy returned error {code}: {message}")]
    Rpc { code: i64, message: String },
    #[error("unexpected response: {0}")]
    Decode(String),
}

/// Build a JSON-RPC 2.0 request envelope.
pub fn build_request(method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params })
}

/// Extract the `result` from a JSON-RPC response, mapping an `error` object
/// to [`MopidyRpcError::Rpc`].
pub fn parse_response(v: Value) -> Result<Value, MopidyRpcError> {
    if let Some(err) = v.get("error") {
        let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
        let message = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        return Err(MopidyRpcError::Rpc { code, message });
    }
    Ok(v.get("result").cloned().unwrap_or(Value::Null))
}

pub fn state_from_str(s: &str) -> CastState {
    match s {
        "playing" => CastState::Playing,
        "paused" => CastState::Paused,
        _ => CastState::Stopped,
    }
}

pub fn volume_to_mopidy(v: f32) -> i64 {
    (v.clamp(0.0, 1.0) * 100.0).round() as i64
}

pub fn volume_from_mopidy(v: i64) -> f32 {
    (v as f32 / 100.0).clamp(0.0, 1.0)
}

pub fn secs_to_ms(secs: f64) -> i64 {
    (secs.max(0.0) * 1000.0) as i64
}

pub fn ms_to_secs(ms: i64) -> f64 {
    ms.max(0) as f64 / 1000.0
}

/// Map a PodFetch control command to the Mopidy `(method, params)` to call.
pub fn control_to_call(cmd: &ControlCmd) -> (&'static str, Value) {
    match cmd {
        ControlCmd::Pause => ("core.playback.pause", json!({})),
        ControlCmd::Resume => ("core.playback.resume", json!({})),
        ControlCmd::Stop => ("core.playback.stop", json!({})),
        ControlCmd::Seek { position_secs } => (
            "core.playback.seek",
            json!({ "time_position": secs_to_ms(*position_secs) }),
        ),
        ControlCmd::SetVolume { volume } => (
            "core.mixer.set_volume",
            json!({ "volume": volume_to_mopidy(*volume) }),
        ),
    }
}

/// Thin async client over `POST {base_url}/mopidy/rpc`.
pub struct MopidyRpcClient {
    http: reqwest::Client,
    rpc_url: String,
}

impl MopidyRpcClient {
    pub fn new(base_url: &str) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            http: reqwest::Client::new(),
            rpc_url: format!("{base}/mopidy/rpc"),
        }
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value, MopidyRpcError> {
        let body = build_request(method, params);
        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| MopidyRpcError::Transport(e.to_string()))?;
        let v: Value = resp
            .json()
            .await
            .map_err(|e| MopidyRpcError::Decode(e.to_string()))?;
        parse_response(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_request_is_jsonrpc_2() {
        let req = build_request("core.playback.play", json!({}));
        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["method"], "core.playback.play");
    }

    #[test]
    fn parse_response_returns_result_or_error() {
        let ok = parse_response(json!({"jsonrpc":"2.0","id":1,"result":"playing"})).unwrap();
        assert_eq!(ok, json!("playing"));

        let err = parse_response(json!({"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"nope"}}));
        match err {
            Err(MopidyRpcError::Rpc { code, message }) => {
                assert_eq!(code, -32601);
                assert_eq!(message, "nope");
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[test]
    fn conversions_round_trip() {
        assert_eq!(volume_to_mopidy(0.5), 50);
        assert_eq!(volume_from_mopidy(50), 0.5);
        assert_eq!(secs_to_ms(1.5), 1500);
        assert_eq!(ms_to_secs(1500), 1.5);
        assert_eq!(state_from_str("paused"), CastState::Paused);
        assert_eq!(state_from_str("anything-else"), CastState::Stopped);
    }

    #[test]
    fn control_maps_to_mopidy_methods() {
        assert_eq!(control_to_call(&ControlCmd::Pause).0, "core.playback.pause");
        let (method, params) = control_to_call(&ControlCmd::Seek { position_secs: 2.0 });
        assert_eq!(method, "core.playback.seek");
        assert_eq!(params["time_position"], 2000);
        let (method, params) = control_to_call(&ControlCmd::SetVolume { volume: 1.0 });
        assert_eq!(method, "core.mixer.set_volume");
        assert_eq!(params["volume"], 100);
    }
}
```

- [ ] **Step 3: Run test to verify it fails then passes**

Run: `cargo test -p podfetch-web services::mopidy::rpc::tests`
Expected: PASS (4 tests).

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/mod.rs crates/podfetch-web/src/services/mopidy/mod.rs crates/podfetch-web/src/services/mopidy/rpc.rs
git commit -m "feat(mopidy): add JSON-RPC client and cast<->mopidy value mappings (#505)"
```

---

## Task 5: MopidyDriver + target + event + status pump

**Files:**
- Create: `crates/podfetch-web/src/services/mopidy/driver.rs`
- Modify: `crates/podfetch-web/src/services/mopidy/mod.rs` (add `pub mod driver;`)

The driver reuses `podfetch_cast` value types and `crate::events::CastEndedReason`. It keeps a per-session map; the pump task is cancelled via a `tokio::sync::watch` channel.

- [ ] **Step 1: Write the failing test**

Add `pub mod driver;` to `crates/podfetch-web/src/services/mopidy/mod.rs` (it should now read `pub mod driver;` + `pub mod rpc;`), then create `crates/podfetch-web/src/services/mopidy/driver.rs`:

```rust
//! In-process Mopidy playback driver. Sibling of `AgentDispatcher` — it is
//! deliberately NOT a `CastDriver` impl, because `CastTarget` is Chromecast
//! shaped (raw IpAddr + fixed port) and cannot represent a URL-addressed
//! server. It reuses the shared cast value types and emits status over a
//! channel that the startup-wired consumer drains into the orchestrator.

use crate::events::CastEndedReason;
use crate::services::mopidy::rpc::{
    self, MopidyRpcClient, control_to_call, ms_to_secs, state_from_str, volume_from_mopidy,
};
use chrono::Utc;
use podfetch_cast::{CastMedia, CastSessionId, CastState, CastStatus, ControlCmd};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

const POLL_INTERVAL: Duration = Duration::from_millis(1500);

/// Where a Mopidy play/control command is routed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MopidyTarget {
    pub base_url: String,
}

/// Events emitted by per-session pumps, drained by the consumer.
#[derive(Debug, Clone)]
pub enum MopidyEvent {
    Status(CastStatus),
    SessionEnded {
        session_id: CastSessionId,
        reason: CastEndedReason,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MopidyDriveError {
    #[error("mopidy rpc: {0}")]
    Rpc(#[from] rpc::MopidyRpcError),
    #[error("session {0:?} not active")]
    SessionGone(CastSessionId),
}

struct ActiveMopidySession {
    base_url: String,
    cancel: tokio::sync::watch::Sender<bool>,
}

pub struct MopidyDriver {
    event_tx: mpsc::Sender<MopidyEvent>,
    sessions: Mutex<HashMap<CastSessionId, ActiveMopidySession>>,
}

/// Decide whether a freshly polled state means the session ended.
/// `has_played` is true once we have observed a non-stopped state, so an
/// initial `Stopped` while buffering does not end the session prematurely.
pub fn end_reason_for_poll(has_played: bool, state: CastState) -> Option<CastEndedReason> {
    if has_played && state == CastState::Stopped {
        Some(CastEndedReason::Finished)
    } else {
        None
    }
}

impl MopidyDriver {
    pub fn new(event_tx: mpsc::Sender<MopidyEvent>) -> Self {
        Self {
            event_tx,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Connection test used by the management API. Returns the version string.
    pub async fn ping(base_url: &str) -> Result<String, MopidyDriveError> {
        let client = MopidyRpcClient::new(base_url);
        let v = client.call("core.get_version", json!({})).await?;
        Ok(v.as_str().unwrap_or_default().to_string())
    }

    pub async fn play(
        &self,
        target: &MopidyTarget,
        media: &CastMedia,
        resume_secs: Option<f64>,
    ) -> Result<CastSessionId, MopidyDriveError> {
        let client = MopidyRpcClient::new(&target.base_url);
        client.call("core.tracklist.clear", json!({})).await?;
        client
            .call("core.tracklist.add", json!({ "uris": [media.url] }))
            .await?;
        client.call("core.playback.play", json!({})).await?;
        if let Some(secs) = resume_secs.filter(|s| *s > 0.0) {
            let (method, params) = control_to_call(&ControlCmd::Seek { position_secs: secs });
            let _ = client.call(method, params).await;
        }

        let session_id = CastSessionId::new();
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        self.sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .insert(
                session_id.clone(),
                ActiveMopidySession {
                    base_url: target.base_url.clone(),
                    cancel: cancel_tx,
                },
            );

        let event_tx = self.event_tx.clone();
        let base_url = target.base_url.clone();
        let pump_session = session_id.clone();
        tokio::spawn(async move {
            run_pump(base_url, pump_session, event_tx, cancel_rx).await;
        });

        Ok(session_id)
    }

    pub async fn control(
        &self,
        session_id: &CastSessionId,
        cmd: &ControlCmd,
    ) -> Result<(), MopidyDriveError> {
        let base_url = {
            let sessions = self.sessions.lock().expect("mopidy session lock poisoned");
            sessions
                .get(session_id)
                .map(|s| s.base_url.clone())
                .ok_or_else(|| MopidyDriveError::SessionGone(session_id.clone()))?
        };
        let client = MopidyRpcClient::new(&base_url);
        let (method, params) = control_to_call(cmd);
        client.call(method, params).await?;

        if matches!(cmd, ControlCmd::Stop) {
            self.finish_session(session_id, CastEndedReason::Stopped);
        }
        Ok(())
    }

    fn finish_session(&self, session_id: &CastSessionId, reason: CastEndedReason) {
        if let Some(session) = self
            .sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .remove(session_id)
        {
            let _ = session.cancel.send(true);
            let _ = self.event_tx.try_send(MopidyEvent::SessionEnded {
                session_id: session_id.clone(),
                reason,
            });
        }
    }

    #[cfg(test)]
    pub fn knows_session(&self, session_id: &CastSessionId) -> bool {
        self.sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .contains_key(session_id)
    }
}

async fn run_pump(
    base_url: String,
    session_id: CastSessionId,
    event_tx: mpsc::Sender<MopidyEvent>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) {
    let client = MopidyRpcClient::new(&base_url);
    let mut has_played = false;
    loop {
        if *cancel_rx.borrow() {
            return;
        }
        let snapshot = poll_once(&client).await;
        if let Some((state, position_secs, volume)) = snapshot {
            if state != CastState::Stopped {
                has_played = true;
            }
            let status = CastStatus {
                session_id: session_id.clone(),
                state,
                position_secs,
                volume,
                at: Utc::now(),
            };
            if event_tx.send(MopidyEvent::Status(status)).await.is_err() {
                return;
            }
            if let Some(reason) = end_reason_for_poll(has_played, state) {
                let _ = event_tx
                    .send(MopidyEvent::SessionEnded { session_id, reason })
                    .await;
                return;
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            _ = cancel_rx.changed() => return,
        }
    }
}

/// One poll cycle → `(state, position_secs, volume)`. `None` on transport error.
async fn poll_once(client: &MopidyRpcClient) -> Option<(CastState, f64, f32)> {
    let state = match client.call("core.playback.get_state", json!({})).await {
        Ok(Value::String(s)) => state_from_str(&s),
        Ok(_) => CastState::Stopped,
        Err(e) => {
            warn!("mopidy get_state failed: {e}");
            return None;
        }
    };
    let position = client
        .call("core.playback.get_time_position", json!({}))
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .map(ms_to_secs)
        .unwrap_or(0.0);
    let volume = client
        .call("core.mixer.get_volume", json!({}))
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .map(volume_from_mopidy)
        .unwrap_or(1.0);
    Some((state, position, volume))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_reason_ignores_initial_stopped_then_finishes() {
        assert_eq!(end_reason_for_poll(false, CastState::Stopped), None);
        assert_eq!(end_reason_for_poll(true, CastState::Playing), None);
        assert_eq!(
            end_reason_for_poll(true, CastState::Stopped),
            Some(CastEndedReason::Finished)
        );
    }

    #[tokio::test]
    async fn control_on_unknown_session_is_session_gone() {
        let (tx, _rx) = mpsc::channel(4);
        let driver = MopidyDriver::new(tx);
        let err = driver
            .control(&CastSessionId("ghost".into()), &ControlCmd::Pause)
            .await;
        assert!(matches!(err, Err(MopidyDriveError::SessionGone(_))));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail then pass**

Run: `cargo test -p podfetch-web services::mopidy::driver::tests`
Expected: PASS (2 tests).

- [ ] **Step 3: Commit**

```bash
git add crates/podfetch-web/src/services/mopidy/driver.rs crates/podfetch-web/src/services/mopidy/mod.rs
git commit -m "feat(mopidy): add MopidyDriver with status pump and end detection (#505)"
```

---

## Task 6: Orchestrator routing + AppState wiring

**Files:**
- Modify: `crates/podfetch-web/src/services/cast/service.rs`
- Modify: `crates/podfetch-web/src/app_state.rs`

- [ ] **Step 1: Write the failing test**

In `crates/podfetch-web/src/services/cast/service.rs`, the existing `#[cfg(test)] mod tests` builds the orchestrator via a helper `orchestrator(devices)` calling `CastOrchestrator::new(device_service, Arc::new(StubCastDriver), dispatcher)`. Add a Mopidy routing test (place inside that `mod tests`):

```rust
    #[tokio::test]
    async fn start_against_mopidy_device_routes_to_mopidy_driver() {
        let alice = user(1, "user");
        let mut device = make_device(20, alice.id, device_kind::MOPIDY_SHARED, "mopidy-uuid");
        device.base_url = Some("http://127.0.0.1:1/".to_string()); // unreachable on purpose
        let orch = orchestrator(vec![device]);

        let media = CastMedia {
            url: "https://example.com/a.mp3".into(),
            mime: "audio/mpeg".into(),
            title: "Ep".into(),
            artwork_url: None,
            duration_secs: None,
            episode_id: None,
        };
        // The mopidy server is unreachable, so start() returns a Cast/transport
        // error — proving the request was routed to the Mopidy branch (the
        // StubCastDriver would have returned NotImplemented instead).
        let err = orch
            .start(&alice, "mopidy-uuid", media, None, None)
            .await;
        assert!(matches!(err, Err(OrchestratorError::Mopidy(_))));
    }
```

Update the `make_device` test helper to set `base_url: None,` in its `Device { .. }` literal (it currently omits it).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p podfetch-web start_against_mopidy_device_routes`
Expected: FAIL — `OrchestratorError::Mopidy` missing; `base_url` missing from `make_device`; routing not implemented.

- [ ] **Step 3: Implement orchestrator changes**

In `crates/podfetch-web/src/services/cast/service.rs`:

Add imports at the top:

```rust
use crate::services::mopidy::driver::{MopidyDriver, MopidyDriveError, MopidyTarget};
```

Add a field to `struct CastOrchestrator<L: CastDriver>` (after `agent_dispatcher`):

```rust
    mopidy_driver: Arc<MopidyDriver>,
```

Update `CastOrchestrator::new` to take and store it:

```rust
    pub fn new(
        device_service: Arc<DeviceService>,
        local_driver: Arc<L>,
        agent_dispatcher: Arc<AgentDispatcher>,
        mopidy_driver: Arc<MopidyDriver>,
    ) -> Self {
        Self {
            device_service,
            local_driver,
            agent_dispatcher,
            mopidy_driver,
            sessions: RwLock::new(HashMap::new()),
        }
    }
```

Add an error variant to `enum OrchestratorError`:

```rust
    #[error("mopidy: {0}")]
    Mopidy(#[from] MopidyDriveError),
```

And map it in `impl From<OrchestratorError> for CustomError` (add arm before the closing `}` of the match):

```rust
            OrchestratorError::Mopidy(e) => {
                CustomErrorInner::BadRequest(e.to_string(), ErrorSeverity::Warning).into()
            }
```

Add `device_kind: String` to `struct ActiveSession` (after `agent_id`) and set it where `ActiveSession` is built in `start()`. First, in `start()`, replace the routing block:

```rust
    pub async fn start(
        &self,
        user: &User,
        chromecast_uuid: &str,
        media: CastMedia,
        episode_id: Option<Uuid>,
        episode_string_id: Option<String>,
    ) -> Result<ActiveSession, OrchestratorError> {
        let device = self.resolve_castable(user, chromecast_uuid)?;
        let device_kind_str = device.kind.clone();
        let device_uuid = CastDeviceUuid(
            device
                .chromecast_uuid
                .clone()
                .ok_or(OrchestratorError::DeviceNotFound)?,
        );

        let (session_id, agent_id) = if device_kind::is_mopidy(&device.kind) {
            let base_url = device
                .base_url
                .clone()
                .ok_or(OrchestratorError::DeviceUnreachable)?;
            let target = MopidyTarget { base_url };
            let sid = self
                .mopidy_driver
                .play(&target, &media, episode_string_id_position(&episode_string_id))
                .await?;
            (sid, None)
        } else {
            let target = build_target(&device)?;
            let agent_id = device.agent_id.clone();
            let sid = match &agent_id {
                Some(id) => self.agent_dispatcher.play(id, &target, &media).await?,
                None => self.local_driver.play(&target, &media).await?,
            };
            (sid, agent_id)
        };

        let status = CastStatus {
            session_id: session_id.clone(),
            state: CastState::Buffering,
            position_secs: 0.0,
            volume: 1.0,
            at: Utc::now(),
        };
        let active = ActiveSession {
            session_id: session_id.clone(),
            device_uuid,
            user_id: user.id,
            username: user.username.clone(),
            episode_id,
            episode_string_id,
            agent_id,
            device_kind: device_kind_str,
            last_status: status,
        };
        self.sessions
            .write()
            .expect("orchestrator session lock poisoned")
            .insert(session_id, active.clone());
        Ok(active)
    }
```

Add a small free helper near `build_target` (Mopidy has no resume position in v1; this keeps the signature explicit and is a single place to add resume later):

```rust
/// v1 does not resume mid-episode on Mopidy; always start from the beginning.
fn episode_string_id_position(_episode_string_id: &Option<String>) -> Option<f64> {
    None
}
```

Update `control()` to route Mopidy sessions:

```rust
    pub async fn control(
        &self,
        user: &User,
        session_id: &CastSessionId,
        cmd: ControlCmd,
    ) -> Result<(), OrchestratorError> {
        let session = self.lookup_session(user, session_id)?;
        if device_kind::is_mopidy(&session.device_kind) {
            self.mopidy_driver.control(&session.session_id, &cmd).await?;
        } else {
            match &session.agent_id {
                Some(id) => {
                    self.agent_dispatcher
                        .control(id, &session.session_id, &cmd)
                        .await?
                }
                None => self.local_driver.control(&session.session_id, &cmd).await?,
            }
        }
        Ok(())
    }
```

Add `use podfetch_domain::device::kind as device_kind;` if not already imported (the file already imports `kind as device_kind`).

Update the test helpers in the same file's `mod tests`:
- `make_device(...)` `Device { .. }` literal: add `base_url: None,`.
- Any place that builds `ActiveSession { .. }` literally (e.g. `record_status_returns_owning_user`): add `device_kind: device_kind::CHROMECAST_PERSONAL.to_string(),`.
- `FakeDeviceRepo::list_castable_for_user` filters Chromecast-only; broaden it so the Mopidy routing test's `resolve_castable` can find the shared Mopidy device. Replace its `.filter(|d| { ... })` predicate with:

```rust
                .filter(|d| {
                    d.kind == device_kind::CHROMECAST_SHARED
                        || d.kind == device_kind::MOPIDY_SHARED
                        || ((d.kind == device_kind::CHROMECAST_PERSONAL
                            || d.kind == device_kind::MOPIDY_PERSONAL)
                            && d.user_id == viewer_user_id)
                })
```

- The `orchestrator(...)` and `orchestrator_with_dispatcher(...)` helpers: pass a Mopidy driver. Add at the top of each helper:

```rust
        let (mopidy_tx, _mopidy_rx) = tokio::sync::mpsc::channel(8);
        let mopidy_driver = std::sync::Arc::new(
            crate::services::mopidy::driver::MopidyDriver::new(mopidy_tx),
        );
```

and pass `mopidy_driver` as the new 4th argument to `CastOrchestrator::new(...)`.

- [ ] **Step 4: Wire AppState**

In `crates/podfetch-web/src/app_state.rs`:

Add imports:

```rust
use crate::services::mopidy::driver::{MopidyDriver, MopidyEvent};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc;
```

Add a field to `struct AppState`:

```rust
    pub mopidy_event_rx: Arc<AsyncMutex<Option<mpsc::Receiver<MopidyEvent>>>>,
```

In `AppState::new()`, replace the `cast_orchestrator` construction:

```rust
        let (mopidy_tx, mopidy_rx) = mpsc::channel::<MopidyEvent>(64);
        let mopidy_driver = Arc::new(MopidyDriver::new(mopidy_tx));
        let cast_orchestrator = Arc::new(CastOrchestrator::new(
            device_service.clone(),
            Arc::new(StubCastDriver),
            agent_dispatcher.clone(),
            mopidy_driver,
        ));
```

Add `mopidy_event_rx: Arc::new(AsyncMutex::new(Some(mopidy_rx))),` to the returned `Self { ... }` literal.

`StubCastDriver` is already imported in `cast.rs`; ensure `app_state.rs` imports it (it already constructs `Arc::new(StubCastDriver)`).

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p podfetch-web services::cast::service::tests`
Expected: PASS, including `start_against_mopidy_device_routes_to_mopidy_driver`.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/services/cast/service.rs crates/podfetch-web/src/app_state.rs
git commit -m "feat(mopidy): route cast orchestrator start/control to mopidy by kind (#505)"
```

---

## Task 7: Status consumer + startup wiring

**Files:**
- Create: `crates/podfetch-web/src/services/mopidy/consumer.rs`
- Modify: `crates/podfetch-web/src/services/mopidy/mod.rs` (add `pub mod consumer;`)
- Modify: `crates/podfetch-web/src/startup.rs`

- [ ] **Step 1: Create the consumer**

Create `crates/podfetch-web/src/services/mopidy/consumer.rs`:

```rust
//! Drains [`MopidyEvent`]s from the driver and feeds them into the cast
//! orchestrator — the in-process analogue of how `agent_ws_controller`
//! handles `AgentMsg::Status` / `SessionEnded` from the LAN agent.

use crate::app_state::AppState;
use crate::server::ChatServerHandle;
use crate::services::cast::service::ActiveSession;
use crate::services::mopidy::driver::MopidyEvent;
use crate::usecases::watchtime::WatchtimeUseCase;
use tokio::sync::mpsc;
use tracing::warn;

/// Spawn the consumer loop. Call once at startup when the integration is on.
pub fn spawn_status_consumer(state: AppState, mut rx: mpsc::Receiver<MopidyEvent>) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                MopidyEvent::Status(status) => {
                    if let Some(session) = state.cast_orchestrator.record_status(status.clone()) {
                        ChatServerHandle::broadcast_cast_status(status.clone());
                        persist_watchtime(&session, status.position_secs);
                    }
                }
                MopidyEvent::SessionEnded { session_id, reason } => {
                    if let Some(session) = state.cast_orchestrator.drop_session(&session_id) {
                        persist_watchtime(&session, session.last_status.position_secs);
                        ChatServerHandle::broadcast_cast_ended(session_id, reason);
                    }
                }
            }
        }
    });
}

fn persist_watchtime(session: &ActiveSession, position_secs: f64) {
    let Some(podcast_episode_id) = session.episode_string_id.clone() else {
        return;
    };
    let username = session.username.clone();
    let position = position_secs.max(0.0).min(f64::from(i32::MAX)) as i32;
    tokio::task::spawn_blocking(move || {
        if let Err(err) = WatchtimeUseCase::log_watchtime(&podcast_episode_id, position, username) {
            warn!("mopidy watchtime persist failed: {err}");
        }
    });
}
```

> Import paths confirmed from `agent_ws_controller.rs`: `use crate::server::ChatServerHandle;` and `use crate::usecases::watchtime::WatchtimeUseCase;` (it calls `WatchtimeUseCase::log_watchtime` and `ChatServerHandle::broadcast_cast_status` / `broadcast_cast_ended`).

Add `pub mod consumer;` to `crates/podfetch-web/src/services/mopidy/mod.rs` so it now contains all three modules:

```rust
pub mod consumer;
pub mod driver;
pub mod rpc;
```

- [ ] **Step 2: Wire startup**

In `crates/podfetch-web/src/startup.rs`, find the block around line 459 (`if ENVIRONMENT_SERVICE.audiobookshelf_integration_enabled { start_audiobookshelf_file_watcher(&state); }`, inside `build_server_router`). Add, right after that block:

```rust
    if ENVIRONMENT_SERVICE.mopidy_integration_enabled {
        if let Some(rx) = state.mopidy_event_rx.lock().await.take() {
            crate::services::mopidy::consumer::spawn_status_consumer(state.clone(), rx);
        }
    }
```

> If the enclosing function is not `async` at that point, the `.lock().await` won't compile. The surrounding socket.io setup block is inside an async context (it `.await`s elsewhere). If a borrow-checker/async error appears, use the blocking lock instead: replace the field type plan with `std::sync::Mutex` in Task 6 and call `state.mopidy_event_rx.lock().unwrap().take()` here (no `.await`). Prefer `std::sync::Mutex` if `build_server_router` is synchronous — confirm by checking whether the function signature is `async fn`.

Mount the management router in `get_private_api` (around line 343, after `get_cast_router`):

```rust
    let router = router.merge(get_mopidy_router().with_state(state.clone()));
```

…but only when enabled. Concretely, change `get_private_api` so the base `router` conditionally includes the Mopidy routes. Replace the `.merge(get_cast_router()...)` chain's start with a conditional:

```rust
    let mut router = OpenApiRouter::new()
        .merge(get_cast_router().with_state(state.clone()))
        .merge(get_discover_router().with_state(state.clone()))
        // ...keep the rest of the existing merges unchanged...
        .merge(get_user_router().with_state(state.clone()));

    if ENVIRONMENT_SERVICE.mopidy_integration_enabled {
        router = router.merge(get_mopidy_router().with_state(state.clone()));
    }
```

Add the import at the top of `startup.rs` next to `use crate::controllers::cast_controller::get_cast_router;`:

```rust
use crate::controllers::mopidy_controller::get_mopidy_router;
```

> `get_mopidy_router` is created in Task 8. If implementing strictly in order, add the import + mount in Task 8 instead and keep this task to the consumer wiring only. The recommended order is: do the consumer wiring here, then add the router import/mount as the final step of Task 8.

- [ ] **Step 3: Verify build**

Run: `cargo build -p podfetch-web` (after Task 8 if the router import was deferred).
Expected: compiles. Until Task 8, comment the router import/mount lines.

- [ ] **Step 4: Commit**

```bash
git add crates/podfetch-web/src/services/mopidy/consumer.rs crates/podfetch-web/src/services/mopidy/mod.rs crates/podfetch-web/src/startup.rs
git commit -m "feat(mopidy): wire status consumer into startup (#505)"
```

---

## Task 8: Management controller `/mopidy/servers`

**Files:**
- Create: `crates/podfetch-web/src/controllers/mopidy_controller.rs`
- Modify: `crates/podfetch-web/src/controllers/mod.rs`
- Modify: `crates/podfetch-web/src/services/device/service.rs`
- Modify: `crates/podfetch-web/src/startup.rs` (router import/mount from Task 7)

- [ ] **Step 1: Add DeviceService passthroughs**

In `crates/podfetch-web/src/services/device/service.rs`, add inside `impl DeviceService`:

```rust
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<Device>, CustomError> {
        self.repository.find_by_id(id)
    }

    pub fn delete_by_id(&self, id: Uuid) -> Result<usize, CustomError> {
        self.repository.delete_by_id(id)
    }
```

- [ ] **Step 2: Register the controller module**

In `crates/podfetch-web/src/controllers/mod.rs`, add (alphabetical with siblings):

```rust
pub mod mopidy_controller;
```

- [ ] **Step 3: Write the failing test + controller**

Create `crates/podfetch-web/src/controllers/mopidy_controller.rs`:

```rust
//! Management API for Mopidy servers (gated behind MOPIDY_INTEGRATION_ENABLED
//! at the router-mount level). A server is stored as a `mopidy_*` Device with
//! a generated `chromecast_uuid` public handle and a `base_url`.

use crate::app_state::AppState;
use crate::services::mopidy::driver::MopidyDriver;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_domain::device::{Device, kind as device_kind};
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct AddMopidyServerRequest {
    pub name: String,
    pub url: String,
    /// When true the server is visible to every user; otherwise owner-only.
    pub shared: bool,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct TestMopidyServerRequest {
    pub url: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct MopidyServerResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub kind: String,
}

impl MopidyServerResponse {
    fn from_device(device: &Device) -> Option<Self> {
        Some(Self {
            id: device.id?.to_string(),
            name: device.name.clone(),
            url: device.base_url.clone()?,
            kind: device.kind.clone(),
        })
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct MopidyTestResult {
    pub reachable: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

fn normalize_url(raw: &str) -> Result<String, CustomError> {
    let trimmed = raw.trim().trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(CustomErrorInner::BadRequest(
            "Mopidy URL must start with http:// or https://".to_string(),
            ErrorSeverity::Warning,
        )
        .into());
    }
    Ok(trimmed.to_string())
}

fn require_admin(user: &User) -> Result<(), CustomError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into())
    }
}

#[utoipa::path(
    get,
    path = "/mopidy/servers",
    responses((status = 200, description = "Mopidy servers visible to the caller", body = Vec<MopidyServerResponse>)),
    tag = "mopidy"
)]
pub async fn list_mopidy_servers(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<MopidyServerResponse>>, CustomError> {
    let devices = state.device_service.list_castable_for_user(user.id)?;
    Ok(Json(
        devices
            .iter()
            .filter(|d| device_kind::is_mopidy(&d.kind))
            .filter_map(MopidyServerResponse::from_device)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/mopidy/servers",
    request_body = AddMopidyServerRequest,
    responses(
        (status = 200, description = "Server added", body = MopidyServerResponse),
        (status = 400, description = "Invalid URL or server unreachable"),
        (status = 403, description = "Admin only")
    ),
    tag = "mopidy"
)]
pub async fn add_mopidy_server(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<AddMopidyServerRequest>,
) -> Result<Json<MopidyServerResponse>, CustomError> {
    require_admin(&user)?;
    let url = normalize_url(&req.url)?;
    MopidyDriver::ping(&url).await.map_err(|e| {
        CustomError::from(CustomErrorInner::BadRequest(
            format!("Mopidy server unreachable: {e}"),
            ErrorSeverity::Warning,
        ))
    })?;

    let kind = if req.shared {
        device_kind::MOPIDY_SHARED
    } else {
        device_kind::MOPIDY_PERSONAL
    };
    let device = Device {
        id: None,
        deviceid: url.clone(),
        kind: kind.to_string(),
        name: req.name,
        user_id: user.id,
        chromecast_uuid: Some(Uuid::new_v4().to_string()),
        agent_id: None,
        last_seen_at: None,
        ip: None,
        base_url: Some(url),
    };
    let created = state.device_service.create(device)?;
    MopidyServerResponse::from_device(&created).map(Json).ok_or_else(|| {
        CustomErrorInner::BadRequest(
            "could not build server response".to_string(),
            ErrorSeverity::Error,
        )
        .into()
    })
}

#[utoipa::path(
    post,
    path = "/mopidy/servers/test",
    request_body = TestMopidyServerRequest,
    responses((status = 200, description = "Connection test result", body = MopidyTestResult)),
    tag = "mopidy"
)]
pub async fn test_mopidy_server(
    Extension(user): Extension<User>,
    Json(req): Json<TestMopidyServerRequest>,
) -> Result<Json<MopidyTestResult>, CustomError> {
    require_admin(&user)?;
    let url = normalize_url(&req.url)?;
    match MopidyDriver::ping(&url).await {
        Ok(version) => Ok(Json(MopidyTestResult {
            reachable: true,
            version: Some(version),
            error: None,
        })),
        Err(e) => Ok(Json(MopidyTestResult {
            reachable: false,
            version: None,
            error: Some(e.to_string()),
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/mopidy/servers/{id}",
    responses(
        (status = 200, description = "Server deleted"),
        (status = 403, description = "Not allowed"),
        (status = 404, description = "Not found")
    ),
    tag = "mopidy"
)]
pub async fn delete_mopidy_server(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> Result<(), CustomError> {
    let uuid = Uuid::parse_str(&id).map_err(|_| {
        CustomError::from(CustomErrorInner::BadRequest(
            "invalid server id".to_string(),
            ErrorSeverity::Warning,
        ))
    })?;
    let device = state
        .device_service
        .find_by_id(uuid)?
        .filter(|d| device_kind::is_mopidy(&d.kind))
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;

    // Admin can delete any; a non-admin may only delete their own personal server.
    let owns = device.user_id == user.id && device.kind == device_kind::MOPIDY_PERSONAL;
    if !user.is_admin() && !owns {
        return Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into());
    }
    state.device_service.delete_by_id(uuid)?;
    Ok(())
}

pub fn get_mopidy_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_mopidy_servers))
        .routes(routes!(add_mopidy_server))
        .routes(routes!(test_mopidy_server))
        .routes(routes!(delete_mopidy_server))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rejects_non_http_and_trims_slash() {
        assert!(normalize_url("ftp://x").is_err());
        assert_eq!(normalize_url("http://m.local:6680/").unwrap(), "http://m.local:6680");
    }
}
```

> Variant names confirmed against `common_infrastructure::error`: `NotFound(ErrorSeverity)`, `Forbidden(ErrorSeverity)`, `BadRequest(String, ErrorSeverity)`, `Conflict(String, ErrorSeverity)`. Use these signatures exactly.

- [ ] **Step 4: Add the router import + mount in startup**

Complete the deferred lines from Task 7 in `crates/podfetch-web/src/startup.rs`: uncomment/add `use crate::controllers::mopidy_controller::get_mopidy_router;` and the `if ENVIRONMENT_SERVICE.mopidy_integration_enabled { router = router.merge(get_mopidy_router().with_state(state.clone())); }` mount.

- [ ] **Step 5: Run tests**

Run: `cargo test -p podfetch-web mopidy_controller`
Expected: PASS (`normalize_rejects_non_http_and_trims_slash`).

Run: `cargo build -p podfetch-web`
Expected: compiles.

- [ ] **Step 6: Commit**

```bash
git add crates/podfetch-web/src/controllers/mopidy_controller.rs crates/podfetch-web/src/controllers/mod.rs crates/podfetch-web/src/services/device/service.rs crates/podfetch-web/src/startup.rs
git commit -m "feat(mopidy): add /mopidy/servers management API (#505)"
```

---

## Task 9: Frontend — settings page, route, nav, CastButton label, i18n

**Files:**
- Modify: `ui/schema.d.ts` (regenerate)
- Create: `ui/src/pages/MopidyIntegration.tsx`
- Modify: `ui/src/App.tsx`
- Modify: `ui/src/pages/SettingsPage.tsx`
- Modify: `ui/src/components/CastButton.tsx`
- Modify: `ui/src/language/json/{en,de,da,es,fr,pl,zh}.json`

- [ ] **Step 1: Regenerate the OpenAPI types**

Start the backend with the flag on, then regenerate:

```bash
# terminal A (PowerShell): build + run with the integration enabled
$env:MOPIDY_INTEGRATION_ENABLED="true"; $env:BASIC_AUTH="false"; cargo run
# terminal B: once http://localhost:8000 is up
pnpm -C ui run generate:types
```

Expected: `ui/schema.d.ts` now contains `mopidyIntegrationEnabled` on `ConfigModel` and the `/api/v1/mopidy/servers` paths + `MopidyServerResponse`, `AddMopidyServerRequest`, `TestMopidyServerRequest`, `MopidyTestResult` schemas. Commit the regenerated file.

- [ ] **Step 2: Add i18n keys**

In `ui/src/language/json/en.json`, add these keys (anywhere in the object):

```json
  "mopidy": "Mopidy",
  "manage-mopidy-servers": "Mopidy servers",
  "mopidy-server-name": "Name",
  "mopidy-server-url": "Server URL",
  "mopidy-server-shared": "Shared with everyone",
  "mopidy-add-server": "Add server",
  "mopidy-test-connection": "Test connection",
  "mopidy-connection-ok": "Connected to Mopidy {{version}}",
  "mopidy-connection-failed": "Could not reach Mopidy: {{error}}",
  "mopidy-server-added": "Mopidy server added",
  "mopidy-server-deleted": "Mopidy server removed",
  "mopidy-no-servers": "No Mopidy servers configured yet.",
  "cast-kind-mopidy": "Mopidy",
  "delete": "Delete"
```

Add the same keys to `de.json`, `da.json`, `es.json`, `fr.json`, `pl.json`, `zh.json`. Translate the values where you can; an English fallback value is acceptable if a translation is unavailable. (`delete` may already exist in some files — skip duplicates.)

- [ ] **Step 3: Create the settings page**

Create `ui/src/pages/MopidyIntegration.tsx`:

```tsx
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQueryClient } from '@tanstack/react-query'
import { $api } from '../utils/http'
import { CustomButtonPrimary } from '../components/CustomButtonPrimary'
import { useSnackbar } from '../utils/toast'

export const MopidyIntegration = () => {
    const { t } = useTranslation()
    const { enqueueSnackbar } = useSnackbar()
    const queryClient = useQueryClient()

    const [name, setName] = useState('')
    const [url, setUrl] = useState('')
    const [shared, setShared] = useState(true)

    const serversQuery = $api.useQuery('get', '/api/v1/mopidy/servers')
    const addServer = $api.useMutation('post', '/api/v1/mopidy/servers')
    const testServer = $api.useMutation('post', '/api/v1/mopidy/servers/test')
    const deleteServer = $api.useMutation('delete', '/api/v1/mopidy/servers/{id}')

    const invalidate = () =>
        queryClient.invalidateQueries({ queryKey: ['get', '/api/v1/mopidy/servers'] })

    const onTest = async () => {
        const result = await testServer.mutateAsync({ body: { url } })
        if (result.reachable) {
            enqueueSnackbar(t('mopidy-connection-ok', { version: result.version ?? '' }), { variant: 'success' })
        } else {
            enqueueSnackbar(t('mopidy-connection-failed', { error: result.error ?? '' }), { variant: 'error' })
        }
    }

    const onAdd = async () => {
        await addServer.mutateAsync({ body: { name, url, shared } })
        enqueueSnackbar(t('mopidy-server-added'), { variant: 'success' })
        setName('')
        setUrl('')
        invalidate()
    }

    const onDelete = async (id: string) => {
        await deleteServer.mutateAsync({ params: { path: { id } } })
        enqueueSnackbar(t('mopidy-server-deleted'), { variant: 'success' })
        invalidate()
    }

    return (
        <div className="flex flex-col gap-6 ui-text">
            <div className="flex flex-col gap-3 max-w-md">
                <label className="flex flex-col gap-1 text-sm">
                    {t('mopidy-server-name')}
                    <input className="ui-input" value={name} onChange={(e) => setName(e.target.value)} />
                </label>
                <label className="flex flex-col gap-1 text-sm">
                    {t('mopidy-server-url')}
                    <input className="ui-input" placeholder="http://mopidy.local:6680" value={url} onChange={(e) => setUrl(e.target.value)} />
                </label>
                <label className="flex items-center gap-2 text-sm">
                    <input type="checkbox" checked={shared} onChange={(e) => setShared(e.target.checked)} />
                    {t('mopidy-server-shared')}
                </label>
                <div className="flex gap-2">
                    <button className="ui-bg-surface hover:bg-(--surface-hover) px-3 py-2 rounded-md text-sm" onClick={onTest} disabled={!url}>
                        {t('mopidy-test-connection')}
                    </button>
                    <CustomButtonPrimary onClick={onAdd} disabled={!name || !url}>
                        {t('mopidy-add-server')}
                    </CustomButtonPrimary>
                </div>
            </div>

            {(serversQuery.data ?? []).length === 0 ? (
                <div className="text-sm ui-text-muted">{t('mopidy-no-servers')}</div>
            ) : (
                <table className="text-left text-sm w-full">
                    <thead>
                        <tr className="border-b ui-border">
                            <th className="px-2 py-3">{t('mopidy-server-name')}</th>
                            <th className="px-2 py-3">{t('mopidy-server-url')}</th>
                            <th className="px-2 py-3">{t('actions')}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {(serversQuery.data ?? []).map((server) => (
                            <tr key={server.id}>
                                <td className="px-2 py-4">{server.name}</td>
                                <td className="px-2 py-4">{server.url}</td>
                                <td className="px-2 py-4">
                                    <button className="ui-text-accent hover:underline" onClick={() => onDelete(server.id)}>
                                        {t('delete')}
                                    </button>
                                </td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            )}
        </div>
    )
}
```

> Match the actual class/util names in the repo. If `ui-input` is not a defined utility, copy the `className` used by an existing settings text input (e.g. in `SettingsNaming`/`Settings`). `useSnackbar` import path is `../utils/toast` (used by `CastButton.tsx`).

- [ ] **Step 4: Add the route**

In `ui/src/App.tsx`, add the import near the `GPodderIntegration` import:

```tsx
import {MopidyIntegration} from "./pages/MopidyIntegration";
```

And add a route inside the `settings` `<Route>` block, after the `gpodder` route:

```tsx
                <Route path="mopidy" element={<MopidyIntegration/>}/>
```

- [ ] **Step 5: Conditional nav link**

In `ui/src/pages/SettingsPage.tsx`, after the `gpodder` `<li>` block, add a Mopidy link gated on the config flag. At the top of the component, read the config (follow the existing pattern — `getConfigFromHtmlFile()` from `../utils/config`):

```tsx
import {getConfigFromHtmlFile} from "../utils/config";
// inside the component:
const config = getConfigFromHtmlFile()
```

Then add:

```tsx
                    {config?.mopidyIntegrationEnabled && (
                        <li className={`cursor-pointer inline-block px-2 py-4`}>
                            <NavLink to="mopidy">
                                {t('manage-mopidy-servers')}
                            </NavLink>
                        </li>
                    )}
```

- [ ] **Step 6: CastButton Mopidy label**

In `ui/src/components/CastButton.tsx`, the device row renders a kind label:
`{device.kind === 'chromecast_shared' ? t('cast-kind-shared') : t('cast-kind-personal')}`. Replace with a Mopidy-aware version:

```tsx
                                    <span className="text-xs ui-text-muted">
                                        {device.kind.startsWith('mopidy')
                                            ? t('cast-kind-mopidy')
                                            : device.kind === 'chromecast_shared'
                                              ? t('cast-kind-shared')
                                              : t('cast-kind-personal')}
                                    </span>
```

- [ ] **Step 7: Build + lint the UI**

Run: `pnpm -C ui run build`
Expected: type-checks and builds with no errors referencing the new code.

Run: `pnpm -C ui run lint` (if a `lint` script exists; otherwise skip)
Expected: no new lint errors.

- [ ] **Step 8: Commit**

```bash
git add ui/schema.d.ts ui/src/pages/MopidyIntegration.tsx ui/src/App.tsx ui/src/pages/SettingsPage.tsx ui/src/components/CastButton.tsx ui/src/language/json
git commit -m "feat(mopidy): add settings page, route, cast label and i18n (#505)"
```

---

## Task 10: Full verification

**Files:** none (verification only)

- [ ] **Step 1: Format + lint + test the workspace**

Run each and confirm clean output:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p common-infrastructure -p podfetch-domain -p podfetch-persistence -p podfetch-web
```

Expected: fmt makes no further changes after committing; clippy passes with no warnings; all tests pass.

- [ ] **Step 2: Manual smoke test (documented, optional but recommended)**

```bash
# With a reachable Mopidy (mopidy + Mopidy-HTTP + Mopidy-Stream) on the LAN:
$env:MOPIDY_INTEGRATION_ENABLED="true"; cargo run
```

In the UI: Settings → Mopidy servers → add `http://<mopidy-host>:6680` (Test connection should report the version) → open an episode → the Cast button popover lists the Mopidy server → pick it → audio plays on Mopidy's output; pause/stop/seek/volume work; the player reflects live status.

- [ ] **Step 3: Commit any fmt changes**

```bash
git add -A
git commit -m "chore(mopidy): cargo fmt (#505)"
```

---

## Notes for the implementer

- **Reachability:** the episode URL handed to Mopidy must be reachable from the Mopidy host (same constraint as Chromecast). For local episodes the server's `local_url` must resolve from where Mopidy runs. No code change needed; document for users.
- **No resume in v1:** `episode_string_id_position` always returns `None`. A later version can look up stored watchtime and pass it to `MopidyDriver::play`.
- **Out of scope (v1):** auth-protected/reverse-proxied Mopidy, the Mopidy WebSocket event stream (we poll), mDNS discovery, adopting externally-started playback. See the design doc §9.
