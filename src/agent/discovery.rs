//! Live mDNS discovery for `_googlecast._tcp.local.` services.
//!
//! Wraps [`mdns_sd::ServiceDaemon`] in a [`DiscoveryHandle`] that
//! maintains a snapshot of currently-visible Chromecasts and exposes a
//! `wait_for_change()` notification so the WS client can push fresh
//! `DeviceList` messages whenever the LAN topology changes.
//!
//! The actual TXT-record parsing lives in [`parse_cast_record`] so it
//! can be unit-tested without spinning up a real daemon.

use mdns_sd::{ResolvedService, ServiceDaemon, ServiceEvent};
use podfetch_cast::{CastDeviceUuid, DiscoveredCastDevice};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};
use tokio::sync::Notify;
use tracing::{debug, warn};

/// mDNS service type for Chromecast / Google Cast receivers.
const CAST_SERVICE: &str = "_googlecast._tcp.local.";

/// Shared handle wired into the agent client. Cheap to clone.
#[derive(Clone)]
pub struct DiscoveryHandle {
    inner: Arc<DiscoveryInner>,
}

struct DiscoveryInner {
    devices: RwLock<HashMap<String, DiscoveredCastDevice>>,
    changed: Notify,
    _daemon: ServiceDaemon,
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("mdns: {0}")]
    Mdns(#[from] mdns_sd::Error),
}

impl DiscoveryHandle {
    /// Start an mDNS browser and spawn the background loop that keeps
    /// the snapshot up to date.
    pub fn start() -> Result<Self, DiscoveryError> {
        let daemon = ServiceDaemon::new()?;
        let receiver = daemon.browse(CAST_SERVICE)?;

        let inner = Arc::new(DiscoveryInner {
            devices: RwLock::new(HashMap::new()),
            changed: Notify::new(),
            _daemon: daemon,
        });

        let inner_clone = inner.clone();
        tokio::spawn(async move {
            loop {
                match receiver.recv_async().await {
                    Ok(event) => apply_event(&inner_clone, event),
                    Err(err) => {
                        warn!("mdns receiver closed: {err}");
                        break;
                    }
                }
            }
        });

        Ok(Self { inner })
    }

    /// Snapshot the currently-known device list.
    pub fn snapshot(&self) -> Vec<DiscoveredCastDevice> {
        let guard = self
            .inner
            .devices
            .read()
            .expect("discovery snapshot lock poisoned");
        guard.values().cloned().collect()
    }

    /// Resolves whenever the device set changes (add, remove, update).
    pub async fn wait_for_change(&self) {
        self.inner.changed.notified().await;
    }
}

fn apply_event(inner: &Arc<DiscoveryInner>, event: ServiceEvent) {
    let mut changed = false;
    {
        let mut guard = inner
            .devices
            .write()
            .expect("discovery write lock poisoned");
        match event {
            ServiceEvent::ServiceResolved(resolved) => {
                if let Some(device) = resolved_to_device(&resolved) {
                    debug!(
                        uuid = device.uuid.as_ref(),
                        name = %device.friendly_name,
                        "discovered chromecast"
                    );
                    let key = resolved.fullname.clone();
                    let prior = guard.insert(key, device.clone());
                    changed = prior.as_ref() != Some(&device);
                }
            }
            ServiceEvent::ServiceRemoved(_, fullname) => {
                if guard.remove(&fullname).is_some() {
                    changed = true;
                }
            }
            // Other events are informational only.
            _ => {}
        }
    }
    if changed {
        inner.changed.notify_waiters();
    }
}

fn resolved_to_device(service: &ResolvedService) -> Option<DiscoveredCastDevice> {
    let uuid = service.txt_properties.get_property_val_str("id")?;
    let friendly_name = service.txt_properties.get_property_val_str("fn")?;
    let model = service.txt_properties.get_property_val_str("md");
    let ip = service
        .addresses
        .iter()
        .next()
        .map(|scoped| scoped.to_ip_addr());
    parse_cast_record(uuid, friendly_name, model, service.port, ip)
}

/// Pure function that turns the salient fields of a Chromecast mDNS
/// record into our internal device shape. Returns `None` if the required
/// fields (uuid, friendly name) are missing or empty.
pub fn parse_cast_record(
    uuid: &str,
    friendly_name: &str,
    model: Option<&str>,
    port: u16,
    ip: Option<IpAddr>,
) -> Option<DiscoveredCastDevice> {
    if uuid.is_empty() || friendly_name.is_empty() {
        return None;
    }
    Some(DiscoveredCastDevice {
        uuid: CastDeviceUuid(uuid.to_string()),
        friendly_name: friendly_name.to_string(),
        model: model.filter(|s| !s.is_empty()).map(str::to_string),
        ip,
        port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn parses_complete_record() {
        let device = parse_cast_record(
            "uuid-1234",
            "Living Room",
            Some("Chromecast Audio"),
            8009,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50))),
        )
        .expect("should parse");
        assert_eq!(device.uuid.as_ref(), "uuid-1234");
        assert_eq!(device.friendly_name, "Living Room");
        assert_eq!(device.model.as_deref(), Some("Chromecast Audio"));
        assert_eq!(device.port, 8009);
        assert_eq!(
            device.ip,
            Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)))
        );
    }

    #[test]
    fn rejects_empty_uuid() {
        assert!(parse_cast_record("", "name", None, 8009, None).is_none());
    }

    #[test]
    fn rejects_empty_friendly_name() {
        assert!(parse_cast_record("uuid-x", "", None, 8009, None).is_none());
    }

    #[test]
    fn empty_model_string_becomes_none() {
        let device =
            parse_cast_record("uuid-1", "name", Some(""), 8009, None).expect("parse");
        assert!(device.model.is_none());
    }

    #[test]
    fn missing_ip_is_allowed() {
        let device =
            parse_cast_record("uuid-1", "name", Some("model"), 8009, None).expect("parse");
        assert_eq!(device.ip, None);
        assert_eq!(device.port, 8009);
    }

    #[test]
    fn preserves_distinct_devices_across_call_sites() {
        let a = parse_cast_record("uuid-A", "Kitchen", None, 8009, None).unwrap();
        let b = parse_cast_record("uuid-B", "Bedroom", None, 8009, None).unwrap();
        assert_ne!(a, b);
    }
}
