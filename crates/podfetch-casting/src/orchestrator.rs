use crate::agent::dispatcher::AgentDispatcher;
use crate::devices::{CastDevice, CastDeviceRepository, kind as device_kind};
use crate::mopidy::driver::{MopidyDriveError, MopidyDriver, MopidyTarget};
use chrono::Utc;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_cast::{
    CastDeviceUuid, CastDriver, CastError, CastMedia, CastSessionId, CastState, CastStatus,
    CastTarget, ControlCmd, DiscoveredCastDevice,
};
use podfetch_identity::api::Principal;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::warn;
use uuid::Uuid;

/// Bookkeeping for an active cast session as seen by the orchestrator.
#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub session_id: CastSessionId,
    pub device_uuid: CastDeviceUuid,
    pub user_id: Uuid,
    /// Username of the session owner — cached so watchtime persistence
    /// from the WS handler doesn't need to round-trip the user repo.
    pub username: String,
    pub episode_id: Option<Uuid>,
    /// String episode identifier used by `WatchtimeUseCase::log_watchtime`
    /// (i.e. `PodcastEpisode::episode_id`, the GUID-like string), looked
    /// up at session-start time.
    pub episode_string_id: Option<String>,
    /// Set when the session is being driven through a remote agent;
    /// `None` means the session is on the server's own LAN.
    pub agent_id: Option<String>,
    /// CastDevice kind string — used at control time to route Mopidy vs Chromecast.
    pub device_kind: String,
    pub last_status: CastStatus,
}

/// Routes Chromecast operations on behalf of a user, enforces the
/// per-user / shared-device permission model, and tracks active sessions.
///
/// `L` is the local CAST driver — mDNS+TLS to devices on the server's own
/// LAN. Devices contributed by a connected agent are routed through the
/// [`AgentDispatcher`] instead, picked per-device via `device.agent_id`.
pub struct CastOrchestrator<L: CastDriver> {
    devices: Arc<dyn CastDeviceRepository<Error = CustomError>>,
    local_driver: Arc<L>,
    agent_dispatcher: Arc<AgentDispatcher>,
    mopidy_driver: Arc<MopidyDriver>,
    sessions: RwLock<HashMap<CastSessionId, ActiveSession>>,
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("device not found or not visible")]
    DeviceNotFound,
    #[error("session not found")]
    SessionNotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("device has no IP address recorded")]
    DeviceUnreachable,
    #[error("cast: {0}")]
    Cast(#[from] CastError),
    #[error("mopidy: {0}")]
    Mopidy(#[from] MopidyDriveError),
    #[error(transparent)]
    Persistence(#[from] CustomError),
}

impl From<OrchestratorError> for CustomError {
    fn from(value: OrchestratorError) -> Self {
        match value {
            OrchestratorError::Persistence(e) => e,
            OrchestratorError::DeviceNotFound => {
                CustomErrorInner::NotFound(ErrorSeverity::Warning).into()
            }
            OrchestratorError::SessionNotFound => {
                CustomErrorInner::NotFound(ErrorSeverity::Warning).into()
            }
            OrchestratorError::Forbidden => {
                CustomErrorInner::Forbidden(ErrorSeverity::Warning).into()
            }
            OrchestratorError::DeviceUnreachable => CustomErrorInner::BadRequest(
                "Chromecast has no IP recorded".to_string(),
                ErrorSeverity::Warning,
            )
            .into(),
            OrchestratorError::Cast(e) => {
                CustomErrorInner::BadRequest(e.to_string(), ErrorSeverity::Warning).into()
            }
            OrchestratorError::Mopidy(e) => {
                CustomErrorInner::BadRequest(e.to_string(), ErrorSeverity::Warning).into()
            }
        }
    }
}

impl<L: CastDriver> CastOrchestrator<L> {
    pub fn new(
        devices: Arc<dyn CastDeviceRepository<Error = CustomError>>,
        local_driver: Arc<L>,
        agent_dispatcher: Arc<AgentDispatcher>,
        mopidy_driver: Arc<MopidyDriver>,
    ) -> Self {
        Self {
            devices,
            local_driver,
            agent_dispatcher,
            mopidy_driver,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// All Chromecast devices the user is allowed to see — owned personal
    /// devices plus any shared/household device on the instance.
    pub fn list_castable(&self, user: &Principal) -> Result<Vec<CastDevice>, OrchestratorError> {
        Ok(self.devices.list_visible_to(user.id.into())?)
    }

    /// Resolve a device by chromecast UUID and check the user is allowed to
    /// use it. Returns the persisted device row.
    pub fn resolve_castable(
        &self,
        user: &Principal,
        chromecast_uuid: &str,
    ) -> Result<CastDevice, OrchestratorError> {
        let visible = self.list_castable(user)?;
        visible
            .into_iter()
            .find(|d| d.chromecast_uuid.as_deref() == Some(chromecast_uuid))
            .ok_or(OrchestratorError::DeviceNotFound)
    }

    /// Trigger a fresh discovery scan against the local driver. Admin-only.
    /// Agent-contributed devices are reported asynchronously via DeviceList
    /// over the agent websocket and therefore aren't included here.
    pub async fn discover(
        &self,
        user: &Principal,
    ) -> Result<Vec<DiscoveredCastDevice>, OrchestratorError> {
        if !user.is_admin() {
            return Err(OrchestratorError::Forbidden);
        }
        Ok(self.local_driver.discover().await?)
    }

    /// Start a media session on the given Chromecast. Routes to the local
    /// driver or to the owning agent based on `device.agent_id`.
    ///
    /// `episode_string_id` is the GUID-like string the watchtime store
    /// keys on; the caller looks it up from the numeric episode id before
    /// invoking us (see the cast controller).
    pub async fn start(
        &self,
        user: &Principal,
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
                .play(
                    &target,
                    &media,
                    episode_string_id_position(&episode_string_id),
                )
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
            user_id: user.id.into(),
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

    /// Issue a control command against a session the user owns. Routes to
    /// whichever backend originally started the session.
    pub async fn control(
        &self,
        user: &Principal,
        session_id: &CastSessionId,
        cmd: ControlCmd,
    ) -> Result<(), OrchestratorError> {
        let session = self.lookup_session(user, session_id)?;
        if device_kind::is_mopidy(&session.device_kind) {
            self.mopidy_driver
                .control(&session.session_id, &cmd)
                .await?;
        } else {
            match &session.agent_id {
                Some(id) => {
                    self.agent_dispatcher
                        .control(id, &session.session_id, &cmd)
                        .await?;
                }
                None => self.local_driver.control(&session.session_id, &cmd).await?,
            }
        }
        Ok(())
    }

    /// Snapshot of the most recent cached status. Live updates flow over
    /// Socket.io independently of this call.
    pub fn status(
        &self,
        user: &Principal,
        session_id: &CastSessionId,
    ) -> Result<CastStatus, OrchestratorError> {
        let session = self.lookup_session(user, session_id)?;
        Ok(session.last_status)
    }

    /// Update the cached last_status — called from the status pump that
    /// also broadcasts over Socket.io. Returns a clone of the active
    /// session (post-update) so the caller has everything it needs to
    /// broadcast the event AND persist watchtime against the session's
    /// owning user / episode.
    pub fn record_status(&self, status: CastStatus) -> Option<ActiveSession> {
        let mut guard = self
            .sessions
            .write()
            .expect("orchestrator session lock poisoned");
        let session = guard.get_mut(&status.session_id)?;
        session.last_status = status;
        Some(session.clone())
    }

    /// Drop a session from the registry. Returns the full session state
    /// (with last cached status) so the caller can broadcast `cast:ended`
    /// AND persist a final watchtime snapshot.
    pub fn drop_session(&self, session_id: &CastSessionId) -> Option<ActiveSession> {
        self.sessions
            .write()
            .expect("orchestrator session lock poisoned")
            .remove(session_id)
    }

    /// The cast targets this context owns.
    pub fn devices(&self) -> &Arc<dyn CastDeviceRepository<Error = CustomError>> {
        &self.devices
    }

    /// Stops and forgets every session of `user_id`, e.g. because the user
    /// was deleted. Stop failures are logged: the device may already be
    /// gone, and the session is dropped either way.
    pub async fn end_sessions_of(&self, user_id: Uuid) -> Vec<ActiveSession> {
        let ended: Vec<ActiveSession> = {
            let mut guard = self
                .sessions
                .write()
                .expect("orchestrator session lock poisoned");
            let ids: Vec<CastSessionId> = guard
                .values()
                .filter(|s| s.user_id == user_id)
                .map(|s| s.session_id.clone())
                .collect();
            ids.iter().filter_map(|id| guard.remove(id)).collect()
        };
        for session in &ended {
            let result = if device_kind::is_mopidy(&session.device_kind) {
                self.mopidy_driver
                    .control(&session.session_id, &ControlCmd::Stop)
                    .await
                    .map_err(OrchestratorError::from)
            } else {
                match &session.agent_id {
                    Some(id) => self
                        .agent_dispatcher
                        .control(id, &session.session_id, &ControlCmd::Stop)
                        .await
                        .map_err(OrchestratorError::from),
                    None => self
                        .local_driver
                        .control(&session.session_id, &ControlCmd::Stop)
                        .await
                        .map_err(OrchestratorError::from),
                }
            };
            if let Err(err) = result {
                warn!(
                    "could not stop cast session {:?}: {err}",
                    session.session_id
                );
            }
        }
        ended
    }

    fn lookup_session(
        &self,
        user: &Principal,
        session_id: &CastSessionId,
    ) -> Result<ActiveSession, OrchestratorError> {
        let guard = self
            .sessions
            .read()
            .expect("orchestrator session lock poisoned");
        let session = guard
            .get(session_id)
            .ok_or(OrchestratorError::SessionNotFound)?;
        // Only the user that started the session can control it. Admins do
        // not get override here on purpose: shared devices can still only be
        // controlled by whoever holds the active session, otherwise users
        // could yank each other's playback.
        if session.user_id != user.id {
            return Err(OrchestratorError::Forbidden);
        }
        Ok(session.clone())
    }
}

/// v1 does not resume mid-episode on Mopidy; always start from the beginning.
fn episode_string_id_position(_episode_string_id: &Option<String>) -> Option<f64> {
    None
}

fn build_target(device: &CastDevice) -> Result<CastTarget, OrchestratorError> {
    let uuid = device
        .chromecast_uuid
        .as_ref()
        .ok_or(OrchestratorError::DeviceNotFound)?
        .clone();
    let ip_str = device
        .ip
        .as_ref()
        .ok_or(OrchestratorError::DeviceUnreachable)?;
    let ip = IpAddr::from_str(ip_str).map_err(|err| {
        warn!("device {} has unparseable ip {}: {err}", uuid, ip_str);
        OrchestratorError::DeviceUnreachable
    })?;
    Ok(CastTarget {
        uuid: CastDeviceUuid(uuid),
        ip,
        // 8009 is the standard CAST receiver TLS port. We persist port too
        // when discovery records it, but the default is correct for every
        // current Chromecast/Google-cast device.
        port: 8009,
    })
}

/// Helper expected by AppState wiring — checks whether the given device kind
/// represents a Chromecast at all (used to filter from generic device lists).
pub fn is_chromecast_kind(kind: &str) -> bool {
    device_kind::is_chromecast(kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::registry::AgentRegistry;
    use chrono::NaiveDate;
    use podfetch_cast::StubCastDriver;

    struct FakeDeviceRepo {
        devices: std::sync::Mutex<Vec<CastDevice>>,
    }

    impl FakeDeviceRepo {
        fn new(devices: Vec<CastDevice>) -> Self {
            Self {
                devices: std::sync::Mutex::new(devices),
            }
        }
    }

    impl CastDeviceRepository for FakeDeviceRepo {
        type Error = CustomError;

        fn register(&self, device: CastDevice) -> Result<CastDevice, Self::Error> {
            self.devices.lock().unwrap().push(device.clone());
            Ok(device)
        }

        fn delete_owned_by(&self, owner: Uuid) -> Result<usize, Self::Error> {
            let mut devices = self.devices.lock().unwrap();
            let before = devices.len();
            devices.retain(|d| d.user_id != owner);
            Ok(before - devices.len())
        }

        fn list_visible_to(&self, viewer_user_id: Uuid) -> Result<Vec<CastDevice>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .filter(|d| {
                    d.kind == device_kind::CHROMECAST_SHARED
                        || d.kind == device_kind::MOPIDY_SHARED
                        || ((d.kind == device_kind::CHROMECAST_PERSONAL
                            || d.kind == device_kind::MOPIDY_PERSONAL)
                            && d.user_id == viewer_user_id)
                })
                .cloned()
                .collect())
        }

        fn find_by_chromecast_uuid(
            &self,
            chromecast_uuid: &str,
        ) -> Result<Option<CastDevice>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.chromecast_uuid.as_deref() == Some(chromecast_uuid))
                .cloned())
        }

        fn find_by_id(&self, id: Uuid) -> Result<Option<CastDevice>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.id == Some(id))
                .cloned())
        }

        fn delete_by_id(&self, id: Uuid) -> Result<usize, Self::Error> {
            let mut devices = self.devices.lock().unwrap();
            let before = devices.len();
            devices.retain(|d| d.id != Some(id));
            Ok(before - devices.len())
        }

        fn upsert_chromecast_from_agent(
            &self,
            _chromecast_uuid: &str,
            _agent_id: &str,
            _owner_user_id: Uuid,
            _name: &str,
            _ip: Option<&str>,
            _last_seen_at: chrono::NaiveDateTime,
        ) -> Result<CastDevice, Self::Error> {
            unimplemented!("not exercised by these tests")
        }
    }

    fn user(id: u128, role: &str) -> Principal {
        Principal::from(&podfetch_identity::domain::user::User::new(
            Uuid::from_u128(id),
            format!("user{id}"),
            role,
            None::<String>,
            NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            true,
        ))
    }

    fn make_device(id: u128, owner: Uuid, kind: &str, uuid: &str) -> CastDevice {
        CastDevice {
            id: Some(Uuid::from_u128(id)),
            deviceid: format!("dev-{id}"),
            kind: kind.to_string(),
            name: format!("CastDevice {id}"),
            user_id: owner,
            chromecast_uuid: Some(uuid.to_string()),
            agent_id: None,
            last_seen_at: None,
            ip: Some("192.168.1.10".to_string()),
            base_url: None,
        }
    }

    fn orchestrator(devices: Vec<CastDevice>) -> CastOrchestrator<StubCastDriver> {
        orchestrator_with_dispatcher(
            devices,
            Arc::new(AgentDispatcher::new(Arc::new(AgentRegistry::new()))),
        )
        .0
    }

    fn orchestrator_with_dispatcher(
        devices: Vec<CastDevice>,
        dispatcher: Arc<AgentDispatcher>,
    ) -> (CastOrchestrator<StubCastDriver>, Arc<AgentRegistry>) {
        let repo = Arc::new(FakeDeviceRepo::new(devices));
        // Tests that need the registry construct the dispatcher from a
        // shared registry; tests that don't can use the simple helper.
        let registry = Arc::new(AgentRegistry::new());
        let (mopidy_tx, _mopidy_rx) = tokio::sync::mpsc::channel(8);
        let mopidy_driver = Arc::new(MopidyDriver::new(mopidy_tx));
        let orch = CastOrchestrator::new(repo, Arc::new(StubCastDriver), dispatcher, mopidy_driver);
        (orch, registry)
    }

    #[test]
    fn user_sees_own_personal_and_all_shared() {
        let alice = user(1, "user");
        let bob = user(2, "user");
        let devices = vec![
            make_device(
                10,
                alice.id.into(),
                device_kind::CHROMECAST_PERSONAL,
                "uuid-alice",
            ),
            make_device(
                11,
                bob.id.into(),
                device_kind::CHROMECAST_PERSONAL,
                "uuid-bob",
            ),
            make_device(
                12,
                Uuid::from_u128(99),
                device_kind::CHROMECAST_SHARED,
                "uuid-shared",
            ),
        ];
        let orch = orchestrator(devices);

        let alice_visible = orch.list_castable(&alice).unwrap();
        let uuids: Vec<_> = alice_visible
            .iter()
            .filter_map(|d| d.chromecast_uuid.clone())
            .collect();
        assert!(uuids.contains(&"uuid-alice".to_string()));
        assert!(uuids.contains(&"uuid-shared".to_string()));
        assert!(!uuids.contains(&"uuid-bob".to_string()));
    }

    #[test]
    fn resolving_invisible_device_returns_not_found() {
        let alice = user(1, "user");
        let devices = vec![make_device(
            11,
            Uuid::from_u128(2),
            device_kind::CHROMECAST_PERSONAL,
            "uuid-bob",
        )];
        let orch = orchestrator(devices);

        match orch.resolve_castable(&alice, "uuid-bob") {
            Err(OrchestratorError::DeviceNotFound) => {}
            other => panic!("expected DeviceNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn non_admin_discover_is_forbidden() {
        let orch = orchestrator(vec![]);
        let regular = user(7, "user");
        match orch.discover(&regular).await {
            Err(OrchestratorError::Forbidden) => {}
            other => panic!("expected Forbidden, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn admin_discover_passes_through_to_driver() {
        let orch = orchestrator(vec![]);
        let admin = user(1, "admin");
        // Stub returns NotImplemented — we just verify we got past the
        // permission gate and saw the driver's response.
        match orch.discover(&admin).await {
            Err(OrchestratorError::Cast(CastError::NotImplemented)) => {}
            other => panic!("expected Cast(NotImplemented), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn start_routes_to_agent_when_device_has_agent_id() {
        use crate::agent::registry::{AgentRegistry, AgentSessionHandle};
        use podfetch_agent_protocol::{AgentMsg, ServerMsg};
        use tokio::sync::mpsc;

        // Wire a shared registry so we can both register an agent and
        // observe what the dispatcher sends it.
        let registry = Arc::new(AgentRegistry::new());
        let dispatcher = Arc::new(AgentDispatcher::new(registry.clone()));

        let (tx, mut wire) = mpsc::channel(16);
        registry.register(AgentSessionHandle::new(
            "agent-1".into(),
            Uuid::from_u128(5),
            "0.1.0".into(),
            tx,
        ));

        let alice = user(5, "user");
        let mut device = make_device(
            21,
            alice.id.into(),
            device_kind::CHROMECAST_PERSONAL,
            "uuid-remote",
        );
        device.agent_id = Some("agent-1".into());

        let repo = Arc::new(FakeDeviceRepo::new(vec![device]));
        let (mopidy_tx2, _mopidy_rx2) = mpsc::channel(8);
        let mopidy_driver2 = Arc::new(MopidyDriver::new(mopidy_tx2));
        let orch = CastOrchestrator::new(
            repo,
            Arc::new(StubCastDriver),
            dispatcher.clone(),
            mopidy_driver2,
        );

        // Drive `start` and intercept the Play that leaves for the agent.
        let alice_clone = alice.clone();
        let dispatcher_clone = dispatcher.clone();
        let task = tokio::spawn(async move {
            let _ = dispatcher_clone; // keep alive for duration
            orch.start(
                &alice_clone,
                "uuid-remote",
                CastMedia {
                    url: "https://x/audio.mp3".into(),
                    mime: "audio/mpeg".into(),
                    title: "Ep".into(),
                    artwork_url: None,
                    duration_secs: Some(60.0),
                    episode_id: None,
                },
                Some(Uuid::from_u128(1)),
                Some("ep-uuid".into()),
            )
            .await
            .map(|s| (s.session_id, s.agent_id))
        });

        let request_id = match wire.recv().await.expect("server msg") {
            ServerMsg::Play { request_id, .. } => request_id,
            other => panic!("expected Play, got {other:?}"),
        };
        dispatcher.complete_pending(
            &request_id,
            AgentMsg::SessionStarted {
                request_id: request_id.clone(),
                session_id: CastSessionId("remote-session".into()),
            },
        );

        let (session_id, agent_id) = task.await.unwrap().unwrap();
        assert_eq!(session_id.0, "remote-session");
        assert_eq!(agent_id.as_deref(), Some("agent-1"));
    }

    #[tokio::test]
    async fn start_against_local_device_uses_local_driver() {
        let alice = user(1, "user");
        let device = make_device(
            10,
            alice.id.into(),
            device_kind::CHROMECAST_PERSONAL,
            "uuid-local",
        );
        let orch = orchestrator(vec![device]);

        // Local driver is the stub which returns NotImplemented; so we
        // assert the call routed through it (rather than the dispatcher).
        let res = orch
            .start(
                &alice,
                "uuid-local",
                CastMedia {
                    url: "https://x/audio.mp3".into(),
                    mime: "audio/mpeg".into(),
                    title: "Ep".into(),
                    artwork_url: None,
                    duration_secs: None,
                    episode_id: None,
                },
                None,
                None,
            )
            .await;
        match res {
            Err(OrchestratorError::Cast(CastError::NotImplemented)) => {}
            other => panic!("expected stub local NotImplemented, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn start_against_mopidy_device_routes_to_mopidy_driver() {
        let alice = user(1, "user");
        let mut device = make_device(
            20,
            alice.id.into(),
            device_kind::MOPIDY_SHARED,
            "mopidy-uuid",
        );
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
        let err = orch.start(&alice, "mopidy-uuid", media, None, None).await;
        assert!(matches!(err, Err(OrchestratorError::Mopidy(_))));
    }

    #[test]
    fn record_status_returns_owning_user() {
        let orch = orchestrator(vec![]);
        let session_id = CastSessionId::new();
        // Manually inject session.
        orch.sessions.write().unwrap().insert(
            session_id.clone(),
            ActiveSession {
                session_id: session_id.clone(),
                device_uuid: CastDeviceUuid("u".into()),
                user_id: Uuid::from_u128(42),
                username: "u42".into(),
                episode_id: None,
                episode_string_id: None,
                agent_id: None,
                device_kind: device_kind::CHROMECAST_PERSONAL.to_string(),
                last_status: CastStatus {
                    session_id: session_id.clone(),
                    state: CastState::Idle,
                    position_secs: 0.0,
                    volume: 1.0,
                    at: Utc::now(),
                },
            },
        );

        let status = CastStatus {
            session_id: session_id.clone(),
            state: CastState::Playing,
            position_secs: 5.0,
            volume: 0.8,
            at: Utc::now(),
        };
        let recorded = orch.record_status(status).expect("known session");
        assert_eq!(recorded.user_id, Uuid::from_u128(42));
        assert_eq!(recorded.last_status.position_secs, 5.0);

        let unknown = CastStatus {
            session_id: CastSessionId::new(),
            state: CastState::Playing,
            position_secs: 0.0,
            volume: 1.0,
            at: Utc::now(),
        };
        assert!(orch.record_status(unknown).is_none());
    }

    fn session_of(user: u128) -> ActiveSession {
        let session_id = CastSessionId::new();
        ActiveSession {
            session_id: session_id.clone(),
            device_uuid: CastDeviceUuid("u".into()),
            user_id: Uuid::from_u128(user),
            username: format!("u{user}"),
            episode_id: None,
            episode_string_id: None,
            agent_id: None,
            device_kind: device_kind::CHROMECAST_PERSONAL.to_string(),
            last_status: CastStatus {
                session_id,
                state: CastState::Playing,
                position_secs: 0.0,
                volume: 1.0,
                at: Utc::now(),
            },
        }
    }

    /// Ending a deleted user's sessions drops them even when the device
    /// can no longer be reached (the stub driver fails every stop), and
    /// leaves other users' sessions alone.
    #[tokio::test]
    async fn end_sessions_of_drops_only_that_users_sessions() {
        let orch = orchestrator(vec![]);
        let mine = [session_of(1), session_of(1)];
        let other = session_of(2);
        {
            let mut sessions = orch.sessions.write().unwrap();
            for session in mine.iter().chain(std::iter::once(&other)) {
                sessions.insert(session.session_id.clone(), session.clone());
            }
        }

        let ended = orch.end_sessions_of(Uuid::from_u128(1)).await;

        assert_eq!(ended.len(), 2);
        assert!(ended.iter().all(|s| s.user_id == Uuid::from_u128(1)));
        let remaining = orch.sessions.read().unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains_key(&other.session_id));
    }
}
