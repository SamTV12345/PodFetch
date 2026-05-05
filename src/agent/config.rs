use std::time::Duration;
use thiserror::Error;

/// Parsed `--agent` CLI configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfig {
    /// Remote PodFetch base URL, e.g. `https://podfetch.example.com` or
    /// `http://192.168.1.5:8000`. The agent appends `/agent/ws` to this.
    pub remote: String,
    /// Existing user API key on the remote instance — used as the bearer
    /// token to authenticate the WS upgrade.
    pub api_key: String,
    /// Stable identifier for this agent installation. Optional; if not
    /// supplied, callers should generate and persist one.
    pub agent_id: Option<String>,
    /// TCP port the agent's local HTTP proxy will bind to. Default 8011.
    pub proxy_port: u16,
    /// Initial reconnect delay; doubles up to `reconnect_max` on failure.
    pub reconnect_initial: Duration,
    /// Cap for the reconnect backoff.
    pub reconnect_max: Duration,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            remote: String::new(),
            api_key: String::new(),
            agent_id: None,
            proxy_port: 8011,
            reconnect_initial: Duration::from_secs(1),
            reconnect_max: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required argument {0}")]
    Missing(&'static str),
    #[error("argument {0} requires a value")]
    NoValue(&'static str),
    #[error("invalid value for {arg}: {message}")]
    BadValue { arg: &'static str, message: String },
}

/// Parse the agent CLI flags from any iterator of String args. The
/// `--agent` flag itself is tolerated (skipped) so callers can forward
/// the unfiltered tail of `std::env::args()`.
///
/// Recognised: `--remote URL`, `--api-key KEY`, `--agent-id ID`,
/// `--proxy-port PORT`.
pub fn parse_from_iter<I>(iter: I) -> Result<AgentConfig, ConfigError>
where
    I: IntoIterator<Item = String>,
{
    let mut iter = iter.into_iter();
    let mut config = AgentConfig::default();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--remote" => {
                config.remote = iter.next().ok_or(ConfigError::NoValue("--remote"))?;
            }
            "--api-key" => {
                config.api_key = iter.next().ok_or(ConfigError::NoValue("--api-key"))?;
            }
            "--agent-id" => {
                config.agent_id = Some(iter.next().ok_or(ConfigError::NoValue("--agent-id"))?);
            }
            "--proxy-port" => {
                let raw = iter.next().ok_or(ConfigError::NoValue("--proxy-port"))?;
                config.proxy_port =
                    raw.parse()
                        .map_err(|err: std::num::ParseIntError| ConfigError::BadValue {
                            arg: "--proxy-port",
                            message: err.to_string(),
                        })?;
            }
            // Tolerate the `--agent` flag if the caller forwarded it.
            "--agent" => {}
            other => {
                return Err(ConfigError::BadValue {
                    arg: "<agent>",
                    message: format!("unrecognised argument: {other}"),
                });
            }
        }
    }

    if config.remote.is_empty() {
        return Err(ConfigError::Missing("--remote"));
    }
    if config.api_key.is_empty() {
        return Err(ConfigError::Missing("--api-key"));
    }
    Ok(config)
}

/// Build the WS URL the agent should connect to from the remote base URL.
/// Accepts http:// and https:// — converts the scheme to ws/wss.
pub fn ws_url(remote: &str) -> Result<String, ConfigError> {
    let trimmed = remote.trim_end_matches('/');
    let with_scheme = if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{rest}")
    } else if trimmed.starts_with("ws://") || trimmed.starts_with("wss://") {
        trimmed.to_string()
    } else {
        return Err(ConfigError::BadValue {
            arg: "--remote",
            message: format!("unsupported scheme in {remote}"),
        });
    };
    Ok(format!("{with_scheme}/agent/ws"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<AgentConfig, ConfigError> {
        parse_from_iter(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn parses_minimal_required_args() {
        let cfg = parse(&["--remote", "http://srv:8000", "--api-key", "k1"]).unwrap();
        assert_eq!(cfg.remote, "http://srv:8000");
        assert_eq!(cfg.api_key, "k1");
        assert_eq!(cfg.agent_id, None);
        assert_eq!(cfg.proxy_port, 8011);
    }

    #[test]
    fn parses_all_optional_args() {
        let cfg = parse(&[
            "--remote",
            "https://srv",
            "--api-key",
            "k1",
            "--agent-id",
            "agent-A",
            "--proxy-port",
            "9090",
        ])
        .unwrap();
        assert_eq!(cfg.agent_id.as_deref(), Some("agent-A"));
        assert_eq!(cfg.proxy_port, 9090);
    }

    #[test]
    fn missing_remote_is_an_error() {
        match parse(&["--api-key", "k1"]) {
            Err(ConfigError::Missing("--remote")) => {}
            other => panic!("expected missing --remote, got {other:?}"),
        }
    }

    #[test]
    fn missing_api_key_is_an_error() {
        match parse(&["--remote", "http://srv:8000"]) {
            Err(ConfigError::Missing("--api-key")) => {}
            other => panic!("expected missing --api-key, got {other:?}"),
        }
    }

    #[test]
    fn unrecognised_argument_is_an_error() {
        match parse(&["--remote", "http://x", "--api-key", "k", "--bogus"]) {
            Err(ConfigError::BadValue { arg: "<agent>", .. }) => {}
            other => panic!("expected bogus arg error, got {other:?}"),
        }
    }

    #[test]
    fn invalid_proxy_port_is_an_error() {
        match parse(&[
            "--remote",
            "http://x",
            "--api-key",
            "k",
            "--proxy-port",
            "not-a-number",
        ]) {
            Err(ConfigError::BadValue {
                arg: "--proxy-port",
                ..
            }) => {}
            other => panic!("expected --proxy-port BadValue, got {other:?}"),
        }
    }

    #[test]
    fn ws_url_converts_http_scheme() {
        assert_eq!(ws_url("http://srv:8000").unwrap(), "ws://srv:8000/agent/ws");
        assert_eq!(
            ws_url("https://srv:8000/").unwrap(),
            "wss://srv:8000/agent/ws"
        );
    }

    #[test]
    fn ws_url_passes_through_ws_scheme() {
        assert_eq!(ws_url("ws://srv:8000").unwrap(), "ws://srv:8000/agent/ws");
    }

    #[test]
    fn ws_url_rejects_unknown_scheme() {
        match ws_url("ftp://srv") {
            Err(ConfigError::BadValue {
                arg: "--remote", ..
            }) => {}
            other => panic!("expected scheme rejection, got {other:?}"),
        }
    }

    #[test]
    fn agent_flag_in_args_is_tolerated() {
        let cfg = parse(&["--agent", "--remote", "http://x", "--api-key", "k"]).unwrap();
        assert_eq!(cfg.remote, "http://x");
    }
}
