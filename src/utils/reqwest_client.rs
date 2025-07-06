use crate::constants::inner_constants::{COMMON_USER_AGENT, ENVIRONMENT_SERVICE};
use reqwest::blocking::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Proxy;

pub fn get_sync_client() -> ClientBuilder {
    let mut res = ClientBuilder::new();
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "User-Agent",
        HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
    );
    if let Some(unwrapped_proxy) = ENVIRONMENT_SERVICE.proxy_url.clone() {
        let proxy = Proxy::all(unwrapped_proxy);
        match proxy {
            Ok(e) => {
                res = res.proxy(e);
            }
            Err(e) => {
                log::error!("Proxy is invalid {e}")
            }
        }
    }

    res.default_headers(header_map)
}
