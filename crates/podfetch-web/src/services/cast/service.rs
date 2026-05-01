use crate::services::device::service::DeviceService;
use chrono::Utc;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_cast::{
    CastDeviceUuid, CastDriver, CastError, CastMedia, CastSessionId, CastState, CastStatus,
    CastTarget, ControlCmd, DiscoveredCastDevice,
};
use podfetch_domain::device::{Device, kind as device_kind};
use podfetch_domain::user::User;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tracing::warn;

/// Bookkeeping for an active cast session as seen by the orchestrator.
#[derive(Debug, Clone)]
pub struct ActiveSession {
    pub session_id: CastSessionId,
    pub device_uuid: CastDeviceUuid,
    pub user_id: i32,
    pub episode_id: Option<i32>,
    pub last_status: CastStatus,
}

/// Routes Chromecast operations on behalf of a user, enforces the
/// per-user / shared-device permission model, and tracks active sessions.
///
/// `D` is the backing CAST driver. In production this is a local mDNS+CAST
/// implementation; in tests it is a [`StubCastDriver`].
pub struct CastOrchestrator<D: CastDriver> {
    device_service: Arc<DeviceService>,
    driver: Arc<D>,
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
            OrchestratorError::Cast(e) => CustomErrorInner::BadRequest(
                e.to_string(),
                ErrorSeverity::Warning,
            )
            .into(),
        }
    }
}

impl<D: CastDriver> CastOrchestrator<D> {
    pub fn new(device_service: Arc<DeviceService>, driver: Arc<D>) -> Self {
        Self {
            device_service,
            driver,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// All Chromecast devices the user is allowed to see — owned personal
    /// devices plus any shared/household device on the instance.
    pub fn list_castable(&self, user: &User) -> Result<Vec<Device>, OrchestratorError> {
        Ok(self.device_service.list_castable_for_user(user.id)?)
    }

    /// Resolve a device by chromecast UUID and check the user is allowed to
    /// use it. Returns the persisted Device row.
    pub fn resolve_castable(
        &self,
        user: &User,
        chromecast_uuid: &str,
    ) -> Result<Device, OrchestratorError> {
        let visible = self.list_castable(user)?;
        visible
            .into_iter()
            .find(|d| d.chromecast_uuid.as_deref() == Some(chromecast_uuid))
            .ok_or(OrchestratorError::DeviceNotFound)
    }

    /// Trigger a fresh discovery scan via the driver. Admin-only.
    pub async fn discover(
        &self,
        user: &User,
    ) -> Result<Vec<DiscoveredCastDevice>, OrchestratorError> {
        if !user.is_admin() {
            return Err(OrchestratorError::Forbidden);
        }
        Ok(self.driver.discover().await?)
    }

    /// Start a media session on the given Chromecast.
    pub async fn start(
        &self,
        user: &User,
        chromecast_uuid: &str,
        media: CastMedia,
    ) -> Result<ActiveSession, OrchestratorError> {
        let device = self.resolve_castable(user, chromecast_uuid)?;
        let target = build_target(&device)?;

        let session_id = self.driver.play(&target, &media).await?;
        let status = self.driver.status_snapshot(&session_id).await.unwrap_or(
            CastStatus {
                session_id: session_id.clone(),
                state: CastState::Buffering,
                position_secs: 0.0,
                volume: 1.0,
                at: Utc::now(),
            },
        );
        let active = ActiveSession {
            session_id: session_id.clone(),
            device_uuid: target.uuid,
            user_id: user.id,
            episode_id: media.episode_id,
            last_status: status,
        };
        self.sessions
            .write()
            .expect("orchestrator session lock poisoned")
            .insert(session_id, active.clone());
        Ok(active)
    }

    /// Issue a control command against a session the user owns.
    pub async fn control(
        &self,
        user: &User,
        session_id: &CastSessionId,
        cmd: ControlCmd,
    ) -> Result<(), OrchestratorError> {
        let session = self.lookup_session(user, session_id)?;
        self.driver.control(&session.session_id, &cmd).await?;
        Ok(())
    }

    /// Snapshot of the most recent cached status. Live updates flow over
    /// Socket.io independently of this call.
    pub fn status(
        &self,
        user: &User,
        session_id: &CastSessionId,
    ) -> Result<CastStatus, OrchestratorError> {
        let session = self.lookup_session(user, session_id)?;
        Ok(session.last_status)
    }

    /// Update the cached last_status — called from the status pump that
    /// also broadcasts over Socket.io. Returns the user_id the session
    /// belongs to so the caller knows which room to broadcast into.
    pub fn record_status(&self, status: CastStatus) -> Option<i32> {
        let mut guard = self
            .sessions
            .write()
            .expect("orchestrator session lock poisoned");
        let session = guard.get_mut(&status.session_id)?;
        session.last_status = status;
        Some(session.user_id)
    }

    /// Drop a session from the registry. Returns the user_id if the session
    /// was known, so the caller can broadcast a `cast:ended` event into the
    /// owning user's room.
    pub fn drop_session(&self, session_id: &CastSessionId) -> Option<i32> {
        self.sessions
            .write()
            .expect("orchestrator session lock poisoned")
            .remove(session_id)
            .map(|s| s.user_id)
    }

    fn lookup_session(
        &self,
        user: &User,
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

fn build_target(device: &Device) -> Result<CastTarget, OrchestratorError> {
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
        warn!(
            "device {} has unparseable ip {}: {err}",
            uuid, ip_str
        );
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
    use chrono::NaiveDate;
    use podfetch_cast::StubCastDriver;
    use podfetch_domain::device::DeviceRepository;

    struct FakeDeviceRepo {
        devices: std::sync::Mutex<Vec<Device>>,
    }

    impl FakeDeviceRepo {
        fn new(devices: Vec<Device>) -> Self {
            Self {
                devices: std::sync::Mutex::new(devices),
            }
        }
    }

    impl DeviceRepository for FakeDeviceRepo {
        type Error = CustomError;

        fn create(&self, device: Device) -> Result<Device, Self::Error> {
            self.devices.lock().unwrap().push(device.clone());
            Ok(device)
        }

        fn get_devices_of_user(&self, user_id: i32) -> Result<Vec<Device>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .filter(|d| d.user_id == user_id)
                .cloned()
                .collect())
        }

        fn delete_by_user_id(&self, _user_id: i32) -> Result<(), Self::Error> {
            Ok(())
        }

        fn list_castable_for_user(&self, viewer_user_id: i32) -> Result<Vec<Device>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .filter(|d| {
                    d.kind == device_kind::CHROMECAST_SHARED
                        || (d.kind == device_kind::CHROMECAST_PERSONAL
                            && d.user_id == viewer_user_id)
                })
                .cloned()
                .collect())
        }

        fn find_by_chromecast_uuid(
            &self,
            chromecast_uuid: &str,
        ) -> Result<Option<Device>, Self::Error> {
            Ok(self
                .devices
                .lock()
                .unwrap()
                .iter()
                .find(|d| d.chromecast_uuid.as_deref() == Some(chromecast_uuid))
                .cloned())
        }

        fn upsert_chromecast_from_agent(
            &self,
            _chromecast_uuid: &str,
            _agent_id: &str,
            _owner_user_id: i32,
            _name: &str,
            _ip: Option<&str>,
            _last_seen_at: chrono::NaiveDateTime,
        ) -> Result<Device, Self::Error> {
            unimplemented!("not exercised by these tests")
        }
    }

    fn user(id: i32, role: &str) -> User {
        User::new(
            id,
            format!("user{id}"),
            role,
            None::<String>,
            NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            true,
        )
    }

    fn make_device(id: i32, owner: i32, kind: &str, uuid: &str) -> Device {
        Device {
            id: Some(id),
            deviceid: format!("dev-{id}"),
            kind: kind.to_string(),
            name: format!("Device {id}"),
            user_id: owner,
            chromecast_uuid: Some(uuid.to_string()),
            agent_id: None,
            last_seen_at: None,
            ip: Some("192.168.1.10".to_string()),
        }
    }

    fn orchestrator(devices: Vec<Device>) -> CastOrchestrator<StubCastDriver> {
        let repo = Arc::new(FakeDeviceRepo::new(devices));
        let device_service = Arc::new(DeviceService::new(repo));
        CastOrchestrator::new(device_service, Arc::new(StubCastDriver))
    }

    #[test]
    fn user_sees_own_personal_and_all_shared() {
        let alice = user(1, "user");
        let bob = user(2, "user");
        let devices = vec![
            make_device(10, alice.id, device_kind::CHROMECAST_PERSONAL, "uuid-alice"),
            make_device(11, bob.id, device_kind::CHROMECAST_PERSONAL, "uuid-bob"),
            make_device(12, 99, device_kind::CHROMECAST_SHARED, "uuid-shared"),
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
            2,
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
                user_id: 42,
                episode_id: None,
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
        assert_eq!(orch.record_status(status), Some(42));

        let unknown = CastStatus {
            session_id: CastSessionId::new(),
            state: CastState::Playing,
            position_secs: 0.0,
            volume: 1.0,
            at: Utc::now(),
        };
        assert_eq!(orch.record_status(unknown), None);
    }
}
