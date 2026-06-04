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
