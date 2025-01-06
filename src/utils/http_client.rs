use crate::constants::inner_constants::{COMMON_USER_AGENT, ENVIRONMENT_SERVICE};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Proxy};
use std::sync::OnceLock;

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_http_client() -> Client {
    HTTP_CLIENT
        .get_or_init(|| get_async_sync_client().build().unwrap())
        .clone()
}

pub fn get_async_sync_client() -> reqwest::ClientBuilder {
    let mut res = reqwest::ClientBuilder::new();
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
                log::error!("Proxy is invalid {}", e)
            }
        }
    }

    res.default_headers(header_map)
}
