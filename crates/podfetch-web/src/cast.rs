//! HTTP DTOs for the cast API plus the concrete orchestrator type alias used
//! throughout the web layer.
//!
//! Server is currently wired with [`StubCastDriver`] until a real CAST
//! protocol implementation lands. The type alias keeps the rest of the code
//! (controllers, AppState) agnostic to that swap.

use crate::services::cast::service::{ActiveSession, CastOrchestrator};
use podfetch_cast::{
    CastDeviceUuid, CastSessionId, CastState, CastStatus, ControlCmd, DiscoveredCastDevice,
    StubCastDriver,
};
use podfetch_domain::device::Device;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Concrete orchestrator type stored in `AppState`. Swapping the driver in
/// place (e.g. once a real local CAST driver lands) is a one-line change
/// here plus the matching constructor in `AppState::new`.
pub type ServerCastOrchestrator = CastOrchestrator<StubCastDriver>;

/// One Chromecast device as exposed to the UI.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CastDeviceResponse {
    pub chromecast_uuid: String,
    pub name: String,
    /// `chromecast_personal` or `chromecast_shared`.
    pub kind: String,
    /// `None` for devices on the same LAN as the server, otherwise the
    /// agent that contributed this device.
    pub agent_id: Option<String>,
    pub last_seen_at: Option<chrono::NaiveDateTime>,
    pub ip: Option<String>,
}

impl CastDeviceResponse {
    pub fn from_device(device: &Device) -> Option<Self> {
        let chromecast_uuid = device.chromecast_uuid.clone()?;
        Some(Self {
            chromecast_uuid,
            name: device.name.clone(),
            kind: device.kind.clone(),
            agent_id: device.agent_id.clone(),
            last_seen_at: device.last_seen_at,
            ip: device.ip.clone(),
        })
    }
}

/// One device returned by an explicit discovery scan. Distinct from
/// `CastDeviceResponse` because discovery results are *not yet persisted*
/// and have no `kind` until an admin promotes them.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DiscoveredCastDeviceResponse {
    pub uuid: String,
    pub friendly_name: String,
    pub model: Option<String>,
    pub ip: Option<String>,
    pub port: u16,
}

impl From<DiscoveredCastDevice> for DiscoveredCastDeviceResponse {
    fn from(value: DiscoveredCastDevice) -> Self {
        Self {
            uuid: value.uuid.0,
            friendly_name: value.friendly_name,
            model: value.model,
            ip: value.ip.map(|ip| ip.to_string()),
            port: value.port,
        }
    }
}

/// Request body for `POST /cast/sessions`.
#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct CastStartRequest {
    pub chromecast_uuid: String,
    pub episode_id: i32,
    /// Direct URL the Chromecast should fetch. For local server scenarios
    /// the caller resolves this from the episode (proxied or direct file
    /// URL); the controller passes it through unchanged.
    pub url: String,
    pub mime: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub duration_secs: Option<f64>,
}

/// Request body for `POST /cast/sessions/:id/control`.
#[derive(Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "snake_case", tag = "cmd")]
pub enum CastControlRequest {
    Pause,
    Resume,
    Stop,
    Seek { position_secs: f64 },
    SetVolume { volume: f32 },
}

impl From<CastControlRequest> for ControlCmd {
    fn from(value: CastControlRequest) -> Self {
        match value {
            CastControlRequest::Pause => ControlCmd::Pause,
            CastControlRequest::Resume => ControlCmd::Resume,
            CastControlRequest::Stop => ControlCmd::Stop,
            CastControlRequest::Seek { position_secs } => ControlCmd::Seek { position_secs },
            CastControlRequest::SetVolume { volume } => ControlCmd::SetVolume { volume },
        }
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct CastSessionResponse {
    pub session_id: String,
    pub chromecast_uuid: String,
    pub episode_id: Option<i32>,
    pub state: CastStateDto,
    pub position_secs: f64,
    pub volume: f32,
}

impl CastSessionResponse {
    pub fn from_active(session: &ActiveSession) -> Self {
        Self {
            session_id: session.session_id.0.clone(),
            chromecast_uuid: session.device_uuid.0.clone(),
            episode_id: session.episode_id,
            state: session.last_status.state.into(),
            position_secs: session.last_status.position_secs,
            volume: session.last_status.volume,
        }
    }
}

/// Wire-friendly mirror of [`podfetch_cast::CastState`] — gives utoipa a
/// schema without leaking the cast crate's enum into the OpenAPI surface.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CastStateDto {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
}

impl From<CastState> for CastStateDto {
    fn from(value: CastState) -> Self {
        match value {
            CastState::Idle => Self::Idle,
            CastState::Buffering => Self::Buffering,
            CastState::Playing => Self::Playing,
            CastState::Paused => Self::Paused,
            CastState::Stopped => Self::Stopped,
        }
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct CastStatusResponse {
    pub session_id: String,
    pub state: CastStateDto,
    pub position_secs: f64,
    pub volume: f32,
}

impl From<CastStatus> for CastStatusResponse {
    fn from(value: CastStatus) -> Self {
        Self {
            session_id: value.session_id.0,
            state: value.state.into(),
            position_secs: value.position_secs,
            volume: value.volume,
        }
    }
}

pub fn parse_session_id(raw: &str) -> CastSessionId {
    CastSessionId(raw.to_string())
}

pub fn parse_device_uuid(raw: &str) -> CastDeviceUuid {
    CastDeviceUuid(raw.to_string())
}
