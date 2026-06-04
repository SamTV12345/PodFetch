# Mopidy Integration — Design

- **Issue:** [#505 — Playing through Mopidy](https://github.com/SamTV12345/PodFetch/issues/505)
- **Date:** 2026-06-04
- **Status:** Approved (pending spec review)

## 1. Overview & scope

Let a PodFetch user stream an episode to a **Mopidy** server, which then plays
through whatever output Mopidy is wired to (speakers, Snapcast, Chromecast via
`mopidy-chromecast`, etc.). PodFetch hands Mopidy the episode's HTTP(S) URL
(played by the `mopidy-stream` extension) and drives playback over Mopidy's
JSON-RPC API.

Mopidy is modelled as a new **device kind** that reuses the existing cast
orchestration stack (sessions, control, status, watchtime persistence, the
`CastButton` device picker). The whole feature is opt-in behind a
`MOPIDY_INTEGRATION_ENABLED` env flag and configured from a settings page —
matching the gpodder / audiobookshelf integration convention.

**Supported playback controls** (reusing the existing `ControlCmd` enum):
play, pause, resume, stop, seek, set-volume.

### Why reuse the cast framework

PodFetch already has a complete remote-playback framework:

- `CastDriver` trait + value types (`CastMedia`, `CastSessionId`, `CastStatus`,
  `ControlCmd`, `CastState`) in the `podfetch-cast` crate.
- `CastOrchestrator` (`crates/podfetch-web/src/services/cast/service.rs`) which
  enforces the per-user / shared-device permission model, tracks active
  sessions, and routes each op to either the local Chromecast driver or the LAN
  agent (`AgentDispatcher`) based on `device.agent_id`.
- REST endpoints under `/cast/*` (`controllers/cast_controller.rs`).
- A `Device` domain model with a `kind` discriminator
  (`chromecast_personal` / `chromecast_shared`).
- The `CastButton` device picker in the player UI (`ui/src/components/CastButton.tsx`).

Mopidy is functionally just another playback target, so it slots into this
framework rather than duplicating session/control/status logic.

## 2. Data model & migration

Extend the `devices` table with one nullable column:

```
base_url TEXT NULL   -- Mopidy RPC base, e.g. http://mopidy.local:6680
```

This is a plain `ALTER TABLE ... ADD COLUMN`, **not** a table rebuild, so the
SQLite FK-pragma gotcha (table-rebuild migrations needing
`run_in_transaction = false`) does not apply here. A matching migration is added
for both the SQLite and PostgreSQL migration trees.

New `kind` discriminators in `podfetch_domain::device::kind`:

```rust
pub const MOPIDY_PERSONAL: &str = "mopidy_personal"; // owner-only
pub const MOPIDY_SHARED:   &str = "mopidy_shared";   // visible to all users

pub fn is_mopidy(kind: &str) -> bool { ... }
pub fn is_castable(kind: &str) -> bool { is_chromecast(kind) || is_mopidy(kind) }
```

The existing `chromecast_uuid` field is **reused as the public device handle**
(a generated UUID v4) for Mopidy devices. It is already the selector that the
cast endpoints and `CastButton` use to start/control a session; for a Mopidy
device it simply isn't a real Chromecast UUID. (A future rename to a
kind-neutral `public_id` is possible but out of scope here.)

The `Device` struct gains `base_url: Option<String>`. For Chromecast rows it
stays `None`; for Mopidy rows it holds the validated RPC base.

## 3. Backend: `MopidyDriver` + JSON-RPC + status pump

A new `MopidyDriver` lives in `podfetch-web` as a **sibling to
`AgentDispatcher`** — it is deliberately **not** a literal `CastDriver` impl.

### Rationale (approved)

`CastDriver::play` takes a `CastTarget { uuid, ip: IpAddr, port }` — a
Chromecast-shaped target with a raw `IpAddr` and the fixed CAST TLS port 8009,
and no room for a scheme, path, or hostname. A URL-addressed server doesn't fit
it. So `MopidyDriver` takes its own `MopidyTarget { base_url }` and reuses the
shared cast value types. This is the same relationship `AgentDispatcher` already
has to the orchestrator (it, too, is routed to without implementing
`CastDriver`).

### JSON-RPC client

All calls are `POST {base_url}/mopidy/rpc` with body:

```json
{ "jsonrpc": "2.0", "id": <n>, "method": "<method>", "params": { ... } }
```

| Operation        | Mopidy call(s)                                                              |
|------------------|----------------------------------------------------------------------------|
| play             | `core.tracklist.clear` → `core.tracklist.add({uris:[url]})` → `core.playback.play()` (+ `core.playback.seek({time_position})` for a resume offset) |
| pause            | `core.playback.pause()`                                                    |
| resume           | `core.playback.resume()`                                                   |
| stop             | `core.playback.stop()`                                                     |
| seek             | `core.playback.seek({time_position: <ms>})`                                |
| set-volume       | `core.mixer.set_volume({volume: <0..100>})`                                |
| status poll      | `core.playback.get_state`, `core.playback.get_time_position`, `core.mixer.get_volume` |
| connection test  | `core.get_version`                                                         |

Volume conversion: `CastStatus.volume` is `0.0..1.0`; Mopidy uses `0..100`
(multiply / divide by 100). Position: Mopidy uses milliseconds; `CastStatus`
uses seconds.

State mapping: Mopidy `"playing" | "paused" | "stopped"` → `CastState::{Playing,
Paused, Stopped}`. `Buffering` is reported transiently at session start before
the first poll.

### Status pump

When a Mopidy session starts, the driver spawns a tokio task that polls every
~1.5s and emits `MopidyEvent`s over an `mpsc` channel:

- `MopidyEvent::Status(CastStatus)` on each poll.
- `MopidyEvent::SessionEnded { session_id, reason }` when the playback state
  becomes `stopped` after having played (finished) or the current track clears.

A startup-wired **consumer task** (holding `AppState`) drains the channel and
calls — exactly mirroring what `agent_ws_controller` does for the LAN agent:

- `cast_orchestrator.record_status(status)` → `ChatServerHandle::broadcast_cast_status(status)` + `persist_watchtime_async(session, position)`.
- on end: `cast_orchestrator.drop_session(id)` → `persist_watchtime_async(...)` + `ChatServerHandle::broadcast_cast_ended(id, reason)`.

The `cast:status` / `cast:ended` socket.io events are already registered in
`server.rs`, so the existing UI live-status path is reused with no change.

This event-channel indirection (rather than the driver calling the orchestrator
directly) avoids a circular dependency between the orchestrator and the driver.

## 4. Orchestrator routing

`CastOrchestrator::{start, control}` currently branch on `device.agent_id`
(LAN agent vs. local `StubCastDriver`). Add a `mopidy_driver: Arc<MopidyDriver>`
field and a third branch keyed on `device.kind`:

```
if is_mopidy(device.kind):
    target = MopidyTarget { base_url: device.base_url }   // error if missing
    route start/control to mopidy_driver
else:
    existing agent / local path  (build_target → CastTarget)
```

`list_castable_for_user` is extended so a `mopidy_shared` device is visible to
every user and a `mopidy_personal` device only to its owner — the same rule
already applied to the Chromecast kinds. As a result Mopidy servers appear in
the existing `/cast/devices` response and the `CastButton` picker with **no
change to the start/control request flow**.

`build_target` stays Chromecast-only; the Mopidy branch builds its own target,
so a missing/garbage `base_url` surfaces as a clear orchestrator error rather
than an `IpAddr` parse failure.

## 5. Management API (`/mopidy/servers`)

A new controller, mounted only when `MOPIDY_INTEGRATION_ENABLED`. Writes are
admin-only (consistent with Chromecast discovery being admin-only):

| Method & path                  | Body / params            | Behaviour                                                                 |
|--------------------------------|--------------------------|---------------------------------------------------------------------------|
| `POST /mopidy/servers`         | `{ name, url, shared }`  | Validate via `core.get_version` RPC ping; store a `mopidy_personal`/`mopidy_shared` Device with a generated `chromecast_uuid` handle and `base_url = url`. |
| `GET  /mopidy/servers`         | —                        | List servers visible to the caller.                                       |
| `POST /mopidy/servers/test`    | `{ url }`                | Connection test (RPC ping) without persisting.                            |
| `DELETE /mopidy/servers/{id}`  | path id                  | Delete a server (admin, or owner of a personal server).                   |

URL normalisation: trim a trailing `/`; reject non-`http(s)` schemes.

## 6. Config / enablement

- Add `MOPIDY_INTEGRATION_ENABLED` constant + field to `EnvironmentService`
  (parsed with the existing `is_env_var_present_and_true`), plus a banner log
  line alongside the gpodder / audiobookshelf lines.
- Expose `mopidyIntegrationEnabled` in the client config
  (`ConfigModel` / `clientconfig.json`) so the UI can conditionally render the
  settings page and Mopidy device entries.
- All `/mopidy/*` routes and the Mopidy orchestrator branch are gated by the
  flag; with the flag off, the feature is entirely inert.

## 7. UI

- **Settings page** `ui/src/pages/MopidyIntegration.tsx`, linked from
  `SettingsPage` and rendered only when `mopidyIntegrationEnabled` is true.
  Contains an add-server form (name, URL, personal/shared toggle, *Test
  connection* button) and a table listing configured servers with a delete
  action. Modeled on the existing `GPodderIntegration.tsx` table + the
  `$api.useQuery` / `useMutation` pattern. A new route is added in `App.tsx`.
- **Playback:** no new control component. Mopidy servers appear in the existing
  `CastButton` popover alongside Chromecasts, with a Mopidy-appropriate kind
  label. Start / stop / seek / volume flow through the existing `/cast/sessions`
  endpoints unchanged.
- **i18n:** new keys (settings nav label, form labels, kind labels, test-result
  and error toasts) added to every locale json under `ui/src/language/json/`.

## 8. Testing

- **Unit (Rust):**
  - JSON-RPC request building and response parsing (success + Mopidy error
    object).
  - `ControlCmd` → Mopidy method mapping; volume 0..1 ↔ 0..100 scaling;
    ms ↔ seconds position conversion.
  - `device::kind` helpers (`is_mopidy`, `is_castable`).
  - Orchestrator routing: a `mopidy_*` device routes to the Mopidy branch; a
    Chromecast device still routes to the existing path; missing `base_url`
    yields a clear error.
  - `list_castable_for_user` visibility for `mopidy_shared` vs `mopidy_personal`
    (mirroring the existing Chromecast visibility tests).
- **Driver / integration:** drive the JSON-RPC client against a mock HTTP server
  (assert request bodies, feed canned responses); an unreachable base URL
  returns a transport error (mirrors the cast driver's "unreachable returns
  Connect error" test).
- **Status pump:** state/position mapping and end-detection (`stopped` after
  play → `SessionEnded(Finished)`).
- **Management API:** add/list/delete happy paths; non-admin write is forbidden;
  invalid URL rejected; test endpoint reports reachable/unreachable.

## 9. Out of scope for v1

- Auth-protected / reverse-proxied Mopidy (URL only, no credentials).
- Subscribing to Mopidy's WebSocket event stream (`/mopidy/ws`) — v1 polls.
- mDNS auto-discovery of `_mopidy-http._tcp`.
- Adopting/controlling playback that Mopidy started on its own.
- Renaming `chromecast_uuid` to a kind-neutral `public_id`.

## 10. Reachability constraint

As with Chromecast, the episode URL handed to Mopidy must be reachable **from
the Mopidy server's network** — for local episodes that means PodFetch's
`local_url` must resolve from where Mopidy runs. The `CastButton` already sends
`ep.url || ep.local_url`; this constraint is documented for users but requires
no code change.
